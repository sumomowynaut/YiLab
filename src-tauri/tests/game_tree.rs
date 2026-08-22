//! 棋谱树集成测试：Root/MoveNode/主线/变例/嵌套变例/撤销重做/导航/注释/NAG/恢复局面。

use pikaxiangqi_lib::board::fen::to_fen;
use pikaxiangqi_lib::board::types::{Move, START_FEN};
use pikaxiangqi_lib::game::dto;
use pikaxiangqi_lib::game::nag::Nag;
use pikaxiangqi_lib::game::tree::{GameError, GameTree};

fn tree() -> GameTree {
    GameTree::new(START_FEN).unwrap()
}

fn mv(s: &str) -> Move {
    Move::parse_uci(s).unwrap()
}

fn insert_seq(t: &mut GameTree, seq: &[&str]) -> Vec<u64> {
    seq.iter().map(|s| t.insert_move(mv(s)).unwrap()).collect()
}

// ---------- Root / 基础 ----------

#[test]
fn new_tree_has_only_root() {
    let t = tree();
    assert_eq!(t.current, t.root);
    assert_eq!(t.main_line(), vec![t.root]);
    let root = t.node(t.root).unwrap();
    assert!(root.mv.is_none());
    assert!(root.parent.is_none());
    assert!(root.children.is_empty());
    assert!(!t.undo_available());
    assert!(!t.redo_available());
}

#[test]
fn new_tree_rejects_bad_fen() {
    assert!(GameTree::new("not-a-fen").is_err());
    assert!(GameTree::new("9/9/9/9/9/9/9/9/9 w - - 0 1").is_err());
}

// ---------- InsertMove ----------

#[test]
fn insert_move_creates_child_and_moves_current() {
    let mut t = tree();
    let id = t.insert_move(mv("h2e2")).unwrap();
    assert_eq!(t.current, id);
    let n = t.node(id).unwrap();
    assert_eq!(n.mv, Some(mv("h2e2")));
    assert_eq!(n.parent, Some(t.root));
    assert_eq!(t.node(t.root).unwrap().children, vec![id]);
    assert!(t.undo_available());
}

#[test]
fn insert_illegal_move_errors_and_keeps_current() {
    let mut t = tree();
    assert!(matches!(
        t.insert_move(mv("e2e5")),
        Err(GameError::IllegalMove(_))
    ));
    assert_eq!(t.current, t.root);
}

#[test]
fn insert_same_move_reuses_child() {
    let mut t = tree();
    let a = t.insert_move_at(t.root, mv("h2e2")).unwrap();
    let b = t.insert_move_at(t.root, mv("h2e2")).unwrap();
    assert_eq!(a, b);
    assert_eq!(t.node(t.root).unwrap().children.len(), 1);
}

#[test]
fn insert_move_at_unknown_parent_errors() {
    let mut t = tree();
    assert!(matches!(
        t.insert_move_at(999, mv("h2e2")),
        Err(GameError::NodeNotFound(999))
    ));
}

// ---------- 主线 / 变例 / 嵌套变例 ----------

#[test]
fn main_line_follows_first_children() {
    let mut t = tree();
    let ids = insert_seq(&mut t, &["h2e2", "h7e7"]);
    t.go_to_start().unwrap();
    t.insert_move(mv("b0c2")).unwrap(); // 根节点变例
    assert_eq!(t.main_line(), vec![t.root, ids[0], ids[1]]);
}

#[test]
fn insert_variation_creates_sibling() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    let main = t.current;
    t.go_to_start().unwrap();
    let var = t.insert_move(mv("b0c2")).unwrap();
    let root = t.node(t.root).unwrap();
    assert_eq!(root.children, vec![main, var]);
    assert!(!t.is_variation(main));
    assert!(t.is_variation(var));
}

