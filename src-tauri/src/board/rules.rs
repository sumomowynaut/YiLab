//! 中国象棋规则核心：着法生成、合法性、将军/将死/困毙、perft。
//!
//! 规则要点（与 docs/game-model.md 一致）：
//! - 将/帅：九宫内直行一步；两将不可照面（飞将）。
//! - 仕/士：九宫内斜行一步。
//! - 相/象：田字斜走两步，塞象眼，不可过河。
//! - 马：日字走法，蹩马腿。
//! - 车：直线滑动。
//! - 炮：直线滑动不吃子；吃子须隔一个炮架。
//! - 兵/卒：向前一步，过河后可横走，不可后退。
//! - 无升变。

use super::types::{Color, Move, Piece, PieceKind, Position, Square, NUM_FILES, NUM_RANKS};

/// 某方王是否在九宫内。
fn in_palace(sq: Square, color: Color) -> bool {
    sq.file >= 3
        && sq.file <= 5
        && match color {
            Color::Red => sq.rank <= 2,
            Color::Black => sq.rank >= 7,
        }
}

/// 某方相/象允许的行区间（不可过河）。
fn elephant_rank_range(color: Color) -> (u8, u8) {
    match color {
        Color::Red => (0, 4),
        Color::Black => (5, 9),
    }
}

/// 兵/卒是否已过河（红方 rank >= 5，黑方 rank <= 4）。
fn crossed_river(sq: Square, color: Color) -> bool {
    match color {
        Color::Red => sq.rank >= 5,
        Color::Black => sq.rank <= 4,
    }
}

/// 单枚棋子的伪合法目标格（不考虑己方王安全）。
pub(crate) fn piece_targets(pos: &Position, piece: Piece, from: Square) -> Vec<Square> {
    let mut out = Vec::new();
    match piece.kind {
        PieceKind::King => king_targets(from, piece.color, &mut out),
        PieceKind::Advisor => advisor_targets(from, piece.color, &mut out),
        PieceKind::Elephant => elephant_targets(pos, from, piece.color, &mut out),
        PieceKind::Horse => horse_targets(pos, from, &mut out),
        PieceKind::Rook => rook_targets(pos, from, &mut out),
        PieceKind::Cannon => cannon_targets(pos, from, &mut out),
        PieceKind::Pawn => pawn_targets(from, piece.color, &mut out),
    }
    out
}

fn king_targets(from: Square, color: Color, out: &mut Vec<Square>) {
    for (dr, df) in [(1i8, 0i8), (-1, 0), (0, 1), (0, -1)] {
        if let Some(sq) = square_at(from, dr, df) {
            if in_palace(sq, color) {
                out.push(sq);
            }
        }
    }
}

fn advisor_targets(from: Square, color: Color, out: &mut Vec<Square>) {
    for (dr, df) in [(1i8, 1i8), (1, -1), (-1, 1), (-1, -1)] {
        if let Some(sq) = square_at(from, dr, df) {
            if in_palace(sq, color) {
                out.push(sq);
            }
        }
    }
}

fn elephant_targets(pos: &Position, from: Square, color: Color, out: &mut Vec<Square>) {
    let (min_rank, max_rank) = elephant_rank_range(color);
    for (dr, df) in [(2i8, 2i8), (2, -2), (-2, 2), (-2, -2)] {
        let Some(sq) = square_at(from, dr, df) else {
            continue;
        };
        if sq.rank < min_rank || sq.rank > max_rank {
            continue;
        }
        // 塞象眼：斜走两步的中间格。
        let eye = square_at(from, dr / 2, df / 2).expect("eye is on board");
        if pos.board[eye.rank as usize][eye.file as usize].is_none() {
            out.push(sq);
        }
    }
}

fn horse_targets(pos: &Position, from: Square, out: &mut Vec<Square>) {
    for (dr, df) in [
        (2i8, 1i8),
        (2, -1),
        (-2, 1),
        (-2, -1),
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2),
    ] {
        let Some(sq) = square_at(from, dr, df) else {
            continue;
        };
        // 蹩马腿：沿两格方向的第一步格。
        let leg = square_at(from, dr / 2, df / 2).expect("leg is on board");
        if pos.board[leg.rank as usize][leg.file as usize].is_none() {
            out.push(sq);
        }
    }
}

