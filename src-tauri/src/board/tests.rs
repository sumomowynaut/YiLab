//! 棋盘核心单元测试：覆盖将军/应对/将死/吃子/特殊规则/各类棋子/河界/九宫等。

use super::fen::{parse_fen, to_fen};
use super::rules::{
    apply_move, apply_moves, attacks_square, is_checkmate, is_in_check, is_stalemate, legal_moves,
    make_unchecked, perft, piece_targets,
};
use super::transform::{mirrored, rotated_180};
use super::types::{Color, Move, PieceKind, Position, Square, START_FEN};
use super::validate::validate_position;

fn pos(fen: &str) -> Position {
    parse_fen(fen).expect("FEN 应可解析")
}

fn legal_uci(fen: &str) -> Vec<String> {
    let mut v: Vec<String> = legal_moves(&pos(fen)).iter().map(|m| m.uci()).collect();
    v.sort();
    v
}

fn targets_uci(p: &Position, from: &str) -> Vec<String> {
    let from_sq = Square::parse_uci(from).expect("from square");
    let mut v: Vec<String> = legal_moves(p)
        .iter()
        .filter(|m| m.from == from_sq)
        .map(|m| m.uci())
        .collect();
    v.sort();
    v
}

fn assert_set_eq(actual: Vec<String>, expected: &[&str]) {
    let mut exp: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    exp.sort();
    assert_eq!(actual, exp, "着法集合不一致");
}

fn make(fen: &str, mv: &str) -> Position {
    let p = pos(fen);
    let m = Move::parse_uci(mv).expect("move");
    apply_move(&p, m).unwrap_or_else(|| panic!("着法 {mv} 应为合法"))
}

// ---------- 将军 ----------

#[test]
fn rook_gives_check_along_rank() {
    let p = pos("R3k4/9/9/9/9/9/9/9/9/3K5 b - - 0 1");
    assert!(is_in_check(&p, Color::Black), "黑方应被红车将军");
    assert!(!is_in_check(&p, Color::Red), "红方不应被将军");
}

#[test]
fn cannon_gives_check_with_screen() {
    let p = pos("2Cak4/9/9/9/9/9/9/9/9/3K5 b - - 0 1");
    assert!(is_in_check(&p, Color::Black), "黑方应被红炮隔子将军");
}

#[test]
fn horse_gives_check() {
    let p = pos("4k4/2N6/9/9/9/9/9/9/9/3K5 b - - 0 1");
    assert!(is_in_check(&p, Color::Black), "黑方应被红马将军");
}

#[test]
fn flying_general_is_check() {
    let p = pos("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1");
    assert!(is_in_check(&p, Color::Red), "将帅照面，红方应被将军");
    assert!(is_in_check(&p, Color::Black), "将帅照面，黑方也应被将军");
}

// ---------- 将军应对 ----------

#[test]
fn must_escape_rook_check() {
    assert_set_eq(legal_uci("R3k4/9/9/9/9/9/9/9/9/3K5 b - - 0 1"), &["e9e8"]);
}

#[test]
fn must_respond_to_cannon_check() {
    assert_set_eq(
        legal_uci("2Cak4/9/9/9/9/9/9/9/9/3K5 b - - 0 1"),
        &["d9e8", "e9e8"],
    );
}

#[test]
fn must_respond_to_horse_check() {
    assert_set_eq(
        legal_uci("4k4/2N6/9/9/9/9/9/9/9/3K5 b - - 0 1"),
        &["e9e8", "e9f9"],
    );
}

#[test]
fn blocked_check_requires_blocking_or_escape() {
    // 红车 c5 与黑将 e9 之间隔着黑车 e5，黑方未被将军
    let p = pos("4k4/9/9/9/2R1r4/9/9/9/9/3K5 b - - 0 1");
    assert!(!is_in_check(&p, Color::Black));
    assert!(!legal_moves(&p).is_empty());
}

// ---------- 将死 ----------

#[test]
fn double_rook_checkmate() {
    let p = pos("R3k4/R8/9/9/9/9/9/9/9/3K5 b - - 0 1");
    assert!(is_in_check(&p, Color::Black));
    assert!(is_checkmate(&p), "黑方应被将死");
    assert!(legal_moves(&p).is_empty());
}

#[test]
fn stalemate_is_loss_but_not_checkmate() {
    let p = pos("4k4/3R1R3/9/9/9/9/9/9/9/3K5 b - - 0 1");
    assert!(!is_in_check(&p, Color::Black));
    assert!(!is_checkmate(&p));
    assert!(is_stalemate(&p), "黑方应困毙（无子可走且未被将军）");
    assert!(legal_moves(&p).is_empty());
}

