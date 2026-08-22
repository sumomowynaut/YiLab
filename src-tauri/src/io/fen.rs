//! FEN 格式适配器：单局面 <-> 棋谱树（以该局面为起始局面）。

use super::{Codec, Format};
use crate::board::fen::parse_fen;
use crate::game::tree::GameTree;

pub struct FenCodec;

impl Codec for FenCodec {
    fn format(&self) -> Format {
        Format::Fen
    }

    fn parse(&self, text: &str) -> Result<GameTree, String> {
        let fen = text.trim();
        parse_fen(fen).map_err(|e| format!("FEN 无效：{e}"))?;
        GameTree::new(fen).map_err(|e| e.to_string())
    }

    fn serialize(&self, tree: &GameTree) -> Result<String, String> {
        Ok(tree.startpos.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::START_FEN;

    #[test]
    fn fen_roundtrip() {
        let tree = FenCodec.parse(START_FEN).unwrap();
        assert_eq!(tree.startpos, START_FEN);
        assert_eq!(FenCodec.serialize(&tree).unwrap(), START_FEN);
    }

    #[test]
    fn fen_rejects_invalid() {
        assert!(FenCodec.parse("garbage").is_err());
        assert!(FenCodec.parse("").is_err());
    }
}
