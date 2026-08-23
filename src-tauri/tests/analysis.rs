//! 自动复盘集成测试：mock 引擎驱动完整棋谱分析流程。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use pikaxiangqi_lib::analysis::{
    AnalysisConfig, AnalysisEvent, AnalysisStatus, AutoAnalyzer, MoveAssessment, PlannedMove,
};
use pikaxiangqi_lib::board::types::{Move, START_FEN};
use pikaxiangqi_lib::engine::types::EngineConfig;
use pikaxiangqi_lib::engine::EngineManager;
use pikaxiangqi_lib::game::tree::GameTree;
use tokio::time::timeout;

fn mock_config() -> EngineConfig {
    EngineConfig {
        program: PathBuf::from(env!("CARGO_BIN_EXE_mock_engine")),
        args: Vec::new(),
        env: HashMap::new(),
        cwd: None,
        handshake_timeout: Duration::from_secs(3),
    }
}

fn mv(uci: &str) -> Move {
    Move::parse_uci(uci).unwrap()
}

/// 构造 3 步棋的棋谱树并生成分析计划。
fn build_plan() -> (String, Vec<PlannedMove>) {
    let mut tree = GameTree::new(START_FEN).unwrap();
    tree.insert_move(mv("h2e2")).unwrap();
    tree.insert_move(mv("h7e7")).unwrap();
    tree.insert_move(mv("h0g2")).unwrap();
    let mainline = tree.main_line();
    let mut plan = Vec::new();
    for id in mainline.iter().skip(1) {
        let n = tree.node(*id).unwrap();
        plan.push(PlannedMove {
            node_id: *id,
            mv: n.mv.unwrap().uci(),
            is_red: n.is_red(),
        });
    }
    (tree.startpos.clone(), plan)
}

async fn wait_finished(
    rx: &mut tokio::sync::broadcast::Receiver<AnalysisEvent>,
    dur: Duration,
) -> Vec<MoveAssessment> {
    timeout(dur, async {
        loop {
            match rx.recv().await {
                Ok(AnalysisEvent::Finished { assessments }) => return assessments,
                Ok(_) => {}
                Err(_) => return Vec::new(),
            }
        }
    })
    .await
    .ok()
    .unwrap_or_default()
}

#[tokio::test]
async fn analyzes_complete_game_and_records_assessments() {
    let mgr = EngineManager::spawn(mock_config()).await.expect("引擎启动");
    let mgr = Arc::new(mgr);
    let (startpos, moves) = build_plan();
    assert_eq!(moves.len(), 3);

    let analyzer = AutoAnalyzer::new();
    let mut rx = analyzer.subscribe();
    let config = AnalysisConfig {
        depth: Some(9),
        movetime_ms: None,
        ..AnalysisConfig::default()
    };
    analyzer.start(mgr, startpos, moves, config);

    let assessments = wait_finished(&mut rx, Duration::from_secs(20)).await;
    assert_eq!(assessments.len(), 3, "应评估全部 3 步");
    for a in &assessments {
        assert!(!a.mv.is_empty());
        assert!(!a.best_move.is_empty(), "应记录最佳着法");
        assert!(a.depth > 0, "应记录深度");
        assert!(!a.pv.is_empty(), "应记录 PV");
        assert!(a.loss_cp >= 0, "损失应为非负");
        // 损失 = 走子方视角的前后评价差（走子方赚分时钳制为 0）
        let delta = (a.eval_before_cp - a.eval_after_cp).abs();
        assert!(
            a.loss_cp == 0 || a.loss_cp == delta,
            "损失应为 0 或等于前后评价差绝对值（{a:?}）"
        );
    }
    assert_eq!(analyzer.status(), AnalysisStatus::Done, "分析结束应为 Done");
}

#[tokio::test]
async fn stop_then_continue_resumes_analysis() {
    let mgr = Arc::new(EngineManager::spawn(mock_config()).await.expect("引擎启动"));
    let (startpos, moves) = build_plan();
    let analyzer = AutoAnalyzer::new();
    let mut rx = analyzer.subscribe();
    let config = AnalysisConfig {
        depth: Some(9),
        movetime_ms: None,
        ..AnalysisConfig::default()
    };
    analyzer.start(mgr.clone(), startpos, moves, config);

    // 等第一步评估出现后暂停
    timeout(Duration::from_secs(10), async {
        loop {
            match rx.recv().await {
                Ok(AnalysisEvent::Assessment { .. }) => break,
                Ok(_) => {}
                Err(_) => panic!("事件通道关闭"),
            }
        }
    })
    .await
    .expect("应收到至少一条评估");

    analyzer.stop();
    assert_eq!(analyzer.status(), AnalysisStatus::Paused);

    // 继续
    analyzer.resume(mgr);
    let assessments = wait_finished(&mut rx, Duration::from_secs(20)).await;
    assert_eq!(assessments.len(), 3, "继续后应完成全部评估");
    assert_eq!(analyzer.status(), AnalysisStatus::Done);
}

#[tokio::test]
async fn emits_progress_events_during_run() {
    let mgr = Arc::new(EngineManager::spawn(mock_config()).await.expect("引擎启动"));
    let (startpos, moves) = build_plan();
    let analyzer = AutoAnalyzer::new();
    let mut rx = analyzer.subscribe();
    let config = AnalysisConfig {
        depth: Some(9),
        movetime_ms: None,
        ..AnalysisConfig::default()
    };
    analyzer.start(mgr, startpos, moves, config);

    let mut progress_seen = 0usize;
    let mut status_running_seen = false;
    timeout(Duration::from_secs(20), async {
        loop {
            match rx.recv().await {
                Ok(AnalysisEvent::Progress { done, .. }) => {
                    progress_seen = progress_seen.max(done);
                }
                Ok(AnalysisEvent::StatusChanged { status }) => {
                    if status == AnalysisStatus::Running {
                        status_running_seen = true;
                    }
                    if status == AnalysisStatus::Done {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => panic!("事件通道关闭"),
            }
        }
    })
    .await
    .expect("分析应结束");
    assert!(status_running_seen);
    assert_eq!(progress_seen, 3);
}