// ---------- 吃子 ----------

#[test]
fn rook_captures_and_resets_halfmove_clock() {
    let p = pos("4k4/9/9/9/9/9/9/9/4pR3/3K5 w - - 5 41");
    let next = apply_move(&p, Move::parse_uci("f1e1").unwrap()).expect("吃子应合法");
    assert_eq!(
        next.board[1][4],
        Some(super::types::Piece {
            color: Color::Red,
            kind: PieceKind::Rook
        })
    );
    assert_eq!(next.board[1][5], None);
    assert_eq!(next.side_to_move, Color::Black);
    assert_eq!(next.halfmove_clock, 0, "吃子后半步钟归零");
    assert_eq!(next.fullmove_number, 41);
}

#[test]
fn quiet_move_increments_halfmove_clock() {
    let p = pos("4k4/9/9/9/9/9/9/9/4pR3/3K5 w - - 5 41");
    let next = apply_move(&p, Move::parse_uci("f1f0").unwrap()).expect("走棋应合法");
    assert_eq!(next.halfmove_clock, 6);
}

// ---------- 升变 / 特殊规则 ----------

#[test]
fn no_promotion_pawn_on_last_rank_stays_pawn() {
    // 红兵到达底线 e9：只能横走，且不升变
    let p = pos("k3P4/9/9/9/9/9/9/9/9/3K5 w - - 0 1");
    assert_set_eq(targets_uci(&p, "e9"), &["e9d9", "e9f9"]);
    let next = make("k3P4/9/9/9/9/9/9/9/9/3K5 w - - 0 1", "e9d9");
    let piece = next.board[9][3].expect("有棋子");
    assert_eq!(piece.kind, PieceKind::Pawn, "中国象棋无升变");
    assert_eq!(piece.color, Color::Red);
}

#[test]
fn flying_general_blocks_king_move() {
    // 红帅 e0 若移到 e1 仍将帅照面，非法
    assert_set_eq(
        legal_uci("4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1"),
        &["e0d0", "e0f0"],
    );
}

// ---------- 炮 ----------

