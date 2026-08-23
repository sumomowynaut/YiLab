//! 真实 Pikafish 引擎冒烟测试。
//!
//! 默认 `#[ignore]`：需要真实引擎二进制，运行时设置环境变量 `PIKAFISH_BIN` 指向引擎路径。
//! 例：`PIKAFISH_BIN=...\Windows\pikafish-avx2.exe cargo test --test pikafish_smoke -- --ignored`（权重经 discover_eval_file 自动发现；也可用 PIKAFISH_EVALFILE 显式指定）
//!
//! 注意：本测试只在本机运行引擎做冒烟验证，不涉及任何分发/捆绑；
//! 分发相关实现受许可证决策约束（见 docs/licensing.md，当前不做判断）。

use std::time::Duration;

use pikaxiangqi_lib::engine::manager::discover_eval_file;
use pikaxiangqi_lib::engine::types::{EngineConfig, EngineEvent, EngineStatus, GoParams};
use pikaxiangqi_lib::engine::EngineManager;
use tokio::time::timeout;

#[tokio::test]
#[ignore]
async fn pikafish_handshake_and_analyze() {
    let bin = std::env::var("PIKAFISH_BIN").expect("请设置 PIKAFISH_BIN 指向真实 Pikafish 二进制");
    let bin_path = std::path::PathBuf::from(&bin);
    // 权重：优先读 PIKAFISH_EVALFILE；否则自动发现（exe 同目录 / 上一级目录）。
    // 这同时验证了 B1 的「无需手动复制 NNUE」路径。
    let eval_file = std::env::var("PIKAFISH_EVALFILE")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| discover_eval_file(&bin_path));
    // 工作目录设为权重所在目录，引擎用相对默认值 `pikafish.nnue` 加载（兼容中文/空格路径）。
    let cwd = std::env::var("PIKAFISH_CWD")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| {
            eval_file
                .as_ref()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        });
    let config = EngineConfig {
        program: bin_path,
        args: Vec::new(),
        env: std::collections::HashMap::new(),
        cwd,
        handshake_timeout: Duration::from_secs(15),
    };
    let mgr = EngineManager::spawn(config).await.expect("启动 Pikafish");
    assert_eq!(mgr.status(), EngineStatus::Ready);
    assert!(
        !mgr.engine_id().unwrap_or_default().is_empty(),
        "应能读取引擎 id"
    );

    let names: Vec<String> = mgr.options().iter().map(|o| o.name.clone()).collect();
    for required in ["Threads", "Hash", "MultiPV"] {
        assert!(
            names.contains(&required.to_string()),
            "引擎缺少选项 {required}"
        );
    }

    mgr.set_option("Threads", Some("1")).await.expect("Threads");
    mgr.set_option("Hash", Some("16")).await.expect("Hash");
    mgr.set_option("MultiPV", Some("2")).await.expect("MultiPV");
    mgr.is_ready().await.expect("isready");

    let mut rx = mgr.subscribe();
    mgr.set_position_and_go(
        None,
        &[],
        GoParams {
            movetime_ms: Some(400),
            ..Default::default()
        },
    )
    .await
    .expect("go");

    let mut saw_info = false;
    let mut saw_multipv2 = false;
    let mut saw_bestmove = false;
    timeout(Duration::from_secs(20), async {
        loop {
            match rx.recv().await {
                Ok(EngineEvent::Info(i)) => {
                    if i.depth.is_some() {
                        saw_info = true;
                    }
                    if i.multipv >= 2 {
                        saw_multipv2 = true;
                    }
                }
                Ok(EngineEvent::BestMove(_)) => {
                    saw_bestmove = true;
                    break;
                }
                Ok(EngineEvent::Error(e)) => panic!("引擎错误：{e}"),
                Ok(EngineEvent::Crashed { code }) => panic!("引擎崩溃：{code:?}"),
                _ => {}
            }
        }
    })
    .await
    .expect("等待 bestmove 超时");

    assert!(saw_info, "应收到至少一条 info");
    assert!(saw_multipv2, "MultiPV=2 时应收到 multipv>=2 的 info");
    assert!(saw_bestmove, "应收到 bestmove");

    mgr.quit().await.expect("quit");
}
