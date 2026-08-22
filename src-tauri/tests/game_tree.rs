//! 棋谱树集成测试：Root/MoveNode/主线/变例/嵌套变例/撤销重做/导航/注释/NAG/恢复局面/序列化/变例提升与排序。

use pikaxiangqi_lib::board::fen::{parse_fen, to_fen};
use pikaxiangqi_lib::board::types::{Move, START_FEN};
use pikaxiangqi_lib::game::dto;
use pikaxiangqi_lib::game::nag::Nag;
use pikaxiangqi_lib::game::serialize;
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

// ---------- 恢复局面 / 缓存元数据（H3） ----------

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
fn cached_metadata_matches_derived_position() {
    // H3：节点缓存 side_to_move / fullmove_number 必须与局面一致，快照无需逐节点 parse_fen
    let mut t = tree();
    let seq = ["h2e2", "h7e7", "h0g2", "b9c7", "b0c2", "c7e8"];
    let ids = insert_seq(&mut t, &seq);
    for id in ids {
        let n = t.node(id).unwrap();
        let p = parse_fen(&n.fen).unwrap();
        assert_eq!(n.side_to_move, p.side_to_move, "节点 {id} 走子方缓存不一致");
        assert_eq!(
            n.fullmove_number, p.fullmove_number,
            "节点 {id} 回合数缓存不一致"
        );
    }
    let root = t.node(t.root).unwrap();
    let rp = parse_fen(&root.fen).unwrap();
    assert_eq!(root.side_to_move, rp.side_to_move);
    assert_eq!(root.fullmove_number, rp.fullmove_number);
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

// ---------- DeleteVariation / Promote / Reorder（M2） ----------

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

#[test]
fn promote_variation_moves_to_main_line() {
    let mut t = tree();
    let m = t.insert_move(mv("h2e2")).unwrap();
    t.go_to_start().unwrap();
    let v1 = t.insert_move(mv("b0c2")).unwrap();
    t.go_to_start().unwrap();
    let v2 = t.insert_move(mv("a0a1")).unwrap(); // 车一进一
    let root = t.root;
    assert_eq!(t.node(root).unwrap().children, vec![m, v1, v2]);
    // 提升 v1 为主线
    t.promote_variation(v1).unwrap();
    assert_eq!(t.node(root).unwrap().children, vec![v1, m, v2]);
    assert!(!t.is_variation(v1));
    assert!(t.is_variation(m));
    // 提升主线本身应报错
    assert!(matches!(
        t.promote_variation(v1),
        Err(GameError::NotAVariation(_))
    ));
}

#[test]
fn reorder_variation_moves_up_and_down() {
    let mut t = tree();
    let m = t.insert_move(mv("h2e2")).unwrap();
    t.go_to_start().unwrap();
    let v1 = t.insert_move(mv("b0c2")).unwrap();
    t.go_to_start().unwrap();
    let v2 = t.insert_move(mv("a0a1")).unwrap();
    t.go_to_start().unwrap();
    let v3 = t.insert_move(mv("i0i1")).unwrap(); // 车九进一（合法）
    let root = t.root;
    assert_eq!(t.node(root).unwrap().children, vec![m, v1, v2, v3]);
    // v2（index 2）上移 → index 1
    t.reorder_variation(root, 2, 1).unwrap();
    assert_eq!(t.node(root).unwrap().children, vec![m, v2, v1, v3]);
    // v1（index 2）下移 → index 3
    t.reorder_variation(root, 2, 3).unwrap();
    assert_eq!(t.node(root).unwrap().children, vec![m, v2, v3, v1]);
    // 越界报错
    assert!(matches!(
        t.reorder_variation(root, 1, 9),
        Err(GameError::InvalidIndex { .. })
    ));
    // 不允许移动主线（index 0）
    assert!(matches!(
        t.reorder_variation(root, 0, 1),
        Err(GameError::InvalidIndex { .. })
    ));
}

// ---------- Comments / Annotations（H1：按节点定位） ----------

#[test]
fn comments_are_stored_and_editable_by_node() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    t.set_comment_at(n1, "中炮开局".to_string()).unwrap();
    assert_eq!(t.node(n1).unwrap().comment, "中炮开局");
    t.set_comment_at(n1, "修改后".to_string()).unwrap();
    assert_eq!(t.node(n1).unwrap().comment, "修改后");
    t.set_comment_at(t.root, "根注释".to_string()).unwrap();
    assert_eq!(t.node(t.root).unwrap().comment, "根注释");
}

