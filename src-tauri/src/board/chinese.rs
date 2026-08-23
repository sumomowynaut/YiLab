//! 中国象棋中文纵线制记谱（炮二平五 / 马8进7 等）。
//!
//! 约定：
//! - 红方纵线从右到左为一~九（file 8=一 … file 0=九）。
//! - 黑方纵线从右到左为 1~9（file 0=1 … file 8=9）。
//! - 横走（同 rank）用「平」；纵走用「进/退」。
//! - 马/相/仕 走斜线时，「进/退」后跟目标纵线号；其余棋子跟步数。

use super::types::{Color, Move, Piece, PieceKind, Position};

fn glyph(piece: Piece) -> &'static str {
    match (piece.color, piece.kind) {
        (Color::Red, PieceKind::King) => "帅",
        (Color::Red, PieceKind::Advisor) => "仕",
        (Color::Red, PieceKind::Elephant) => "相",
        (Color::Red, PieceKind::Horse) => "马",
        (Color::Red, PieceKind::Rook) => "车",
        (Color::Red, PieceKind::Cannon) => "炮",
        (Color::Red, PieceKind::Pawn) => "兵",
        (Color::Black, PieceKind::King) => "将",
        (Color::Black, PieceKind::Advisor) => "士",
        (Color::Black, PieceKind::Elephant) => "象",
        (Color::Black, PieceKind::Horse) => "马",
        (Color::Black, PieceKind::Rook) => "车",
        (Color::Black, PieceKind::Cannon) => "炮",
        (Color::Black, PieceKind::Pawn) => "卒",
    }
}

const RED_FILES: [&str; 9] = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];

/// 纵线号（1..9）。红方 file 8→1、file 0→9；黑方 file 0→1、file 8→9。
fn file_no(file: u8, red: bool) -> u8 {
    if red {
        9 - file
    } else {
        file + 1
    }
}

fn num_label(n: usize, red: bool) -> String {
    if red {
        RED_FILES
            .get(n - 1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| n.to_string())
    } else {
        n.to_string()
    }
}

fn file_label(file: u8, red: bool) -> String {
    num_label(file_no(file, red) as usize, red)
}

/// 把一个着法转成中文纵线制记谱。`pos` 是走子前的局面。
pub fn move_to_chinese(pos: &Position, mv: &Move) -> String {
    let Some(piece) = pos.board[mv.from.rank as usize][mv.from.file as usize] else {
        return mv.uci();
    };
    let red = piece.color == Color::Red;
    let g = glyph(piece);
    let from = file_label(mv.from.file, red);

    if mv.from.rank == mv.to.rank {
        return format!("{g}{from}平{}", file_label(mv.to.file, red));
    }

    let advancing = if red {
        mv.to.rank > mv.from.rank
    } else {
        mv.to.rank < mv.from.rank
    };
    let action = if advancing { "进" } else { "退" };
    let diagonal = matches!(
        piece.kind,
        PieceKind::Horse | PieceKind::Elephant | PieceKind::Advisor
    );
    if diagonal {
        format!("{g}{from}{action}{}", file_label(mv.to.file, red))
    } else {
        let steps = (mv.to.rank as i16 - mv.from.rank as i16).unsigned_abs();
        format!("{g}{from}{action}{}", num_label(steps as usize, red))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::fen::parse_fen;
    use crate::board::types::START_FEN;

    fn mv(uci: &str) -> Move {
        Move::parse_uci(uci).unwrap()
    }

    #[test]
    fn cannon_flat_red() {
        // 红炮 h2 → e2（炮二平五）
        let pos = parse_fen(START_FEN).unwrap();
        assert_eq!(move_to_chinese(&pos, &mv("h2e2")), "炮二平五");
    }

    #[test]
    fn horse_diagonal_black() {
        // 黑马 h9→g7：file 7（黑方 8 线）→ file 6（黑方 7 线），rank 9→7 为「进」
        let pos = parse_fen(START_FEN).unwrap();
        assert_eq!(move_to_chinese(&pos, &mv("h9g7")), "马8进7");
    }

    #[test]
    fn rook_advance_red() {
        let pos = parse_fen(START_FEN).unwrap();
        // 红车 a0→a1：file 0（红方 九 线）进 1 步
        assert_eq!(move_to_chinese(&pos, &mv("a0a1")), "车九进一");
    }
}
