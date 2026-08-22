//! Engine Manager 集成测试：使用 Mock UCI 引擎（不依赖真实 Pikafish）。

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use pikaxiangqi_lib::engine::types::{EngineConfig, EngineEvent, GoParams};
use pikaxiangqi_lib::engine::EngineManager;
use tokio::sync::broadcast;
use tokio::time::{sleep, timeout};

fn mock_config(behavior: &str) -> EngineConfig {
    let mut env = HashMap::new();
    if !behavior.is_empty() {
        env.insert("MOCK_BEHAVIOR".to_string(), behavior.to_string());
    }
    EngineConfig {
        program: PathBuf::from(env!("CARGO_BIN_EXE_mock_engine")),
        args: Vec::new(),
        env,
        cwd: None,
        handshake_timeout: Duration::from_secs(3),
    }
}

async fn wait_for<O>(
    rx: &mut broadcast::Receiver<EngineEvent>,
    mut pred: impl FnMut(&EngineEvent) -> Option<O>,
    dur: Duration,
) -> Option<O> {
    timeout(dur, async {
        loop {
            match rx.recv().await {
                Ok(ev) => {
                    if let Some(o) = pred(&ev) {
                        return Some(o);
                    }
                }
                Err(_) => return None,
            }
        }
    })
    .await
    .ok()
    .flatten()
}