fn rook_targets(pos: &Position, from: Square, out: &mut Vec<Square>) {
    slide(pos, from, out);
}

fn cannon_targets(pos: &Position, from: Square, out: &mut Vec<Square>) {
    for (dr, df) in [(1i8, 0i8), (-1, 0), (0, 1), (0, -1)] {
        let mut r = from.rank as i8 + dr;
        let mut f = from.file as i8 + df;
        let mut jumped = false;
        while let Some(sq) = Square::new(r as u8, f as u8) {
            match pos.board[sq.rank as usize][sq.file as usize] {
                None => {
                    if !jumped {
                        out.push(sq);
                    }
                }
                Some(_) => {
                    if !jumped {
                        jumped = true; // 找到炮架，继续寻找吃子目标
                    } else {
                        out.push(sq); // 越过炮架后第一个棋子（吃或挡）
                        break;
                    }
                }
            }
            r += dr;
            f += df;
        }
    }
}

/// 车式直线滑动：空位可走，第一枚敌子可吃，之后停止。
fn slide(pos: &Position, from: Square, out: &mut Vec<Square>) {
    let own = pos.board[from.rank as usize][from.file as usize].map(|p| p.color);
    for (dr, df) in [(1i8, 0i8), (-1, 0), (0, 1), (0, -1)] {
        let mut r = from.rank as i8 + dr;
        let mut f = from.file as i8 + df;
        while let Some(sq) = Square::new(r as u8, f as u8) {
            match pos.board[sq.rank as usize][sq.file as usize] {
                None => out.push(sq),
                Some(p) => {
                    if own != Some(p.color) {
                        out.push(sq);
                    }
                    break;
                }
            }
            r += dr;
            f += df;
        }
    }
}

fn pawn_targets(from: Square, color: Color, out: &mut Vec<Square>) {
    let fwd: i8 = match color {
        Color::Red => 1,
        Color::Black => -1,
    };
    if let Some(sq) = square_at(from, fwd, 0) {
        out.push(sq);
    }
    if crossed_river(from, color) {
        for df in [-1i8, 1] {
            if let Some(sq) = square_at(from, 0, df) {
                out.push(sq);
            }
        }
    }
}

fn square_at(from: Square, dr: i8, df: i8) -> Option<Square> {
    Square::new((from.rank as i8 + dr) as u8, (from.file as i8 + df) as u8)
}

/// 伪合法着法（不含己方安全过滤）。
pub fn pseudo_legal_moves(pos: &Position) -> Vec<Move> {
    let mut moves = Vec::new();
    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            let Some(piece) = pos.board[rank as usize][file as usize] else {
                continue;
            };
            if piece.color != pos.side_to_move {
                continue;
            }
            let from = Square::new(rank, file).expect("in-bounds square");
            for to in piece_targets(pos, piece, from) {
                if let Some(target) = pos.board[to.rank as usize][to.file as usize] {
                    if target.color == piece.color {
                        continue; // 不能吃己方棋子
                    }
                }
                moves.push(Move { from, to });
            }
        }
    }
    moves
}

/// 寻找某方将/帅所在格。
pub fn find_king(pos: &Position, color: Color) -> Option<Square> {
    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            if let Some(p) = pos.board[rank as usize][file as usize] {
                if p.color == color && p.kind == PieceKind::King {
                    return Some(Square::new(rank, file).expect("in-bounds square"));
                }
            }
        }
    }
    None
}

