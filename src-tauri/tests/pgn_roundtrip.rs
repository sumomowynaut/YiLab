//! PGN 导入导出集成测试：round-trip（Import → Export → Import 等价）。

use pikaxiangqi_lib::board::types::Move;
use pikaxiangqi_lib::game::nag::Nag;
use pikaxiangqi_lib::game::tree::GameTree;
use pikaxiangqi_lib::io::pgn::{export, import};

fn assert_nodes_equal(a: &GameTree, aid: u64, b: &GameTree, bid: u64) {
    let an = a.node(aid).unwrap();
    let bn = b.node(bid).unwrap();
    assert_eq!(an.mv, bn.mv, "着法不一致");
    assert_eq!(an.comment, bn.comment, "注释不一致");
    assert_eq!(an.nags, bn.nags, "NAG 不一致");
    assert_eq!(an.children.len(), bn.children.len(), "子节点数不一致");
    for (ac, bc) in an.children.iter().zip(bn.children.iter()) {
        assert_nodes_equal(a, *ac, b, *bc);
    }
}

fn assert_trees_equivalent(a: &GameTree, b: &GameTree) {
    assert_eq!(a.startpos, b.startpos, "起始局面不一致");
    assert_eq!(a.headers, b.headers, "头部信息不一致");
    assert_nodes_equal(a, a.root, b, b.root);
}

/// 构造一棵带主变/变例/注释/NAG/头信息的棋谱树。
fn build_tree() -> GameTree {
    let mut tree = GameTree::new(pikaxiangqi_lib::board::types::START_FEN).unwrap();
    tree.headers.red = "红方".to_string();
    tree.headers.black = "黑方".to_string();
    tree.headers.event = "测试对局".to_string();
    tree.headers.date = "2026-08-22".to_string();
    tree.headers.result = "1-0".to_string();
    tree.headers.title = "标题".to_string();

    let n1 = tree.insert_move(Move::parse_uci("h2e2").unwrap()).unwrap();
    tree.set_comment_at(n1, "中炮开局".to_string()).unwrap();
    tree.set_nag_at(n1, Nag::Good, true).unwrap();

    let n2 = tree.insert_move(Move::parse_uci("h7e7").unwrap()).unwrap();
    tree.set_comment_at(n2, "顺炮".to_string()).unwrap();
    tree.set_nag_at(n2, Nag::Interesting, true).unwrap();

    tree.insert_move(Move::parse_uci("h0g2").unwrap()).unwrap();

    // h2e2 节点下的变例（嵌套：变例内部再分支）
    tree.set_current(n1).unwrap();
    let v1 = tree.insert_move(Move::parse_uci("b9c7").unwrap()).unwrap();
    tree.set_comment_at(v1, "变例一".to_string()).unwrap();
    tree.set_nag_at(v1, Nag::Dubious, true).unwrap();
    tree.insert_move(Move::parse_uci("h0g2").unwrap()).unwrap();
    tree.set_current(v1).unwrap();
    let v1sub = tree.insert_move(Move::parse_uci("b0c2").unwrap()).unwrap();
    tree.set_comment_at(v1sub, "嵌套变例".to_string()).unwrap();

    // 根节点变例（备选首着）
    tree.go_to_start().unwrap();
    let rv = tree.insert_move(Move::parse_uci("b0c2").unwrap()).unwrap();
    tree.set_comment_at(rv, "根变例".to_string()).unwrap();
    tree.set_nag_at(rv, Nag::Mistake, true).unwrap();

    tree
}

#[test]
fn roundtrip_preserves_game_tree() {
    let tree = build_tree();
    let pgn = export(&tree);
    let imported = import(&pgn).expect("导入失败");
    assert_trees_equivalent(&tree, &imported);
}

#[test]
fn root_variation_stays_variation_after_roundtrip() {
    let mut tree = GameTree::new(pikaxiangqi_lib::board::types::START_FEN).unwrap();
    let main = tree.insert_move(Move::parse_uci("h2e2").unwrap()).unwrap();
    tree.go_to_start().unwrap();
    let var = tree.insert_move(Move::parse_uci("b0c2").unwrap()).unwrap();
    assert_eq!(tree.node(tree.root).unwrap().children, vec![main, var]);

    let imported = import(&export(&tree)).expect("导入失败");
    let children = imported.node(imported.root).unwrap().children.clone();
    assert_eq!(children.len(), 2);
    assert_eq!(
        imported.node(children[0]).unwrap().mv,
        Some(Move::parse_uci("h2e2").unwrap())
    );
    assert_eq!(
        imported.node(children[1]).unwrap().mv,
        Some(Move::parse_uci("b0c2").unwrap())
    );
}

#[test]
fn export_import_export_is_stable() {
    let tree = build_tree();
    let e1 = export(&tree);
    let imported = import(&e1).expect("导入失败");
    let e2 = export(&imported);
    assert_eq!(e1, e2, "二次导出应一致");
}

#[test]
fn parse_handwritten_pgn() {
    let pgn = r#"[Event "示例对局"]
[Site ""]
[Date "2026.01.01"]
[Round "1"]
[White "甲"]
[Black "乙"]
[Result "1/2-1/2"]

1. h2e2 h7e7 2. h0g2 2... b9c7 (2... h9g7 {跳马变例} !?) 3. b0c2 1/2-1/2
"#;
    let tree = import(pgn).expect("导入失败");
    assert_eq!(tree.headers.red, "甲");
    assert_eq!(tree.headers.black, "乙");
    assert_eq!(tree.headers.event, "示例对局");
    assert_eq!(tree.headers.date, "2026.01.01");
    assert_eq!(tree.headers.result, "1/2-1/2");
    // 主线
    let main = tree.main_line();
    let main_moves: Vec<Option<Move>> = main.iter().map(|id| tree.node(*id).unwrap().mv).collect();
    let expected = ["h2e2", "h7e7", "h0g2", "b9c7", "b0c2"];
    for (i, e) in expected.iter().enumerate() {
        assert_eq!(
            main_moves[i + 1],
            Some(Move::parse_uci(e).unwrap()),
            "主线第 {i} 步不一致"
        );
    }
    // 变例挂在 h0g2 节点下，注释与 NAG
    let h0g2 = main[3];
    let h0g2_node = tree.node(h0g2).unwrap();
    assert_eq!(h0g2_node.children.len(), 2);
    let var_id = h0g2_node.children[1];
    let var = tree.node(var_id).unwrap();
    assert_eq!(var.mv, Some(Move::parse_uci("h9g7").unwrap()));
    assert_eq!(var.comment, "跳马变例");
    assert_eq!(var.nags, vec![Nag::Interesting]);
}

#[test]
fn reject_illegal_move() {
    let pgn = "1. h2e2 e2e5"; // e2 无子
    assert!(import(pgn).is_err());
}

#[test]
fn reject_unbalanced_paren() {
    let pgn = "(1. h2e2";
    assert!(import(pgn).is_err());
}

#[test]
fn reject_bad_fen_header() {
    let pgn = "[FEN \"garbage\"]\n1. h2e2";
    assert!(import(pgn).is_err());
}

#[test]
fn custom_fen_roundtrip() {
    let fen = "3k5/9/9/9/9/9/9/9/9/K8 b - - 0 1";
    let mut tree = GameTree::new(fen).unwrap();
    tree.insert_move(Move::parse_uci("d9d8").unwrap()).unwrap();
    let pgn = export(&tree);
    assert!(pgn.contains("[FEN \""), "自定义局面应带 FEN 头");
    let imported = import(&pgn).expect("导入失败");
    assert_trees_equivalent(&tree, &imported);
}
