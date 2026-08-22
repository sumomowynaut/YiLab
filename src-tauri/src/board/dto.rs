//! 前端可序列化的局面 DTO。

use serde::Serialize;

use super::fen::to_fen;
use super::types::{BoardArray, Position};

/// 发送给 React 前端的局面快照。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionDto {
    pub board: BoardArray,
    pub side_to_move: String,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub fen: String,
}

impl PositionDto {
    pub fn from_position(pos: &Position) -> Self {
        PositionDto {
            board: pos.board,
            side_to_move: pos.side_to_move.fen_char().to_string(),
            halfmove_clock: pos.halfmove_clock,
            fullmove_number: pos.fullmove_number,
            fen: to_fen(pos),
        }
    }
}
