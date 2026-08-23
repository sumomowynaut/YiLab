//! 截图识别结果 DTO（发送给 React 前端）。

use serde::Serialize;

use crate::board::types::{Color, PieceKind};

use super::{BoardOrientation, RecognitionOutput};

/// 棋子 DTO（color: red/black，kind: king/advisor/elephant/horse/rook/cannon/pawn）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceDto {
    pub color: String,
    pub kind: String,
}

fn color_name(c: Color) -> &'static str {
    match c {
        Color::Red => "red",
        Color::Black => "black",
    }
}

fn kind_name(k: PieceKind) -> &'static str {
    match k {
        PieceKind::King => "king",
        PieceKind::Advisor => "advisor",
        PieceKind::Elephant => "elephant",
        PieceKind::Horse => "horse",
        PieceKind::Rook => "rook",
        PieceKind::Cannon => "cannon",
        PieceKind::Pawn => "pawn",
    }
}

/// 单个格子的识别结果 DTO。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrCellDto {
    pub rank: u8,
    pub file: u8,
    pub piece: Option<PieceDto>,
    pub confidence: f64,
    pub uncertain: bool,
}

/// 识别结果 DTO。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrResultDto {
    pub cells: Vec<OcrCellDto>,
    pub orientation: String,
    pub side_to_move: Option<String>,
    pub fen: String,
    pub confidence: f64,
    pub valid: bool,
    pub issues: Vec<String>,
}

impl From<&RecognitionOutput> for OcrResultDto {
    fn from(out: &RecognitionOutput) -> Self {
        OcrResultDto {
            cells: out
                .cells
                .iter()
                .map(|c| OcrCellDto {
                    rank: c.rank,
                    file: c.file,
                    piece: c.piece.map(|p| PieceDto {
                        color: color_name(p.color).to_string(),
                        kind: kind_name(p.kind).to_string(),
                    }),
                    confidence: c.confidence as f64,
                    uncertain: c.uncertain,
                })
                .collect(),
            orientation: out.orientation.name().to_string(),
            side_to_move: out.side_to_move.map(|c| color_name(c).to_string()),
            fen: out.fen.clone(),
            confidence: out.confidence as f64,
            valid: out.valid,
            issues: out.issues.iter().map(|i| i.message.clone()).collect(),
        }
    }
}

/// 供 `BoardOrientation` 反查（DTO 展示）。
pub fn orientation_from_name(s: &str) -> Option<BoardOrientation> {
    match s {
        "normal" => Some(BoardOrientation::Normal),
        "flipped180" => Some(BoardOrientation::Flipped180),
        _ => None,
    }
}
