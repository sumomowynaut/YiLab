//! 导入导出（Import / Export）：格式无关的内部模型（`GameTree`）与各格式适配器解耦。
//!
//! - `Codec` trait：`parse(text) -> GameTree` / `serialize(tree) -> text`。
//! - 已实现格式：FEN（`fen`）、PGN（`pgn`）。
//! - 后续 XQF / 东萍 / TXT 等格式再扩展；文本型格式先以 `String` 承载，
//!   二进制格式（XQF）落地时再引入字节接口（见 docs/import-export.md）。

pub mod fen;
pub mod pgn;

use crate::game::tree::GameTree;

/// 支持的格式标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Fen,
    Pgn,
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::Fen => "fen",
            Format::Pgn => "pgn",
        }
    }

    pub fn from_name(s: &str) -> Option<Format> {
        match s {
            "fen" => Some(Format::Fen),
            "pgn" => Some(Format::Pgn),
            _ => None,
        }
    }
}

/// 导入导出适配器：文本 <-> 棋谱树。
pub trait Codec: Send + Sync {
    fn format(&self) -> Format;
    fn parse(&self, text: &str) -> Result<GameTree, String>;
    fn serialize(&self, tree: &GameTree) -> Result<String, String>;
}

/// 按格式返回适配器。
pub fn codec(format: Format) -> Box<dyn Codec> {
    match format {
        Format::Fen => Box::new(fen::FenCodec),
        Format::Pgn => Box::new(pgn::PgnCodec),
    }
}

/// 按内容嗅探格式：PGN 头 / 着法标记 / 变例括号 / 注释 → PGN，否则视为 FEN。
pub fn sniff(text: &str) -> Format {
    let t = text.trim();
    if t.starts_with('[') || t.contains("1.") || t.contains('(') || t.contains('{') {
        Format::Pgn
    } else {
        Format::Fen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_name_roundtrip() {
        for f in [Format::Fen, Format::Pgn] {
            assert_eq!(Format::from_name(f.name()), Some(f));
        }
        assert_eq!(Format::from_name("xqf"), None);
    }

    #[test]
    fn sniff_detects_pgn_and_fen() {
        assert_eq!(sniff("[Event \"x\"]\n1. h2e2"), Format::Pgn);
        assert_eq!(sniff("1. h2e2 (1. b0c2) h7e7"), Format::Pgn);
        assert_eq!(sniff(crate::board::types::START_FEN), Format::Fen);
        assert_eq!(sniff("3k5/9/9/9/9/9/9/9/9/K8 b - - 0 1"), Format::Fen);
    }
}
