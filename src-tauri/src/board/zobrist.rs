//! 局面 Zobrist 哈希（开局库检索键 / 重复局面检测）。
//!
//! 哈希表由固定种子的 splitmix64 生成，跨进程/跨运行稳定：
//! 同一局面（棋子布局 + 行棋方）总是得到同一键。
//! 键**不含**半步钟/回合数：开局库检索只关心棋子布局与行棋方。

use super::types::{Color, PieceKind, Position, NUM_FILES, NUM_RANKS};
use std::sync::OnceLock;

const PIECE_KINDS: usize = 7; // King..Pawn
/// 行棋方标记常量（黑方先行时异或）。
const SIDE_BIT: u64 = 0x9e37_79b9_7f4a_7c15;

type ZobristTable = [[[[u64; PIECE_KINDS]; 2]; NUM_FILES as usize]; NUM_RANKS as usize];

struct Tables {
    /// table[rank][file][color(0=red,1=black)][kind]
    table: ZobristTable,
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut seed = 0x1234_5678_9abc_def0u64;
        let mut next = move || {
            // splitmix64
            seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        };
        let mut table = [[[[0u64; PIECE_KINDS]; 2]; NUM_FILES as usize]; NUM_RANKS as usize];
        for rank_row in &mut table {
            for file_col in rank_row {
                for color_table in file_col {
                    for slot in color_table {
                        *slot = next();
                    }
                }
            }
        }
        Tables { table }
    })
}

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

/// 计算局面的 Zobrist 键（棋子布局 + 行棋方）。
pub fn zobrist_key(pos: &Position) -> u64 {
    let t = tables();
    let mut key = 0u64;
    for rank in 0..NUM_RANKS as usize {
        for file in 0..NUM_FILES as usize {
            if let Some(piece) = pos.board[rank][file] {
                let color = match piece.color {
                    Color::Red => 0,
                    Color::Black => 1,
                };
                key ^= t.table[rank][file][color][kind_index(piece.kind)];
            }
        }
    }
    if pos.side_to_move == Color::Black {
        key ^= SIDE_BIT;
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::fen::parse_fen;
    use crate::board::rules::apply_move;
    use crate::board::types::{Move, START_FEN};

    #[test]
    fn deterministic_across_calls() {
        let pos = parse_fen(START_FEN).unwrap();
        assert_eq!(zobrist_key(&pos), zobrist_key(&pos));
    }

    #[test]
    fn startpos_key_is_stable_regression() {
        // 锁定表生成算法，防止未来意外改变哈希语义。
        let pos = parse_fen(START_FEN).unwrap();
        assert_eq!(zobrist_key(&pos), 0x1fe0_5ea9_2489_c535);
    }

    #[test]
    fn different_positions_differ() {
        let a = parse_fen(START_FEN).unwrap();
        let b = apply_move(&a, Move::parse_uci("h2e2").unwrap()).unwrap();
        assert_ne!(zobrist_key(&a), zobrist_key(&b));
    }

    #[test]
    fn side_to_move_changes_key() {
        let red = parse_fen(START_FEN).unwrap();
        let mut black = red.clone();
        black.side_to_move = Color::Black;
        assert_ne!(zobrist_key(&red), zobrist_key(&black));
        assert_eq!(zobrist_key(&black), zobrist_key(&black));
    }
}
