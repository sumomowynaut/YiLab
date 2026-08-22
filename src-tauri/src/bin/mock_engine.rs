//! Mock UCI 引擎：用于 Engine Manager 集成测试（无需真实 Pikafish）。
//!
//! 行为可通过环境变量 `MOCK_BEHAVIOR` 控制：
//! - （空）  ：正常行为
//! - no_uciok    ：`uci` 后不发送 `uciok`（启动失败测试）
//! - no_readyok  ：`isready` 后不发送 `readyok`（启动失败测试）
//! - crash_on_go ：`go` 后立即退出（崩溃测试）
//! - hang_on_go  ：`go` 后挂起不响应（停止超时测试）

use std::io::{self, BufRead, Write};
use std::time::Duration;

fn param_value<'a>(parts: &[&'a str], key: &str) -> Option<&'a str> {
    parts.windows(2).find(|w| w[0] == key).map(|w| w[1])
}

fn emit_search_result(out: &mut io::Stdout, position_line: &str) {
    let _ = writeln!(out, "info depth 8 seldepth 10 multipv 1 score cp 35 nodes 12345 nps 456789 time 123 pv h2e2 h7e7");
    let _ = writeln!(out, "info depth 9 seldepth 11 multipv 1 score cp 38 nodes 23456 nps 456789 time 234 pv h2e2 h7e7 h0g2");
    // 若局面行包含特定 FEN 片段，可换一个 bestmove（用于「分析期间切换局面」测试）
    if position_line.contains("b0c2") {
        let _ = writeln!(out, "bestmove b0c2 ponder b9c7");
    } else {
        let _ = writeln!(out, "bestmove h2e2 ponder h7e7");
    }
}

fn main() {
    let behavior = std::env::var("MOCK_BEHAVIOR").unwrap_or_default();
    let stdin = io::stdin();
    let mut out = io::stdout();
    let mut searching = false;
    let mut position_line = String::new();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.first().copied() {
            Some("uci") => {
                let _ = writeln!(out, "id name PikaMockEngine");
                let _ = writeln!(out, "id author PikaXiangqi");
                let _ = writeln!(
                    out,
                    "option name Threads type spin default 1 min 1 max 1024"
                );
                let _ = writeln!(
                    out,
                    "option name Hash type spin default 16 min 1 max 33554432"
                );
                let _ = writeln!(out, "option name MultiPV type spin default 1 min 1 max 500");
                let _ = writeln!(out, "option name Ponder type check default false");
                if behavior == "no_uciok" {
                    let _ = out.flush();
                    continue;
                }
                let _ = writeln!(out, "uciok");
            }
            Some("isready") => {
                if behavior != "no_readyok" {
                    let _ = writeln!(out, "readyok");
                }
            }
            Some("setoption") => {
                // setoption name <name> [value <value>]
                let rest = parts[1..].join(" ");
                if let Some(name) = rest.strip_prefix("name ") {
                    if let Some((n, v)) = name.split_once(" value ") {
                        let _ = writeln!(out, "info string option {n} = {v}");
                    }
                }
            }
            Some("position") => {
                position_line = line.clone();
            }
            Some("go") => {
                if behavior == "crash_on_go" {
                    let _ = out.flush();
                    std::process::exit(1);
                }
                if behavior == "hang_on_go" {
                    loop {
                        std::thread::sleep(Duration::from_secs(3600));
                    }
                }
                let params = &parts[1..];
                let infinite = params.contains(&"infinite");
                searching = true;
                if !infinite {
                    if let Some(ms) = param_value(params, "movetime") {
                        if let Ok(ms) = ms.parse::<u64>() {
                            std::thread::sleep(Duration::from_millis(ms));
                        }
                    } else if param_value(params, "depth").is_some() {
                        std::thread::sleep(Duration::from_millis(30));
                    } else {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    emit_search_result(&mut out, &position_line);
                    searching = false;
                }
                // infinite：等待 `stop`
            }
            Some("stop") => {
                if searching {
                    searching = false;
                    let _ = writeln!(out, "bestmove h2e2");
                }
            }
            Some("quit") => {
                let _ = out.flush();
                return;
            }
            _ => {}
        }
        let _ = out.flush();
    }
}
