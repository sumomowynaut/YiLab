//! 棋盘视图变换：180° 旋转（换边）与左右镜像。

use super::types::{Piece, Position, NUM_FILES, NUM_RANKS};

/// 180° 旋转并交换红黑（用于「换边」视角）。
/// 方形 (r, f) 映射到 (9 - r, 8 - f)，棋子颜色取反。
pub fn rotated_180(pos: &Position) -> Position {
    let mut board = [[None; NUM_FILES as usize]; NUM_RANKS as usize];
    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            if let Some(piece) = pos.board[rank as usize][file as usize] {
                board[(NUM_RANKS - 1 - rank) as usize][(NUM_FILES - 1 - file) as usize] =
                    Some(Piece {
                        color: piece.color.opponent(),
                        kind: piece.kind,
                    });
            }
        }
    }
    Position {
        board,
        side_to_move: pos.side_to_move.opponent(),
        halfmove_clock: pos.halfmove_clock,
        fullmove_number: pos.fullmove_number,
    }
}

/// 左右镜像（file a↔i），不交换颜色。
pub fn mirrored(pos: &Position) -> Position {
    let mut board = [[None; NUM_FILES as usize]; NUM_RANKS as usize];
    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            board[rank as usize][(NUM_FILES - 1 - file) as usize] =
                pos.board[rank as usize][file as usize];
        }
    }
    Position {
        board,
        side_to_move: pos.side_to_move,
        halfmove_clock: pos.halfmove_clock,
        fullmove_number: pos.fullmove_number,
    }
}