#[tokio::test]
async fn handshake_collects_id_and_options() {
    let mgr = EngineManager::spawn(mock_config("")).await.expect("spawn");
    assert_eq!(
        mgr.status(),
        pikaxiangqi_lib::engine::types::EngineStatus::Ready
    );
    assert_eq!(mgr.engine_id().as_deref(), Some("PikaMockEngine"));
    let names: Vec<String> = mgr.options().iter().map(|o| o.name.clone()).collect();
    assert!(names.contains(&"Threads".to_string()));
    assert!(names.contains(&"Hash".to_string()));
    assert!(names.contains(&"MultiPV".to_string()));
    mgr.is_ready().await.expect("isready");
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn set_option_is_forwarded_to_engine() {
    let mgr = EngineManager::spawn(mock_config("")).await.expect("spawn");
    let mut rx = mgr.subscribe();
    mgr.set_option("Threads", Some("2"))
        .await
        .expect("setoption");
    let echo = wait_for(
        &mut rx,
        |ev| match ev {
            EngineEvent::InfoString(s) if s.contains("Threads = 2") => Some(s.clone()),
            _ => None,
        },
        Duration::from_secs(3),
    )
    .await;
    assert!(echo.is_some(), "应收到 mock 引擎的 setoption 回显");
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn analyze_movetime_emits_info_and_bestmove() {
    let mgr = EngineManager::spawn(mock_config("")).await.expect("spawn");
    let mut rx = mgr.subscribe();
    mgr.set_position_and_go(
        None,
        &[],
        GoParams {
            movetime_ms: Some(80),
            ..Default::default()
        },
    )
    .await
    .expect("go");

    let got_info = wait_for(
        &mut rx,
        |ev| match ev {
            EngineEvent::Info(i) if i.depth == Some(8) => Some(()),
            _ => None,
        },
        Duration::from_secs(5),
    )
    .await
    .is_some();
    assert!(got_info, "应收到 info 事件");

    let got_best = wait_for(
        &mut rx,
        |ev| match ev {
            EngineEvent::BestMove(b) if b.mv == "h2e2" => Some(()),
            _ => None,
        },
        Duration::from_secs(5),
    )
    .await
    .is_some();
    assert!(got_best, "应收到 bestmove h2e2");
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn stop_during_infinite_search() {
    let mgr = EngineManager::spawn(mock_config("")).await.expect("spawn");
    let mut rx = mgr.subscribe();
    mgr.go(GoParams {
        infinite: true,
        ..Default::default()
    })
    .await
    .expect("go infinite");
    sleep(Duration::from_millis(80)).await;
    mgr.stop().await.expect("stop");

    let got_best = wait_for(
        &mut rx,
        |ev| match ev {
            EngineEvent::BestMove(b) if b.mv == "h2e2" => Some(()),
            _ => None,
        },
        Duration::from_secs(3),
    )
    .await
    .is_some();
    assert!(got_best, "stop 后应收到 bestmove");
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn position_changed_during_analysis_is_handled() {
    let mgr = EngineManager::spawn(mock_config("")).await.expect("spawn");
    let mut rx = mgr.subscribe();
    // 1) 无限分析
    mgr.set_position_and_go(
        None,
        &[],
        GoParams {
            infinite: true,
            ..Default::default()
        },
    )
    .await
    .expect("go infinite");
    sleep(Duration::from_millis(60)).await;
    // 2) 分析期间切换局面（fen 含 "b0c2"，mock 会回 bestmove b0c2）
    mgr.set_position_and_go(
        Some("mock-fen b0c2"),
        &[],
        GoParams {
            movetime_ms: Some(60),
            ..Default::default()
        },
    )
    .await
    .expect("switch position");

    let mut saw_h2e2 = false;
    let got_b0c2 = wait_for(
        &mut rx,
        |ev| match ev {
            EngineEvent::BestMove(b) => {
                if b.mv == "h2e2" {
                    saw_h2e2 = true;
                }
                if b.mv == "b0c2" {
                    Some(())
                } else {
                    None
                }
            }
            _ => None,
        },
        Duration::from_secs(5),
    )
    .await
    .is_some();
    assert!(saw_h2e2, "停止旧搜索应产出 bestmove h2e2");
    assert!(got_b0c2, "切换局面后应产出 bestmove b0c2");
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn engine_crash_is_detected() {
    let mgr = EngineManager::spawn(mock_config("crash_on_go"))
        .await
        .expect("spawn");
    let mut rx = mgr.subscribe();
    mgr.go(GoParams {
        movetime_ms: Some(50),
        ..Default::default()
    })
    .await
    .expect("go");
    let crashed = wait_for(
        &mut rx,
        |ev| match ev {
            EngineEvent::Crashed { .. } => Some(()),
            _ => None,
        },
        Duration::from_secs(5),
    )
    .await
    .is_some();
    assert!(crashed, "引擎崩溃应发出 Crashed 事件");
    assert_eq!(
        mgr.status(),
        pikaxiangqi_lib::engine::types::EngineStatus::Crashed
    );
}

#[tokio::test]
async fn restart_recovers_after_crash() {
    let mgr = EngineManager::spawn(mock_config("crash_on_go"))
        .await
        .expect("spawn");
    let mut rx = mgr.subscribe();
    mgr.go(GoParams {
        movetime_ms: Some(50),
        ..Default::default()
    })
    .await
    .expect("go");
    let _ = wait_for(
        &mut rx,
        |ev| matches!(ev, EngineEvent::Crashed { .. }).then_some(()),
        Duration::from_secs(5),
    )
    .await;
    mgr.restart().await.expect("restart");
    assert_eq!(
        mgr.status(),
        pikaxiangqi_lib::engine::types::EngineStatus::Ready
    );
    mgr.is_ready().await.expect("isready after restart");
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn startup_failure_without_uciok() {
    let result = EngineManager::spawn(mock_config("no_uciok")).await;
    assert!(result.is_err(), "未收到 uciok 应启动失败");
}

#[tokio::test]
async fn stop_returns_on_hung_engine_timeout() {
    let mgr = EngineManager::spawn(mock_config("hang_on_go"))
        .await
        .expect("spawn");
    mgr.go(GoParams {
        infinite: true,
        ..Default::default()
    })
    .await
    .expect("go");
    sleep(Duration::from_millis(50)).await;
    // 引擎挂起不响应 stop：管理器应在超时后返回，而非无限等待
    let started = std::time::Instant::now();
    mgr.stop().await.expect("stop should time out gracefully");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "stop 应在超时后返回"
    );
    mgr.quit().await.expect("quit");
}

#[tokio::test]
async fn search_start_emits_searching_event() {
    let mgr = EngineManager::spawn(mock_config("")).await.expect("spawn");
    let mut rx = mgr.subscribe();
    mgr.go(GoParams {
        movetime_ms: Some(60),
        ..Default::default()
    })
    .await
    .expect("go");
    let searching = wait_for(
        &mut rx,
        |ev| matches!(ev, EngineEvent::Searching).then_some(()),
        Duration::from_secs(3),
    )
    .await
    .is_some();
    assert!(searching, "搜索开始应发出 Searching 事件");
    // 让搜索自然结束，避免遗留搜索中的进程
    let _ = wait_for(
        &mut rx,
        |ev| matches!(ev, EngineEvent::BestMove(_)).then_some(()),
        Duration::from_secs(3),
    )
    .await;
    mgr.quit().await.expect("quit");
}
