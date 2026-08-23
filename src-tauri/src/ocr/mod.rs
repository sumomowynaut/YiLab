//! 截图识别（OCR）：视觉模型只负责识别，棋规校验由本地 Rust 完成。
//!
//! 分层：
//! - `OcrEngine`（视觉模型抽象）：`recognize_cells` 只做「图片 → 格子分类 + 方向 + 置信度」，
//!   **不做任何规则判断**。
//! - `recognize()`（管线）：调用引擎后，用 `board::validate::validate_position` 做棋规校验，
//!   合成 FEN，汇总 `RecognitionIssue` 与整体置信度；不确定的格子**保留最佳猜测并标记**，不静默接受。
//!
//! 首版引擎：`template::TemplateRecognizer`（传统 CV，确定性模板匹配）。
//! 真实模型（ONNX 等）选型与权重许可见 docs/ocr.md §3（`NEEDS_VERIFICATION`）。

pub mod dto;
pub mod font;
pub mod glyphs;
pub mod render;
pub mod template;

use crate::board::fen::to_fen;
use crate::board::types::{Color, Piece, PieceKind, Position, NUM_FILES, NUM_RANKS};
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
    /// 置信度低于阈值 → 不确定（保留最佳猜测并标记，供人工核对）。
    pub uncertain: bool,
    /// 全部兵种候选（按分数降序），供本地规则修复使用；不会序列化给前端。
    pub alternatives: Vec<(PieceKind, f32)>,
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
        // 只要有最佳猜测就填入；若存在不确定，管线会同时输出问题提示，
        // 避免「整盘置空」让用户无法在编辑器中快速修正。
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

/// 每方各兵种的最大合法数量（超出必然包含误识别）。
const MAX_COUNTS: [u8; 7] = [1, 2, 2, 2, 2, 2, 5];
/// 修复后若替代兵种分数仍低于该值，则标记为不确定。
const REPAIR_CONFIDENCE: f32 = 0.50;

fn kind_index(kind: PieceKind) -> usize {
    match kind {
        PieceKind::King => 0,
        PieceKind::Advisor => 1,
        PieceKind::Elephant => 2,
        PieceKind::Horse => 3,
        PieceKind::Rook => 4,
        PieceKind::Cannon => 5,
        PieceKind::Pawn => 6,
    }
}

/// 用「每方兵种数量上限」纠正明显误识别：
/// 超出上限的兵种中，把置信度最低的格子改判为「仍有空缺」且候选分数最高的兵种。
/// 兵种是否可能出现在该位置（九宫/河界/兵线约束；马、车、炮可任意位置）。
fn kind_legal_at(color: Color, kind: PieceKind, rank: u8, file: u8) -> bool {
    let in_palace = (3..=5).contains(&file)
        && match color {
            Color::Red => rank <= 2,
            Color::Black => rank >= 7,
        };
    match kind {
        PieceKind::King | PieceKind::Advisor => in_palace,
        PieceKind::Elephant => match color {
            Color::Red => rank <= 4,
            Color::Black => rank >= 5,
        },
        PieceKind::Pawn => match color {
            Color::Red => rank <= 3,
            Color::Black => rank >= 6,
        },
        _ => true,
    }
}

fn repair_counts(cells: &mut [RecognizedCell]) {
    for color in [Color::Red, Color::Black] {
        let mut counts = [0u8; 7];
        for c in cells.iter() {
            if let Some(p) = c.piece {
                if p.color == color {
                    counts[kind_index(p.kind)] += 1;
                }
            }
        }
        // 阶段 1：兵种出现在非法区域 → 改判为「该位置合法且有缺额」的候选
        for cell in cells.iter_mut() {
            let Some(p) = cell.piece else { continue };
            if p.color != color {
                continue;
            }
            if kind_legal_at(color, p.kind, cell.rank, cell.file) {
                continue;
            }
            let mut best_target = None;
            let mut best_score = 0.0f32;
            for (kind, score) in &cell.alternatives {
                let ki = kind_index(*kind);
                if ki == kind_index(p.kind)
                    || counts[ki] >= MAX_COUNTS[ki]
                    || !kind_legal_at(color, *kind, cell.rank, cell.file)
                {
                    continue;
                }
                if *score > best_score {
                    best_score = *score;
                    best_target = Some(*kind);
                }
            }
            if let Some(target) = best_target {
                counts[kind_index(p.kind)] -= 1;
                counts[kind_index(target)] += 1;
                cell.piece = Some(Piece {
                    color: p.color,
                    kind: target,
                });
                cell.confidence = best_score;
                cell.uncertain = best_score < REPAIR_CONFIDENCE;
            }
        }

        // 阶段 2：每方每兵种数量上限约束
        loop {
            // 找超出上限最多的兵种
            let mut over = None;
            let mut over_excess = 0u8;
            for (i, max) in MAX_COUNTS.iter().enumerate() {
                if counts[i] > *max && counts[i] - *max > over_excess {
                    over_excess = counts[i] - *max;
                    over = Some(i);
                }
            }
            let Some(oi) = over else { break };

            let mut cands: Vec<usize> = cells
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    matches!(c.piece, Some(Piece { color: cc, kind }) if cc == color && kind_index(kind) == oi)
                })
                .map(|(i, _)| i)
                .collect();
            cands.sort_by(|a, b| cells[*a].confidence.total_cmp(&cells[*b].confidence));

            let mut moved = 0usize;
            for idx in cands {
                if moved >= over_excess as usize {
                    break;
                }
                let Some(piece) = cells[idx].piece else {
                    continue;
                };
                let mut best_target = None;
                let mut best_score = 0.0f32;
                for (kind, score) in &cells[idx].alternatives {
                    let ki = kind_index(*kind);
                    if ki == oi
                        || counts[ki] >= MAX_COUNTS[ki]
                        || !kind_legal_at(color, *kind, cells[idx].rank, cells[idx].file)
                    {
                        continue;
                    }
                    if *score > best_score {
                        best_score = *score;
                        best_target = Some(*kind);
                    }
                }
                if let Some(target) = best_target {
                    counts[oi] -= 1;
                    counts[kind_index(target)] += 1;
                    cells[idx].piece = Some(Piece {
                        color: piece.color,
                        kind: target,
                    });
                    cells[idx].confidence = best_score;
                    cells[idx].uncertain = best_score < REPAIR_CONFIDENCE;
                    moved += 1;
                }
            }
            if moved == 0 {
                break;
            }
        }
    }
}