/// 直接判断 `from` 的棋子 `piece` 是否攻击 `sq`（不构造临时 Vec，M3）。
///
/// 与 `piece_targets(pos, piece, from).contains(&sq)` 语义等价（由等价性测试保证）。
pub fn attacks_square(pos: &Position, piece: Piece, from: Square, sq: Square) -> bool {
    if from == sq {
        return false;
    }
    // 己方棋子占据的目标格不算攻击（与伪合法走法一致）
    if let Some(occupant) = pos.board[sq.rank as usize][sq.file as usize] {
        if occupant.color == piece.color {
            return false;
        }
    }
    match piece.kind {
        PieceKind::King => adjacent_orthogonal(from, sq) && in_palace(sq, piece.color),
        PieceKind::Advisor => diagonal_step(from, sq) && in_palace(sq, piece.color),
        PieceKind::Elephant => {
            if !diagonal_2(from, sq) {
                return false;
            }
            let (min_rank, max_rank) = elephant_rank_range(piece.color);
            if sq.rank < min_rank || sq.rank > max_rank {
                return false;
            }
            let eye = midpoint(from, sq);
            pos.board[eye.rank as usize][eye.file as usize].is_none()
        }
        PieceKind::Horse => {
            let (dr, df) = delta(from, sq);
            if !knight_step(dr, df) {
                return false;
            }
            let leg = square_at(from, dr / 2, df / 2).expect("leg is on board");
            pos.board[leg.rank as usize][leg.file as usize].is_none()
        }
        PieceKind::Rook => same_line(from, sq) && line_clear_between(pos, from, sq),
        PieceKind::Cannon => {
            if !same_line(from, sq) {
                return false;
            }
            let count = between_count(pos, from, sq);
            if count == 0 {
                pos.board[sq.rank as usize][sq.file as usize].is_none()
            } else {
                count == 1 && pos.board[sq.rank as usize][sq.file as usize].is_some()
            }
        }
        PieceKind::Pawn => pawn_attacks(from, piece.color, sq),
    }
}

fn same_line(a: Square, b: Square) -> bool {
    a.rank == b.rank || a.file == b.file
}

fn adjacent_orthogonal(a: Square, b: Square) -> bool {
    (a.rank as i8 - b.rank as i8).abs() + (a.file as i8 - b.file as i8).abs() == 1
}

fn diagonal_step(a: Square, b: Square) -> bool {
    (a.rank as i8 - b.rank as i8).abs() == 1 && (a.file as i8 - b.file as i8).abs() == 1
}

fn diagonal_2(a: Square, b: Square) -> bool {
    (a.rank as i8 - b.rank as i8).abs() == 2 && (a.file as i8 - b.file as i8).abs() == 2
}

fn knight_step(dr: i8, df: i8) -> bool {
    (dr.abs() == 2 && df.abs() == 1) || (dr.abs() == 1 && df.abs() == 2)
}

fn delta(from: Square, sq: Square) -> (i8, i8) {
    (
        sq.rank as i8 - from.rank as i8,
        sq.file as i8 - from.file as i8,
    )
}

fn midpoint(a: Square, b: Square) -> Square {
    Square::new((a.rank + b.rank) / 2, (a.file + b.file) / 2).expect("midpoint is on board")
}

/// 同行/同列两个格子之间是否全空（不含两端）。
fn line_clear_between(pos: &Position, a: Square, b: Square) -> bool {
    between_count(pos, a, b) == 0
}

/// 同行/同列两个格子之间被占据的格子数（不含两端）。
fn between_count(pos: &Position, a: Square, b: Square) -> usize {
    if a.rank == b.rank {
        let (lo, hi) = if a.file < b.file {
            (a.file, b.file)
        } else {
            (b.file, a.file)
        };
        (lo + 1..hi)
            .filter(|f| pos.board[a.rank as usize][*f as usize].is_some())
            .count()
    } else if a.file == b.file {
        let (lo, hi) = if a.rank < b.rank {
            (a.rank, b.rank)
        } else {
            (b.rank, a.rank)
        };
        (lo + 1..hi)
            .filter(|r| pos.board[*r as usize][a.file as usize].is_some())
            .count()
    } else {
        0
    }
}

fn pawn_attacks(from: Square, color: Color, sq: Square) -> bool {
    let fwd: i8 = match color {
        Color::Red => 1,
        Color::Black => -1,
    };
    if sq.rank as i8 == from.rank as i8 + fwd && sq.file == from.file {
        return true;
    }
    if crossed_river(from, color)
        && sq.rank == from.rank
        && (sq.file as i8 - from.file as i8).abs() == 1
    {
        return true;
    }
    false
}

