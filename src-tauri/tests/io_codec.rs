//! 导入导出框架集成测试：Codec trait、格式嗅探、FEN/PGN 适配器。

use pikaxiangqi_lib::board::types::START_FEN;
use pikaxiangqi_lib::game::nag::Nag;
use pikaxiangqi_lib::game::tree::GameTree;
use pikaxiangqi_lib::io::{codec, sniff, Format};

#[test]
fn pgn_codec_roundtrip_via_trait() {
    let mut tree = GameTree::new(START_FEN).unwrap();
    tree.headers.red = "红方".to_string();
    let n1 = tree
        .insert_move(pikaxiangqi_lib::board::types::Move::parse_uci("h2e2").unwrap())
        .unwrap();
    tree.set_comment_at(n1, "中炮".to_string()).unwrap();
    tree.set_nag_at(n1, Nag::Good, true).unwrap();
    tree.insert_move(pikaxiangqi_lib::board::types::Move::parse_uci("h7e7").unwrap())
        .unwrap();

    let c = codec(Format::Pgn);
    let text = c.serialize(&tree).unwrap();
    assert!(text.contains("[Event"));
    let imported = c.parse(&text).unwrap();
    assert_eq!(imported.headers.red, "红方");
    assert_eq!(imported.main_line().len(), 3);
    let n = imported.node(imported.main_line()[1]).unwrap();
    assert_eq!(n.comment, "中炮");
    assert_eq!(n.nags, vec![Nag::Good]);
}

#[test]
fn fen_codec_creates_tree_from_fen() {
    let fen = "3k5/9/9/9/9/9/9/9/9/K8 b - - 0 1";
    let c = codec(Format::Fen);
    let tree = c.parse(fen).unwrap();
    assert_eq!(tree.startpos, fen);
    assert_eq!(c.serialize(&tree).unwrap(), fen);
}

#[test]
fn sniff_routes_text_to_pgn_or_fen() {
    assert_eq!(sniff("[Event \"x\"]\n1. h2e2 h7e7"), Format::Pgn);
    assert_eq!(sniff("1. h2e2 (1. b0c2) h7e7"), Format::Pgn);
    assert_eq!(sniff(START_FEN), Format::Fen);
}