#[test]
fn cannon_moves_like_rook_without_capturing() {
    let p = pos("4k4/9/9/9/4C4/4p4/9/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "e5");
    assert_eq!(ts.len(), 11);
    assert!(ts.contains(&"e5e6".to_string()));
    assert!(!ts.contains(&"e5e4".to_string()), "无炮架时炮不能吃子");
}

#[test]
fn cannon_captures_over_exactly_one_screen() {
    let p = pos("4k4/9/9/9/4C4/4r4/4p4/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "e5");
    assert!(ts.contains(&"e5e3".to_string()), "隔一个炮架应能吃 e3");
    assert!(!ts.contains(&"e5e4".to_string()), "炮架本身不能吃");
}

#[test]
fn cannon_cannot_capture_second_piece_after_screen() {
    let p = pos("4k4/9/9/9/4C4/4r4/4p4/4p4/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "e5");
    assert!(ts.contains(&"e5e3".to_string()));
    assert!(
        !ts.contains(&"e5e2".to_string()),
        "炮不能越过第二个棋子吃子"
    );
}

#[test]
fn cannon_jumps_over_own_piece_as_screen() {
    // 己方棋子也可作炮架
    let p = pos("4k4/9/9/9/4C4/4R4/4p4/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "e5");
    assert!(
        ts.contains(&"e5e3".to_string()),
        "己方红车作炮架应能隔子吃 e3"
    );
}

// ---------- 马腿 ----------

#[test]
fn horse_blocked_by_leg() {
    // 红马 d5，黑卒 d6 蹩马腿：不能走 e7/c7
    let p = pos("4k4/9/9/3p5/3N5/9/9/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "d5");
    assert!(!ts.contains(&"d5e7".to_string()), "蹩马腿不能走 e7");
    assert!(!ts.contains(&"d5c7".to_string()), "蹩马腿不能走 c7");
    assert!(ts.contains(&"d5f6".to_string()), "未蹩腿方向可走");
}

#[test]
fn horse_moves_normally_when_leg_empty() {
    let p = pos("4k4/9/9/9/3N5/9/9/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "d5");
    assert!(ts.contains(&"d5e7".to_string()));
    assert_eq!(ts.len(), 8, "无阻挡时马应有 8 个目标");
}

// ---------- 象眼 ----------

#[test]
fn elephant_blocked_by_eye() {
    // 红相 c2，黑士 d3 塞象眼：不能走 e4，可走 a4
    let p = pos("4k4/9/9/9/9/9/3a5/2B6/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "c2");
    assert!(ts.contains(&"c2a4".to_string()));
    assert!(!ts.contains(&"c2e4".to_string()), "塞象眼不能走 e4");
}

#[test]
fn elephant_cannot_cross_river() {
    // 红相在 c4（红方底线一侧最后一排），2 步斜走会过河 → 无目标
    let p = pos("4k4/9/9/9/9/2B6/9/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "c4");
    assert!(!ts.contains(&"c4a6".to_string()), "相不可过河");
    assert!(!ts.contains(&"c4e6".to_string()), "相不可过河");
}

// ---------- 士 ----------

#[test]
fn advisor_stays_in_palace() {
    // 黑士 d9 只能到 e8（c8 出九宫）
    assert_set_eq(
        targets_uci(&pos("3ak4/9/9/9/9/9/9/9/9/3K5 b - - 0 1"), "d9"),
        &["d9e8"],
    );
}

// ---------- 将 ----------

#[test]
fn king_stays_in_palace() {
    // 黑将 d9：可走 d8/e9，不可走 c9（出九宫）
    assert_set_eq(
        targets_uci(&pos("3k5/9/9/9/9/9/9/9/9/K8 b - - 0 1"), "d9"),
        &["d9d8", "d9e9"],
    );
}

#[test]
fn king_cannot_move_into_check() {
    // 黑车 i0 沿第 0 行将军：红帅 e0 只能上移 e1，d0/f0 仍被攻击（不能走进被将军的格）
    assert_set_eq(
        targets_uci(&pos("k8/9/9/9/9/9/9/9/9/4K3r w - - 0 1"), "e0"),
        &["e0e1"],
    );
}

// ---------- 车 ----------

#[test]
fn rook_slides_and_captures_first() {
    let p = pos("4k4/9/9/9/2R1r4/9/9/9/9/3K5 w - - 0 1");
    let ts = targets_uci(&p, "c5");
    assert!(ts.contains(&"c5e5".to_string()), "车可吃 e5 黑车");
    assert!(ts.contains(&"c5d5".to_string()));
    assert!(!ts.contains(&"c5f5".to_string()), "车不能越过 e5");
    assert_eq!(ts.len(), 13);
}

// ---------- 兵/卒、河界 ----------

#[test]
fn red_pawn_before_river_moves_forward_only() {
    assert_set_eq(
        targets_uci(&pos("4k4/9/9/9/9/4P4/9/9/9/3K5 w - - 0 1"), "e4"),
        &["e4e5"],
    );
}

#[test]
fn red_pawn_after_river_can_move_sideways() {
    assert_set_eq(
        targets_uci(&pos("4k4/9/9/9/4P4/9/9/9/9/3K5 w - - 0 1"), "e5"),
        &["e5d5", "e5e6", "e5f5"],
    );
}

#[test]
fn black_pawn_before_river_moves_forward_only() {
    assert_set_eq(
        targets_uci(&pos("4k4/9/9/9/4p4/9/9/9/9/3K5 b - - 0 1"), "e5"),
        &["e5e4"],
    );
}

#[test]
fn black_pawn_after_river_can_move_sideways() {
    assert_set_eq(
        targets_uci(&pos("4k4/9/9/9/9/4p4/9/9/9/3K5 b - - 0 1"), "e4"),
        &["e4d4", "e4e3", "e4f4"],
    );
}

#[test]
fn pawn_never_moves_backward() {
    let red = targets_uci(&pos("4k4/9/9/9/4P4/9/9/9/9/3K5 w - - 0 1"), "e5");
    assert!(!red.contains(&"e5e4".to_string()), "红兵不可后退");
    let black = targets_uci(&pos("4k4/9/9/9/9/4p4/9/9/9/3K5 b - - 0 1"), "e4");
    assert!(!black.contains(&"e4e5".to_string()), "黑卒不可后退");
}

// ---------- FEN ----------

#[test]
fn start_fen_round_trip() {
    let p = pos(START_FEN);
    assert_eq!(to_fen(&p), START_FEN);
}

#[test]
fn custom_fen_round_trip() {
    let fen = "3k1a3/4a4/5n3/9/9/9/9/9/9/4KR3 w - - 0 1";
    assert_eq!(to_fen(&pos(fen)), fen);
}

#[test]
fn fen_parse_rejects_bad_rows() {
    assert!(
        parse_fen("9/9/9/9/9/9/9/9/9 w - - 0 1").is_err(),
        "行数不足"
    );
    assert!(
        parse_fen("9/9/9/9/9/9/9/9/9/8 w - - 0 1").is_err(),
        "行合计不为 9"
    );
    assert!(
        parse_fen("9/9/9/9/9/9/9/9/9/x w - - 0 1").is_err(),
        "非法字符"
    );
    assert!(
        parse_fen("9/9/9/9/9/9/9/9/9/9 x - - 0 1").is_err(),
        "非法走子方"
    );
    assert!(
        parse_fen("9/9/9/9/9/9/9/9/9/09 w - - 0 1").is_err(),
        "不允许数字 0"
    );
    assert!(
        parse_fen("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w x - 0 1").is_err(),
        "第 3 字段必须为 -"
    );
}

// ---------- 局面校验 ----------

#[test]
fn startpos_is_valid() {
    let r = validate_position(&pos(START_FEN));
    assert!(r.ok, "起始局面应合法：{:?}", r.issues);
}

#[test]
fn validate_missing_king() {
    let mut p = pos(START_FEN);
    p.board[9][4] = None; // 移除黑将
    let r = validate_position(&p);
    assert!(!r.ok);
    assert!(r.issues.iter().any(|i| i.contains("缺少")));
}

#[test]
fn validate_multiple_kings() {
    let mut p = pos(START_FEN);
    p.board[6][0] = Some(super::types::Piece {
        color: Color::Black,
        kind: PieceKind::King,
    });
    let r = validate_position(&p);
    assert!(r.issues.iter().any(|i| i.contains("多个")));
}

#[test]
fn validate_advisor_outside_palace() {
    let mut p = pos(START_FEN);
    p.board[6][3] = Some(super::types::Piece {
        color: Color::Black,
        kind: PieceKind::Advisor,
    });
    let r = validate_position(&p);
    assert!(r.issues.iter().any(|i| i.contains("九宫")));
}

#[test]
fn validate_pawn_in_impossible_rank() {
    let mut p = pos(START_FEN);
    p.board[2][0] = Some(super::types::Piece {
        color: Color::Red,
        kind: PieceKind::Pawn,
    });
    let r = validate_position(&p);
    assert!(r.issues.iter().any(|i| i.contains("兵/卒")));
}

#[test]
fn validate_elephant_crossed_river() {
    let mut p = pos(START_FEN);
    p.board[4][2] = Some(super::types::Piece {
        color: Color::Black,
        kind: PieceKind::Elephant,
    });
    let r = validate_position(&p);
    assert!(r.issues.iter().any(|i| i.contains("河界")));
}

#[test]
fn validate_non_moving_side_in_check() {
    // 红先行，但黑将被红车将军 → 非法（上一手方被将军）
    let r = validate_position(&pos("R3k4/9/9/9/9/9/9/9/9/3K5 w - - 0 1"));
    assert!(!r.ok);
    assert!(r.issues.iter().any(|i| i.contains("非行棋方")));
}

#[test]
fn validate_side_to_move_in_check_is_ok() {
    // 黑先行且被将军是合法局面（轮到应将）
    let r = validate_position(&pos("R3k4/9/9/9/9/9/9/9/9/3K5 b - - 0 1"));
    assert!(r.ok, "轮到被将军一方行棋是合法的：{:?}", r.issues);
}

// ---------- 旋转与镜像 ----------

#[test]
fn rotated_180_swaps_sides() {
    let p = pos(START_FEN);
    let r = rotated_180(&p);
    assert_eq!(
        to_fen(&r),
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 1"
    );
}

#[test]
fn mirrored_startpos_is_identical() {
    assert_eq!(to_fen(&mirrored(&pos(START_FEN))), START_FEN);
}

#[test]
fn double_rotation_is_identity() {
    let p = pos(START_FEN);
    assert_eq!(to_fen(&rotated_180(&rotated_180(&p))), START_FEN);
}

// ---------- 坐标 ----------

#[test]
fn square_coordinate_round_trip() {
    for rank in 0..10u8 {
        for file in 0..9u8 {
            let sq = Square::new(rank, file).unwrap();
            assert_eq!(Square::parse_uci(&sq.uci()), Some(sq));
        }
    }
}

#[test]
fn square_parse_rejects_invalid() {
    assert_eq!(Square::parse_uci("j0"), None);
    assert_eq!(Square::parse_uci("a10"), None);
    assert_eq!(Square::parse_uci("aa"), None);
    assert_eq!(Square::parse_uci(""), None);
}

#[test]
fn move_coordinate_round_trip() {
    let m = Move::parse_uci("h2e2").unwrap();
    assert_eq!(m.uci(), "h2e2");
    assert_eq!(Move::parse_uci("g0f0").unwrap().uci(), "g0f0");
    assert!(Move::parse_uci("h2").is_none());
}

// ---------- perft ----------

#[test]
fn perft_startpos_matches_reference() {
    // 参考值来源：Chess Programming Wiki — Chinese Chess Perft Results
    assert_eq!(perft(&pos(START_FEN), 1), 44);
    assert_eq!(perft(&pos(START_FEN), 2), 1_920);
    assert_eq!(perft(&pos(START_FEN), 3), 79_666);
}

// ---------- 基础走子冒烟 ----------

#[test]
fn startpos_red_moves_and_make_round_trip() {
    let p = pos(START_FEN);
    let mv = Move::parse_uci("h2e2").unwrap(); // 炮二平五
    assert!(legal_moves(&p).contains(&mv));
    let next = make_unchecked(&p, mv);
    assert_eq!(next.side_to_move, Color::Black);
    assert_eq!(next.halfmove_clock, 1);
    // 再走一步黑方后，红方重新行棋，回合数 +1
    let mv2 = Move::parse_uci("h7e7").unwrap(); // 炮8平5
    assert!(legal_moves(&next).contains(&mv2));
    let next2 = make_unchecked(&next, mv2);
    assert_eq!(next2.side_to_move, Color::Red);
    assert_eq!(next2.fullmove_number, 2);
}
// ---------- attacks_square 与 piece_targets 等价（M3） ----------

#[test]
fn attacks_square_matches_piece_targets() {
    // M3：attacks_square 不得与既有走法生成逻辑产生分歧
    let fens = [
        START_FEN,
        "r3k4/9/9/9/9/9/9/9/9/3K5 b - - 0 1",
        "4k4/3R1R3/9/9/9/9/9/9/9/3K5 b - - 0 1",
        "4k4/9/9/9/4C4/4r4/4p4/4p4/9/3K5 w - - 0 1",
        "4k4/9/9/3p5/3N5/9/9/9/9/3K5 w - - 0 1",
        "4k4/9/9/9/9/9/3a5/2B6/9/3K5 w - - 0 1",
    ];
    for fen in fens {
        let p = pos(fen);
        // 与「走法目标（含己方过滤）」逐起点对比；攻击判断与行棋方无关
        for rank in 0..10u8 {
            for file in 0..9u8 {
                let Some(piece) = p.board[rank as usize][file as usize] else {
                    continue;
                };
                let from = Square::new(rank, file).unwrap();
                let reachable: std::collections::HashSet<Square> = piece_targets(&p, piece, from)
                    .into_iter()
                    .filter(|sq| {
                        p.board[sq.rank as usize][sq.file as usize]
                            .is_none_or(|o| o.color != piece.color)
                    })
                    .collect();
                for t_rank in 0..10u8 {
                    for t_file in 0..9u8 {
                        let sq = Square::new(t_rank, t_file).unwrap();
                        assert_eq!(
                            attacks_square(&p, piece, from, sq),
                            reachable.contains(&sq),
                            "{} {}->{} 不一致",
                            fen,
                            from.uci(),
                            sq.uci()
                        );
                    }
                }
            }
        }
    }
}

// ---------- apply_moves（PV 预览） ----------

#[test]
fn apply_moves_applies_sequence() {
    let p = pos(START_FEN);
    let seq = ["h2e2", "h7e7", "h0g2", "b9c7"];
    let moves: Vec<Move> = seq.iter().map(|s| Move::parse_uci(s).unwrap()).collect();
    let end = apply_moves(&p, &moves).expect("apply");
    let mut cur = p.clone();
    for s in seq {
        cur = make_unchecked(&cur, Move::parse_uci(s).unwrap());
    }
    assert_eq!(to_fen(&end), to_fen(&cur));
}

#[test]
fn apply_moves_rejects_illegal_step() {
    let p = pos(START_FEN);
    let moves = vec![Move::parse_uci("e2e5").unwrap()]; // e2 无子
    assert!(apply_moves(&p, &moves).is_err());
}