/// 判断 `sq` 是否被 `by` 一方攻击。
///
/// 包含「飞将」规则：两将同列且中间无子，视作互相攻击。
pub fn is_attacked(pos: &Position, sq: Square, by: Color) -> bool {
    if let Some(king_sq) = find_king(pos, by) {
        if king_sq.file == sq.file && between_empty(pos, king_sq, sq) {
            return true;
        }
    }
    for rank in 0..NUM_RANKS {
        for file in 0..NUM_FILES {
            let Some(piece) = pos.board[rank as usize][file as usize] else {
                continue;
            };
            if piece.color != by {
                continue;
            }
            let from = Square::new(rank, file).expect("in-bounds square");
            if attacks_square(pos, piece, from, sq) {
                return true;
            }
        }
    }
    false
}

/// 同列两个格子之间是否全空（不含两端）。
fn between_empty(pos: &Position, a: Square, b: Square) -> bool {
    if a.file != b.file || a.rank == b.rank {
        return false;
    }
    let (lo, hi) = if a.rank < b.rank {
        (a.rank, b.rank)
    } else {
        (b.rank, a.rank)
    };
    for rank in (lo + 1)..hi {
        if pos.board[rank as usize][a.file as usize].is_some() {
            return false;
        }
    }
    true
}

/// `color` 是否正被将军。
pub fn is_in_check(pos: &Position, color: Color) -> bool {
    let Some(king_sq) = find_king(pos, color) else {
        return false; // 无王局面不判定将军（由 validate 报告）
    };
    is_attacked(pos, king_sq, color.opponent())
}

/// 假设走法合法，直接执行（不校验）。
pub fn make_unchecked(pos: &Position, mv: Move) -> Position {
    let mut next = pos.clone();
    let piece = next.board[mv.from.rank as usize][mv.from.file as usize]
        .expect("move source must contain a piece");
    let captured = next.board[mv.to.rank as usize][mv.to.file as usize].take();
    next.board[mv.to.rank as usize][mv.to.file as usize] = Some(piece);
    next.board[mv.from.rank as usize][mv.from.file as usize] = None;
    next.side_to_move = next.side_to_move.opponent();
    next.halfmove_clock = if captured.is_some() {
        0
    } else {
        next.halfmove_clock + 1
    };
    if next.side_to_move == Color::Red {
        next.fullmove_number += 1;
    }
    next
}

/// 合法着法（伪合法 + 己方王安全）。
pub fn legal_moves(pos: &Position) -> Vec<Move> {
    pseudo_legal_moves(pos)
        .into_iter()
        .filter(|mv| {
            let next = make_unchecked(pos, *mv);
            !is_in_check(&next, pos.side_to_move)
        })
        .collect()
}

/// 依序执行一串着法（PV 预览用）；任一步非法即返回错误。
pub fn apply_moves(pos: &Position, moves: &[Move]) -> Result<Position, String> {
    let mut cur = pos.clone();
    for mv in moves {
        cur = apply_move(&cur, *mv).ok_or_else(|| format!("非法着法：{}", mv.uci()))?;
    }
    Ok(cur)
}

/// 走棋：仅当合法时返回新局面。
pub fn apply_move(pos: &Position, mv: Move) -> Option<Position> {
    if legal_moves(pos).contains(&mv) {
        Some(make_unchecked(pos, mv))
    } else {
        None
    }
}

/// 是否将死：行棋方被将军且无合法着法。
pub fn is_checkmate(pos: &Position) -> bool {
    is_in_check(pos, pos.side_to_move) && legal_moves(pos).is_empty()
}

/// 是否困毙：行棋方未被将军但无合法着法（中国象棋中判负）。
pub fn is_stalemate(pos: &Position) -> bool {
    !is_in_check(pos, pos.side_to_move) && legal_moves(pos).is_empty()
}

/// perft：固定深度内合法着法总数（叶子计数）。
pub fn perft(pos: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = legal_moves(pos);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut nodes = 0;
    for mv in moves {
        nodes += perft(&make_unchecked(pos, mv), depth - 1);
    }
    nodes
}