#[test]
fn nags_are_added_removed_and_deduplicated_by_node() {
    let mut t = tree();
    let n1 = t.insert_move(mv("h2e2")).unwrap();
    t.set_nag_at(n1, Nag::Good, true).unwrap();
    t.set_nag_at(n1, Nag::Good, true).unwrap();
    t.set_nag_at(n1, Nag::Interesting, true).unwrap();
    assert_eq!(t.node(n1).unwrap().nags, vec![Nag::Good, Nag::Interesting]);
    t.set_nag_at(n1, Nag::Good, false).unwrap();
    assert_eq!(t.node(n1).unwrap().nags, vec![Nag::Interesting]);
    t.set_nag_at(n1, Nag::Interesting, false).unwrap();
    assert!(t.node(n1).unwrap().nags.is_empty());
    t.set_nag_at(n1, Nag::Brilliant, true).unwrap();
    assert_eq!(t.node(n1).unwrap().nags, vec![Nag::Brilliant]);
}

#[test]
fn comment_written_to_explicit_node_survives_navigation() {
    // H1 回归：节点 A 输入注释 → 导航到节点 B → A 的注释仍写入 A，B 不会错误获得
    let mut t = tree();
    let a = t.insert_move(mv("h2e2")).unwrap();
    let b = t.insert_move(mv("h7e7")).unwrap();
    // 导航到 B 之后再按节点 A 写注释（模拟 setComment 与 navigate 乱序）
    t.set_current(b).unwrap();
    t.set_comment_at(a, "A 的注释".to_string()).unwrap();
    assert_eq!(t.node(a).unwrap().comment, "A 的注释");
    assert_eq!(t.node(b).unwrap().comment, "");
    // NAG 同理
    t.set_nag_at(a, Nag::Good, true).unwrap();
    assert_eq!(t.node(a).unwrap().nags, vec![Nag::Good]);
    assert!(t.node(b).unwrap().nags.is_empty());
}

// ---------- Snapshot DTO ----------

#[test]
fn snapshot_contains_tree_and_current_state() {
    let mut t = tree();
    let ids = insert_seq(&mut t, &["h2e2", "h7e7"]);
    t.set_current(ids[0]).unwrap();
    t.set_comment_at(ids[0], "测试注释".to_string()).unwrap();
    t.set_nag_at(ids[0], Nag::Interesting, true).unwrap();
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

// ---------- 文档序列化（H2） ----------

#[test]
fn tree_json_round_trip_preserves_document_not_session() {
    let mut t = tree();
    let a = t.insert_move(mv("h2e2")).unwrap();
    t.set_comment_at(a, "中炮".to_string()).unwrap();
    t.set_nag_at(a, Nag::Good, true).unwrap();
    let b = t.insert_move(mv("h7e7")).unwrap();
    // 根下加一支变例
    t.go_to_start().unwrap();
    let v = t.insert_move(mv("b0c2")).unwrap();
    // 让会话状态非默认：current 回到 a，redo_stack 非空
    t.set_current(a).unwrap();
    t.undo().unwrap(); // current = root, redo_stack = [a]
    assert!(t.redo_available());
    assert_ne!(t.current, b);

    let json = serialize::to_tree_json(&t).unwrap();
    // 文档 JSON 不得包含会话状态字段
    assert!(!json.contains("current"));
    assert!(!json.contains("redo"));
    assert!(!json.contains("redoStack"));

    // 重新导入：结构一致，但会话状态重置
    let t2 = serialize::from_tree_json(&json).unwrap();
    assert_eq!(t2.root, t.root);
    assert_eq!(t2.startpos, t.startpos);
    // 主线/变例结构与注释一致
    assert_eq!(t2.main_line(), vec![t2.root, a, b]);
    let root_children = t2.node(t2.root).unwrap().children.clone();
    assert_eq!(root_children, vec![a, v]);
    assert_eq!(t2.node(a).unwrap().comment, "中炮");
    assert_eq!(t2.node(a).unwrap().nags, vec![Nag::Good]);
    // 会话状态重置
    assert_eq!(t2.current, t2.root);
    assert!(!t2.redo_available());
    // 导出再导出：文档规范一致（不含会话状态）
    let json2 = serialize::to_tree_json(&t2).unwrap();
    assert_eq!(json2, json);
}

#[test]
fn tree_json_rejects_malformed_documents() {
    let mut t = tree();
    t.insert_move(mv("h2e2")).unwrap();
    let json = serialize::to_tree_json(&t).unwrap();

    // 孤儿节点：给节点 1 追加一个不存在的子节点
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["nodes"]["1"]["children"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!(99));
    let broken = serde_json::to_string(&value).unwrap();
    assert!(serialize::from_tree_json(&broken).is_err());

    // 坏版本
    let mut value = serde_json::from_str::<serde_json::Value>(&json).unwrap();
    value["version"] = serde_json::json!(99);
    let bad_version = serde_json::to_string(&value).unwrap();
    assert!(serialize::from_tree_json(&bad_version).is_err());

    // 坏 FEN
    let mut value = serde_json::from_str::<serde_json::Value>(&json).unwrap();
    value["startpos"] = serde_json::json!("not-a-fen");
    let bad_fen = serde_json::to_string(&value).unwrap();
    assert!(serialize::from_tree_json(&bad_fen).is_err());
}
