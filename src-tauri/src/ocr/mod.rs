//! 截图识别（OCR）：视觉模型只负责识别，棋规校验由本地 Rust 完成。
//!
//! 分层：
//! - `OcrEngine`（视觉模型抽象）：`recognize_cells` 只做「图片 → 格子分类 + 方向 + 置信度」，
//!   **不做任何规则判断**。
//! - `recognize()`（管线）：调用引擎后，用 `board::validate::validate_position` 做棋规校验，
//!   合成 FEN，汇总 `RecognitionIssue` 与整体置信度；不确定的格子**置空并标记**，不静默接受。
//!
//! 首版引擎：`template::TemplateRecognizer`（传统 CV，确定性模板匹配）。
//! 真实模型（ONNX 等）选型与权重许可见 docs/ocr.md §3（`NEEDS_VERIFICATION`）。

pub mod dto;
pub mod font;
pub mod render;
pub mod template;

use crate::board::fen::to_fen;
use crate::board::types::{Color, Piece, Position, NUM_FILES, NUM_RANKS};
use crate::board::validate::validate_position;

/// 输入：图片字节（PNG/JPEG）。
#[derive(Debug, Clone)]
pub struct OcrInput {
    pub image: Vec<u8>,
}

/// 棋盘方向（Normal = 图像顶部为黑方底线 rank 9；Flipped180 = 旋转 180°）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardOrientation {
    Normal,
    Flipped180,
}

impl BoardOrientation {
    pub fn name(self) -> &'static str {
        match self {
            BoardOrientation::Normal => "normal",
            BoardOrientation::Flipped180 => "flipped180",
        }
    }
}

/// 单个格子的识别结果。
#[derive(Debug, Clone, PartialEq)]
pub struct RecognizedCell {
    pub rank: u8,
    pub file: u8,
    /// 识别出的棋子；None = 空格。
    pub piece: Option<Piece>,
    /// 最佳匹配置信度 [0,1]。
    pub confidence: f32,
    /// 置信度低于阈值 → 不确定（对应格在 FEN 中置空并标记）。
    pub uncertain: bool,
}

/// 识别问题（展示给用户，用于人工校正）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognitionIssue {
    pub message: String,
}

/// 视觉模型产出的原始识别结果（不含规则校验）。
#[derive(Debug, Clone)]
pub struct RawRecognition {
    pub cells: Vec<RecognizedCell>,
    pub orientation: BoardOrientation,
    /// 行棋方（静态截图通常无法判断 → None）。
    pub side_to_move: Option<Color>,
    pub overall_confidence: f32,
}

/// 识别 + 本地规则校验后的最终结果。
#[derive(Debug, Clone)]
pub struct RecognitionOutput {
    pub cells: Vec<RecognizedCell>,
    pub orientation: BoardOrientation,
    pub side_to_move: Option<Color>,
    /// 合成 FEN（不确定格置空；行棋方未知时按红方先行）。
    pub fen: String,
    pub confidence: f32,
    pub issues: Vec<RecognitionIssue>,
    /// 仅当没有任何问题且没有不确定格时为 true。
    pub valid: bool,
}

/// 视觉模型抽象：只负责识别，不判断合法性。
pub trait OcrEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn recognize_cells(&self, input: &OcrInput) -> Result<RawRecognition, OcrError>;
}

/// 截图识别错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrError {
    /// 图片解码失败。
    ImageDecode(String),
    /// 未能在图中定位棋盘。
    BoardNotFound(String),
    /// 图片尺寸异常。
    InvalidImage(String),
}

impl std::fmt::Display for OcrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrError::ImageDecode(e) => write!(f, "图片解码失败：{e}"),
            OcrError::BoardNotFound(e) => write!(f, "无法定位棋盘：{e}"),
            OcrError::InvalidImage(e) => write!(f, "图片无效：{e}"),
        }
    }
}

