//! UCI 协议：命令构建与 stdout 解析（纯函数，可独立单元测试）。
//!
//! 参考皮卡鱼 Wiki「UCI 协议」（已核实）：
//! - 中国象棋走法为 UCI-Cyclone 坐标（行从 0 开始）。
//! - 本项目显式书写 `position` 关键字。
//! - `go` 支持 infinite / searchmoves / depth / movetime / nodes / wtime / btime。

use super::types::{BestMove, GoParams, InfoLine, Score, UciOption};

pub fn uci() -> &'static str {
    "uci"
}

pub fn isready() -> &'static str {
    "isready"
}

pub fn stop() -> &'static str {
    "stop"
}

pub fn quit() -> &'static str {
    "quit"
}

/// `setoption name <name> [value <value>]`；button 类选项 value 传 None。
pub fn setoption(name: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("setoption name {name} value {v}"),
        None => format!("setoption name {name}"),
    }
}

/// `position startpos [moves ...]` 或 `position fen <fen> [moves ...]`。
pub fn position(fen: Option<&str>, moves: &[String]) -> String {
    let mut s = match fen {
        Some(f) => format!("position fen {f}"),
        None => "position startpos".to_string(),
    };
    if !moves.is_empty() {
        s.push_str(" moves ");
        s.push_str(&moves.join(" "));
    }
    s
}

/// `go ...`。
pub fn go(params: &GoParams) -> String {
    let mut parts = vec!["go".to_string()];
    if params.infinite {
        parts.push("infinite".to_string());
    }
    if !params.searchmoves.is_empty() {
        parts.push("searchmoves".to_string());
        parts.extend(params.searchmoves.iter().cloned());
    }
    if let Some(d) = params.depth {
        parts.push(format!("depth {d}"));
    }
    if let Some(n) = params.nodes {
        parts.push(format!("nodes {n}"));
    }
    if let Some(ms) = params.movetime_ms {
        parts.push(format!("movetime {ms}"));
    }
    if let Some(w) = params.wtime_ms {
        parts.push(format!("wtime {w}"));
    }
    if let Some(b) = params.btime_ms {
        parts.push(format!("btime {b}"));
    }
    parts.join(" ")
}