#[test]
fn nested_variations() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    let main2 = t.insert_move(mv("h7e7")).unwrap();
    t.set_current(n1).unwrap();
    let var = t.insert_move(mv("b9c7")).unwrap(); // 在 n1 下的变例（轮到黑方）
    t.insert_move(mv("h0g2")).unwrap(); // 变例内部继续（红方）
    assert_eq!(t.node(n1).unwrap().children, vec![main2, var]);
    assert!(t.is_variation(var));
    // 变例内部的子节点不是「变例起点」
    assert!(!t.is_variation(t.current));
}

#[test]
fn restore_position_matches_cached_fen() {
    let mut t = tree();
    let seq = ["h2e2", "h7e7", "h0g2", "b9c7", "b0c2", "c7e8"];
    let ids = insert_seq(&mut t, &seq);
    for id in ids {
        let pos = t.restore_position(id).unwrap();
        assert_eq!(
            to_fen(&pos),
            t.node(id).unwrap().fen,
            "节点 {id} 局面应一致"
        );
    }
    // 根节点恢复为起始局面
    assert_eq!(to_fen(&t.restore_position(t.root).unwrap()), START_FEN);
}

#[test]
fn restore_position_from_any_branch() {
    let mut t = tree();
    insert_seq(&mut t, &["h2e2", "h7e7"]);
    t.go_to_start().unwrap();
    let var = t.insert_move(mv("b0c2")).unwrap();
    let pos = t.restore_position(var).unwrap();
    // 变例第一手后：红马到 c2
    assert!(pos.board[2][2].is_some());
}

// ---------- Undo / Redo ----------

#[test]
fn undo_redo_steps() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    let n2 = t.insert_move(mv("h7e7")).unwrap();
    t.undo().unwrap();
    assert_eq!(t.current, n1);
    t.undo().unwrap();
    assert_eq!(t.current, t.root);
    assert!(matches!(t.undo(), Err(GameError::NoParent)));
    t.redo().unwrap();
    assert_eq!(t.current, n1);
    t.redo().unwrap();
    assert_eq!(t.current, n2);
    assert!(matches!(t.redo(), Err(GameError::NothingToRedo)));
}

#[test]
fn navigation_clears_redo_stack() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    t.insert_move(mv("h7e7")).unwrap();
    t.undo().unwrap();
    assert!(t.redo_available());
    t.go_to_start().unwrap();
    assert!(!t.redo_available());
}

#[test]
fn insert_clears_redo_stack() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    t.insert_move(mv("h7e7")).unwrap();
    t.undo().unwrap();
    assert!(t.redo_available());
    t.insert_move(mv("h7h8")).unwrap(); // 从 n1 走新变例
    assert!(!t.redo_available());
}

// ---------- Navigate / Previous / Next ----------

#[test]
fn navigate_jumps_and_validates() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    t.set_current(t.root).unwrap();
    assert_eq!(t.current, t.root);
    t.set_current(n1).unwrap();
    assert_eq!(t.current, n1);
    assert!(t.set_current(999).is_err());
}

#[test]
fn previous_next_along_main_line() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    let n2 = t.insert_move(mv("h7e7")).unwrap();
    t.previous().unwrap();
    assert_eq!(t.current, n1);
    t.previous().unwrap();
    assert_eq!(t.current, t.root);
    assert!(matches!(t.previous(), Err(GameError::NoParent)));
    t.next_move().unwrap();
    assert_eq!(t.current, n1);
    t.next_move().unwrap();
    assert_eq!(t.current, n2);
    assert!(matches!(t.next_move(), Err(GameError::NoNext)));
}

#[test]
fn go_to_start_and_end() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    let n2 = t.insert_move(mv("h7e7")).unwrap();
    t.go_to_start().unwrap();
    assert_eq!(t.current, t.root);
    t.go_to_end().unwrap();
    assert_eq!(t.current, n2);
    assert!(t.next_move().is_err());
    let _ = (n1, n2);
}

// ---------- DeleteVariation ----------

