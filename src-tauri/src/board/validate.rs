//! 局面规则校验。

use serde::{Deserialize, Serialize};

use super::rules::is_in_check;
use super::types::{Color, PieceKind, Position, Square, NUM_FILES, NUM_RANKS};

/// 校验结果（`ok` 为真表示未发现规则问题）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub ok: bool,
    pub issues: Vec<String>,
}

fn palace_error(sq: Square) -> String {
    format!("{} 存在九宫外的仕/士或将/帅", sq.uci())
}

/// 校验局面的规则合法性（可手工编辑的局面会报告问题，但不阻止继续使用）。
pub fn validate_position(pos: &Position) -> ValidationResult {
    let mut issues: Vec<String> = Vec::new();

    let mut red_king: Option<Square> = None;
    let mut black_king: Option<Square> = None;
    let mut counts = std::collections::HashMap::new();

    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            let Some(piece) = pos.board[rank as usize][file as usize] else {
                continue;
            };
            let sq = Square::new(rank, file).expect("in-bounds square");
            *counts.entry((piece.color, piece.kind)).or_insert(0u8) += 1;

            match piece.kind {
                PieceKind::King => {
                    let slot = match piece.color {
                        Color::Red => &mut red_king,
                        Color::Black => &mut black_king,
                    };
                    if slot.is_some() {
                        issues.push(format!("{} 方存在多个将/帅", color_name(piece.color)));
                    } else {
                        *slot = Some(sq);
                    }
                    if !in_palace(sq, piece.color) {
                        issues.push(palace_error(sq));
                    }
                }
                PieceKind::Advisor => {
                    if !in_palace(sq, piece.color) {
                        issues.push(palace_error(sq));
                    }
                }
                PieceKind::Elephant => {
                    let (min_rank, max_rank) = match piece.color {
                        Color::Red => (0, 4),
                        Color::Black => (5, 9),
                    };
                    if sq.rank < min_rank || sq.rank > max_rank {
                        issues.push(format!("{} 的相/象越过河界", sq.uci()));
                    }
                }
                PieceKind::Pawn => {
                    let valid = match piece.color {
                        Color::Red => sq.rank >= 3,
                        Color::Black => sq.rank <= 6,
                    };
                    if !valid {
                        issues.push(format!("{} 的兵/卒位置不合法", sq.uci()));
                    }
                }
                _ => {}
            }
        }
    }

    if red_king.is_none() {
        issues.push("红方缺少将/帅".to_string());
    }
    if black_king.is_none() {
        issues.push("黑方缺少将/帅".to_string());
    }

    // 数量上限：士/象/马/车/炮 各至多 2，兵/卒至多 5。
    let limits = [
        (PieceKind::Advisor, 2),
        (PieceKind::Elephant, 2),
        (PieceKind::Horse, 2),
        (PieceKind::Rook, 2),
        (PieceKind::Cannon, 2),
        (PieceKind::Pawn, 5),
    ];
    for color in [Color::Red, Color::Black] {
        for (kind, limit) in limits {
            if let Some(&count) = counts.get(&(color, kind)) {
                if count > limit {
                    issues.push(format!(
                        "{} 方 {} 数量 {} 超过上限 {}",
                        color_name(color),
                        kind_name(kind),
                        count,
                        limit
                    ));
                }
            }
        }
    }

    // 上一手方（非行棋方）的王不能被将军。
    let prev_color = pos.side_to_move.opponent();
    let prev_king = match pos.side_to_move {
        Color::Red => black_king,
        Color::Black => red_king,
    };
    if prev_king.is_some() && is_in_check(pos, prev_color) {
        issues.push("非行棋方（上一手方）正被将军，局面非法".to_string());
    }

    ValidationResult {
        ok: issues.is_empty(),
        issues,
    }
}

fn in_palace(sq: Square, color: Color) -> bool {
    sq.file >= 3
        && sq.file <= 5
        && match color {
            Color::Red => sq.rank <= 2,
            Color::Black => sq.rank >= 7,
        }
}

fn color_name(c: Color) -> &'static str {
    match c {
        Color::Red => "红",
        Color::Black => "黑",
    }
}

fn kind_name(k: PieceKind) -> &'static str {
    match k {
        PieceKind::King => "将/帅",
        PieceKind::Advisor => "仕/士",
        PieceKind::Elephant => "相/象",
        PieceKind::Horse => "马",
        PieceKind::Rook => "车",
        PieceKind::Cannon => "炮",
        PieceKind::Pawn => "兵/卒",
    }
}