/// 解析 `info ...` 行；不是 info 行时返回 None。
pub fn parse_info(line: &str) -> Option<InfoLine> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.first() != Some(&"info") {
        return None;
    }
    let mut out = InfoLine {
        multipv: 1,
        ..Default::default()
    };
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            "depth" => {
                out.depth = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "seldepth" => {
                out.seldepth = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "multipv" => {
                out.multipv = tokens.get(i + 1).and_then(|t| t.parse().ok()).unwrap_or(1);
                i += 2;
            }
            "nodes" => {
                out.nodes = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "nps" => {
                out.nps = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "time" => {
                out.time_ms = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "score" => {
                let kind = tokens.get(i + 1).copied();
                let value = tokens.get(i + 2).and_then(|t| t.parse::<i32>().ok());
                match (kind, value) {
                    (Some("cp"), Some(v)) => out.score = Some(Score::Cp(v)),
                    (Some("mate"), Some(v)) => out.score = Some(Score::Mate(v)),
                    _ => {}
                }
                i += 3;
            }
            "lowerbound" => {
                out.lowerbound = true;
                i += 1;
            }
            "upperbound" => {
                out.upperbound = true;
                i += 1;
            }
            "pv" => {
                out.pv = tokens[i + 1..].iter().map(|s| s.to_string()).collect();
                break;
            }
            _ => i += 1,
        }
    }
    Some(out)
}

/// 解析 `bestmove <mv> [ponder <mv>]`；`bestmove (none)` 返回 None。
pub fn parse_bestmove(line: &str) -> Option<BestMove> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.first() != Some(&"bestmove") {
        return None;
    }
    let mv = tokens.get(1).copied()?;
    if mv == "(none)" {
        return None;
    }
    let mut ponder = None;
    if tokens.get(2) == Some(&"ponder") {
        ponder = tokens.get(3).map(|s| s.to_string());
    }
    Some(BestMove {
        mv: mv.to_string(),
        ponder,
    })
}

/// 解析 `option name <name> type <kind> [default <v>] [min <v>] [max <v>] [var <v>...]`。
pub fn parse_option(line: &str) -> Option<UciOption> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.first() != Some(&"option") {
        return None;
    }
    let mut name: Option<String> = None;
    let mut kind = String::new();
    let mut default: Option<String> = None;
    let mut min: Option<i64> = None;
    let mut max: Option<i64> = None;
    let mut var: Vec<String> = Vec::new();
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            "name" => {
                // 选项名可能含空格：收集到下一个已知关键字
                let mut parts = Vec::new();
                i += 1;
                while i < tokens.len()
                    && !matches!(tokens[i], "type" | "default" | "min" | "max" | "var")
                {
                    parts.push(tokens[i]);
                    i += 1;
                }
                name = Some(parts.join(" "));
            }
            "type" => {
                kind = tokens.get(i + 1).copied().unwrap_or_default().to_string();
                i += 2;
            }
            "default" => {
                default = tokens.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "min" => {
                min = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "max" => {
                max = tokens.get(i + 1).and_then(|t| t.parse().ok());
                i += 2;
            }
            "var" => {
                var.push(tokens.get(i + 1).copied().unwrap_or_default().to_string());
                i += 2;
            }
            _ => i += 1,
        }
    }
    Some(UciOption {
        name: name?,
        kind,
        default,
        min,
        max,
        var,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_info_full_line() {
        let line = "info depth 8 seldepth 10 multipv 2 score cp 35 lowerbound nodes 12345 nps 456789 time 123 pv h2e2 h7e7";
        let info = parse_info(line).expect("parse");
        assert_eq!(info.depth, Some(8));
        assert_eq!(info.seldepth, Some(10));
        assert_eq!(info.multipv, 2);
        assert_eq!(info.score, Some(Score::Cp(35)));
        assert!(info.lowerbound);
        assert_eq!(info.nodes, Some(12345));
        assert_eq!(info.nps, Some(456789));
        assert_eq!(info.time_ms, Some(123));
        assert_eq!(info.pv, vec!["h2e2", "h7e7"]);
    }

    #[test]
    fn parse_info_mate_and_upperbound() {
        let info = parse_info("info depth 12 score mate 3 upperbound pv h2e2").expect("parse");
        assert_eq!(info.score, Some(Score::Mate(3)));
        assert!(info.upperbound);
        assert_eq!(info.pv, vec!["h2e2"]);
    }

    #[test]
    fn parse_info_defaults() {
        let info = parse_info("info depth 1 score cp 0").expect("parse");
        assert_eq!(info.multipv, 1);
        assert!(info.pv.is_empty());
        assert_eq!(info.score, Some(Score::Cp(0)));
    }

    #[test]
    fn parse_info_rejects_non_info() {
        assert!(parse_info("bestmove h2e2").is_none());
        assert!(parse_info("id name X").is_none());
    }

    #[test]
    fn parse_bestmove_variants() {
        assert_eq!(
            parse_bestmove("bestmove h2e2 ponder h7e7"),
            Some(BestMove {
                mv: "h2e2".into(),
                ponder: Some("h7e7".into())
            })
        );
        assert_eq!(
            parse_bestmove("bestmove h2e2"),
            Some(BestMove {
                mv: "h2e2".into(),
                ponder: None
            })
        );
        assert_eq!(parse_bestmove("bestmove (none)"), None);
        assert_eq!(parse_bestmove("info depth 1"), None);
    }

    #[test]
    fn parse_option_spin() {
        let o =
            parse_option("option name Threads type spin default 1 min 1 max 1024").expect("parse");
        assert_eq!(o.name, "Threads");
        assert_eq!(o.kind, "spin");
        assert_eq!(o.default.as_deref(), Some("1"));
        assert_eq!(o.min, Some(1));
        assert_eq!(o.max, Some(1024));
    }

    #[test]
    fn parse_option_multiword_name() {
        let o = parse_option("option name Skill Level type spin default 20 min 0 max 20")
            .expect("parse");
        assert_eq!(o.name, "Skill Level");
    }

    #[test]
    fn parse_option_check_and_var() {
        let o = parse_option("option name Ponder type check default false").expect("parse");
        assert_eq!(o.kind, "check");
        let o = parse_option(
            "option name UCI_Variant type combo default xiangqi var xiangqi var crazyhouse",
        )
        .expect("parse");
        assert_eq!(o.var, vec!["xiangqi", "crazyhouse"]);
    }

    #[test]
    fn builders() {
        assert_eq!(uci(), "uci");
        assert_eq!(isready(), "isready");
        assert_eq!(stop(), "stop");
        assert_eq!(quit(), "quit");
        assert_eq!(
            setoption("Threads", Some("4")),
            "setoption name Threads value 4"
        );
        assert_eq!(setoption("Clear Hash", None), "setoption name Clear Hash");
        assert_eq!(
            position(None, &["h2e2".into(), "h7e7".into()]),
            "position startpos moves h2e2 h7e7"
        );
        assert_eq!(
            position(Some("rnbakabnr/9/..."), &[]),
            "position fen rnbakabnr/9/..."
        );
    }

    #[test]
    fn go_builder_full() {
        let params = GoParams {
            infinite: false,
            depth: Some(12),
            movetime_ms: Some(100),
            nodes: Some(1000),
            searchmoves: vec!["h2e2".into()],
            wtime_ms: Some(60000),
            btime_ms: Some(60000),
        };
        let s = go(&params);
        assert!(s.starts_with("go "));
        assert!(s.contains("searchmoves h2e2"));
        assert!(s.contains("depth 12"));
        assert!(s.contains("nodes 1000"));
        assert!(s.contains("movetime 100"));
        assert!(s.contains("wtime 60000"));
        assert!(s.contains("btime 60000"));
    }

    #[test]
    fn go_builder_infinite() {
        assert_eq!(
            go(&GoParams {
                infinite: true,
                ..Default::default()
            }),
            "go infinite"
        );
    }
}