#[test]
fn delete_variation_removes_subtree() {
    let mut t = tree();
    let main = t.insert_move(mv("h2e2")).unwrap();
    t.go_to_start().unwrap();
    let var = t.insert_move(mv("b0c2")).unwrap();
    t.insert_move(mv("b9c7")).unwrap(); // 变例内更深节点
    t.delete_variation(var).unwrap();
    assert_eq!(t.node(t.root).unwrap().children, vec![main]);
    assert!(t.node(var).is_err());
    // 当前节点在被删子树内 → 回退到父节点
    assert_eq!(t.current, t.root);
}

#[test]
fn delete_variation_rejects_root_and_main_line() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    assert!(matches!(
        t.delete_variation(t.root),
        Err(GameError::CannotDeleteRoot)
    ));
    assert!(matches!(
        t.delete_variation(n1),
        Err(GameError::NotAVariation(_))
    ));
}

#[test]
fn delete_variation_keeps_current_when_outside() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    let n1 = t.current;
    t.go_to_start().unwrap();
    let var = t.insert_move(mv("b0c2")).unwrap();
    t.set_current(n1).unwrap();
    t.delete_variation(var).unwrap();
    assert_eq!(t.current, n1);
}

// ---------- Comments / Annotations ----------

#[test]
fn comments_are_stored_and_editable() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    t.set_comment("中炮开局".to_string()).unwrap();
    assert_eq!(t.node(t.current).unwrap().comment, "中炮开局");
    t.set_comment("修改后".to_string()).unwrap();
    assert_eq!(t.node(t.current).unwrap().comment, "修改后");
    t.set_comment_at(t.root, "根注释".to_string()).unwrap();
    assert_eq!(t.node(t.root).unwrap().comment, "根注释");
}

#[test]
fn nags_are_added_removed_and_deduplicated() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    t.add_nag(Nag::Good).unwrap();
    t.add_nag(Nag::Good).unwrap();
    t.add_nag(Nag::Interesting).unwrap();
    let n = t.node(t.current).unwrap();
    assert_eq!(n.nags, vec![Nag::Good, Nag::Interesting]);
    t.remove_nag(Nag::Good).unwrap();
    assert_eq!(t.node(t.current).unwrap().nags, vec![Nag::Interesting]);
    t.set_nag(Nag::Interesting, false).unwrap();
    assert!(t.node(t.current).unwrap().nags.is_empty());
    t.set_nag(Nag::Brilliant, true).unwrap();
    assert_eq!(t.node(t.current).unwrap().nags, vec![Nag::Brilliant]);
}

// ---------- Snapshot DTO ----------

#[test]
fn snapshot_contains_tree_and_current_state() {
    let mut t = tree();
    let ids = insert_seq(&mut t, &["h2e2", "h7e7"]);
    t.set_current(ids[0]).unwrap();
    t.set_comment("测试注释".to_string()).unwrap();
    t.add_nag(Nag::Interesting).unwrap();
    let snap = dto::snapshot(&t).unwrap();
    assert_eq!(snap.current_id, ids[0]);
    assert_eq!(snap.comment, "测试注释");
    assert_eq!(snap.nags, vec!["!?"]);
    assert_eq!(snap.previous_id, Some(t.root));
    assert_eq!(snap.next_main_id, Some(ids[1]));
    assert!(snap.undo_available);
    assert!(!snap.redo_available);
    // 树结构序列化
    let json = serde_json::to_string(&snap).unwrap();
    assert!(json.contains("currentId"));
    assert!(json.contains("h2e2"));
    assert!(json.contains("!?"));
    assert!(json.contains("测试注释"));
    // 根节点 moveNumber 为 0，红方第一手为 1.
    assert_eq!(snap.tree.children[0].move_number, 1);
    assert!(snap.tree.children[0].is_red);
}

#[test]
fn snapshot_black_move_number_is_round_one() {
    let mut t = tree();
    insert_seq(&mut t, &["h2e2", "h7e7"]);
    let snap = dto::snapshot(&t).unwrap();
    let black = &snap.tree.children[0].children[0];
    assert_eq!(black.move_number, 1);
    assert!(!black.is_red);
}