/// 识别管线：视觉模型识别 → 本地棋规校验 → 合成 FEN → 汇总问题。
pub fn recognize(engine: &dyn OcrEngine, input: &OcrInput) -> Result<RecognitionOutput, OcrError> {
    let mut raw = engine.recognize_cells(input)?;
    // 本地规则修复：利用「每方每兵种数量不能超过合法上限」纠正明显误识别。
    repair_counts(&mut raw.cells);

    let mut issues: Vec<RecognitionIssue> = Vec::new();
    if raw.side_to_move.is_none() {
        issues.push(RecognitionIssue {
            message: "无法从静态截图判断行棋方，已按红方先行（可在局面编辑器中切换）".to_string(),
        });
    }

    // 不确定格：仍保留最佳猜测，但明确提示用户核对，避免整盘置空。
    let mut uncertain_count = 0usize;
    for c in &raw.cells {
        if c.uncertain {
            uncertain_count += 1;
            let action = match c.piece {
                Some(p) => format!("已暂按「{}」填入，请核对", piece_label(p)),
                None => "已置空，请手动摆棋".to_string(),
            };
            issues.push(RecognitionIssue {
                message: format!(
                    "{} 格识别不确定（置信度 {:.0}%）——{}",
                    cell_label(c.rank, c.file),
                    c.confidence * 100.0,
                    action
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

/// 棋子中文标签（如「红车」）。
fn piece_label(piece: Piece) -> String {
    let color = match piece.color {
        Color::Red => "红",
        Color::Black => "黑",
    };
    let kind = match piece.kind {
        PieceKind::King => "将帅",
        PieceKind::Advisor => "士",
        PieceKind::Elephant => "象",
        PieceKind::Horse => "马",
        PieceKind::Rook => "车",
        PieceKind::Cannon => "炮",
        PieceKind::Pawn => "兵卒",
    };
    format!("{color}{kind}")
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
    fn position_from_cells_keeps_best_guess_for_uncertain() {
        let mut cells = Vec::new();
        for rank in 0..NUM_RANKS {
            for file in 0..NUM_FILES {
                cells.push(RecognizedCell {
                    rank,
                    file,
                    piece: None,
                    confidence: 1.0,
                    uncertain: false,
                    alternatives: Vec::new(),
                });
            }
        }
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
        // 不确定但有最佳猜测时仍保留，避免整盘被清空；用户可在编辑器中修正。
        assert!(pos.board[0][0].is_some());
        assert!(pos.board[0][1].is_some());
    }

    #[test]
    fn repair_counts_fixes_overcounted_kinds() {
        let red = Color::Red;
        let mut cells = vec![
            RecognizedCell {
                rank: 0,
                file: 0,
                piece: Some(Piece {
                    color: red,
                    kind: PieceKind::Rook,
                }),
                confidence: 0.9,
                uncertain: false,
                alternatives: vec![(PieceKind::Rook, 0.9), (PieceKind::Horse, 0.80)],
            },
            RecognizedCell {
                rank: 0,
                file: 1,
                piece: Some(Piece {
                    color: red,
                    kind: PieceKind::Rook,
                }),
                confidence: 0.7,
                uncertain: false,
                alternatives: vec![(PieceKind::Rook, 0.7), (PieceKind::Horse, 0.85)],
            },
            RecognizedCell {
                rank: 0,
                file: 2,
                piece: Some(Piece {
                    color: red,
                    kind: PieceKind::Rook,
                }),
                confidence: 0.5,
                uncertain: false,
                alternatives: vec![(PieceKind::Rook, 0.5), (PieceKind::Horse, 0.90)],
            },
        ];
        repair_counts(&mut cells);
        let rooks = cells
            .iter()
            .filter(|c| {
                c.piece
                    == Some(Piece {
                        color: red,
                        kind: PieceKind::Rook,
                    })
            })
            .count();
        let horses = cells
            .iter()
            .filter(|c| {
                c.piece
                    == Some(Piece {
                        color: red,
                        kind: PieceKind::Horse,
                    })
            })
            .count();
        assert!(rooks <= 2, "红车仍超上限: {rooks}");
        assert_eq!(horses, 1, "应把最低置信度的车改判为马");
    }

    #[test]
    fn repair_illegal_zone_reassigns_kind() {
        // 红「象」出现在黑方腹地（rank 8）不合法，候选里马最高 → 应改判为马。
        let mut cells = vec![RecognizedCell {
            rank: 8,
            file: 4,
            piece: Some(Piece {
                color: Color::Red,
                kind: PieceKind::Elephant,
            }),
            confidence: 0.6,
            uncertain: false,
            alternatives: vec![
                (PieceKind::Elephant, 0.6),
                (PieceKind::Horse, 0.75),
                (PieceKind::Rook, 0.70),
            ],
        }];
        repair_counts(&mut cells);
        assert_eq!(
            cells[0].piece,
            Some(Piece {
                color: Color::Red,
                kind: PieceKind::Horse,
            })
        );
        assert!(!cells[0].uncertain);
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