/// 由格子构建局面（side 未知时按红方先行；由调用方决定）。
pub(crate) fn position_from_cells(cells: &[RecognizedCell], side: Color) -> Position {
    let mut board = [[None; NUM_FILES as usize]; NUM_RANKS as usize];
    for c in cells {
        // 不确定的格子不写入（置空并标记），避免静默接受错误识别。
        if c.uncertain {
            continue;
        }
        if let Some(piece) = c.piece {
            board[c.rank as usize][c.file as usize] = Some(piece);
        }
    }
    Position {
        board,
        side_to_move: side,
        halfmove_clock: 0,
        fullmove_number: 1,
    }
}

/// 识别管线：视觉模型识别 → 本地棋规校验 → 合成 FEN → 汇总问题。
pub fn recognize(engine: &dyn OcrEngine, input: &OcrInput) -> Result<RecognitionOutput, OcrError> {
    let raw = engine.recognize_cells(input)?;

    let mut issues: Vec<RecognitionIssue> = Vec::new();
    if raw.side_to_move.is_none() {
        issues.push(RecognitionIssue {
            message: "无法从静态截图判断行棋方，已按红方先行（可在局面编辑器中切换）".to_string(),
        });
    }

    // 不确定格：置空并标记，提示人工校正。
    let mut uncertain_count = 0usize;
    for c in &raw.cells {
        if c.uncertain {
            uncertain_count += 1;
            issues.push(RecognitionIssue {
                message: format!(
                    "{} 格识别不确定（置信度 {:.0}%）——已置空，请手动摆棋",
                    cell_label(c.rank, c.file),
                    c.confidence * 100.0
                ),
            });
        }
    }

    let side = raw.side_to_move.unwrap_or(Color::Red);
    let pos = position_from_cells(&raw.cells, side);
    let fen = to_fen(&pos);

    // 棋规校验：由本地 Rust 完成（视觉模型不决定合法性）。
    let validation = validate_position(&pos);
    for issue in &validation.issues {
        issues.push(RecognitionIssue {
            message: format!("规则校验：{issue}"),
        });
    }

    let valid = issues.is_empty() && uncertain_count == 0;
    Ok(RecognitionOutput {
        cells: raw.cells,
        orientation: raw.orientation,
        side_to_move: raw.side_to_move,
        fen,
        confidence: raw.overall_confidence,
        issues,
        valid,
    })
}

/// 中文格子标签（如「第 3 行 第 5 列」）。
pub(crate) fn cell_label(rank: u8, file: u8) -> String {
    format!("第 {} 行 第 {} 列", rank + 1, file + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::fen::parse_fen;
    use crate::board::types::{PieceKind, START_FEN};

    #[test]
    fn position_from_cells_skips_uncertain() {
        let mut cells = Vec::new();
        for rank in 0..NUM_RANKS {
            for file in 0..NUM_FILES {
                cells.push(RecognizedCell {
                    rank,
                    file,
                    piece: None,
                    confidence: 1.0,
                    uncertain: false,
                });
            }
        }
        // 一个确定棋子 + 一个不确定棋子（应被忽略）
        cells[0].piece = Some(Piece {
            color: Color::Red,
            kind: PieceKind::King,
        });
        cells[1].piece = Some(Piece {
            color: Color::Black,
            kind: PieceKind::King,
        });
        cells[1].uncertain = true;
        let pos = position_from_cells(&cells, Color::Red);
        assert!(pos.board[0][0].is_some());
        assert!(pos.board[0][1].is_none());
    }

    #[test]
    fn recognize_startpos_pipeline_reports_side_unknown() {
        let start = parse_fen(START_FEN).unwrap();
        let png = render::render_screenshot_png(&start, BoardOrientation::Normal, 48, 24);
        let engine = template::TemplateRecognizer::new();
        let out = recognize(&engine, &OcrInput { image: png }).unwrap();
        // 静态截图无法判断行棋方 → 有一条提示，其余无问题
        assert!(!out.issues.is_empty());
        assert!(out.issues.iter().any(|i| i.message.contains("行棋方")));
        // 合成 FEN 与起始局面一致（红先）
        assert_eq!(out.fen, START_FEN);
    }
}
