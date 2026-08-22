//! 开局库（Book）集成测试：LocalBookProvider、BookChain、推荐策略、自动走库。

use pikaxiangqi_lib::board::fen::parse_fen;
use pikaxiangqi_lib::board::rules::apply_move;
use pikaxiangqi_lib::board::types::{Move, Position, START_FEN};
use pikaxiangqi_lib::book::local::LocalBookProvider;
use pikaxiangqi_lib::book::{
    BookChain, BookError, BookMove, BookProvider, BookStats, BookStrategy,
};
use pikaxiangqi_lib::game::tree::GameTree;

fn mv(uci: &str) -> Move {
    Move::parse_uci(uci).unwrap()
}

fn stats(w: u32, d: u32, l: u32) -> BookStats {
    BookStats {
        wins: w,
        draws: d,
        losses: l,
    }
}

fn start() -> Position {
    parse_fen(START_FEN).unwrap()
}

#[test]
fn local_book_returns_sorted_candidates() {
    let pos = start();
    let mut book = LocalBookProvider::new();
    book.add_entry(&pos, mv("h2e2"), 100, Some(stats(40, 30, 30))); // score 0.55
    book.add_entry(&pos, mv("b0c2"), 10, Some(stats(9, 0, 1))); // score 0.9
    book.add_entry(&pos, mv("h0g2"), 50, None);

    let moves = book.lookup(&pos).unwrap();
    let ucis: Vec<String> = moves.iter().map(|m| m.mv.uci()).collect();
    assert_eq!(ucis, vec!["b0c2", "h2e2", "h0g2"]);
    assert_eq!(moves[0].stats.unwrap().total(), 10);
    assert_eq!(moves[2].count, 50);
}

#[test]
fn recommend_strategies_pick_expected_move() {
    let pos = start();
    let mut book = LocalBookProvider::new();
    book.add_entry(&pos, mv("h2e2"), 100, Some(stats(40, 30, 30)));
    book.add_entry(&pos, mv("b0c2"), 10, Some(stats(9, 0, 1)));
    book.add_entry(&pos, mv("h0g2"), 50, None);
    let chain = BookChain::local_only(Box::new(book));

    assert_eq!(
        chain
            .recommend(&pos, BookStrategy::BestScore)
            .unwrap()
            .mv
            .uci(),
        "b0c2"
    );
    assert_eq!(
        chain
            .recommend(&pos, BookStrategy::MostPopular)
            .unwrap()
            .mv
            .uci(),
        "h2e2"
    );
    assert_eq!(
        chain.recommend(&pos, BookStrategy::First).unwrap().mv.uci(),
        "b0c2"
    );
}

#[test]
fn chain_uses_local_first_and_skips_cloud() {
    let pos = start();
    let mut local = LocalBookProvider::new();
    local.add_entry(&pos, mv("h2e2"), 10, Some(stats(9, 0, 1)));

    // 云库若被查询会返回另一着法；本地命中时不应触发云库
    let cloud = MockProvider {
        name: "mock-cloud",
        result: Ok(vec![BookMove::new(mv("h7e7"), 1, None)]),
    };
    let chain = BookChain::new(Box::new(local), Some(Box::new(cloud)));

    let moves = chain.lookup(&pos);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].mv.uci(), "h2e2");
}

#[test]
fn chain_falls_back_to_cloud_on_local_miss() {
    let pos = start();
    let local = LocalBookProvider::new();
    let cloud = MockProvider {
        name: "mock-cloud",
        result: Ok(vec![BookMove::new(mv("b0c2"), 5, Some(stats(4, 1, 0)))]),
    };
    let chain = BookChain::new(Box::new(local), Some(Box::new(cloud)));

    let moves = chain.lookup(&pos);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].mv.uci(), "b0c2");
}

#[test]
fn chain_cloud_failure_degrades_gracefully() {
    let pos = start();
    let local = LocalBookProvider::new();
    let cloud = MockProvider {
        name: "mock-cloud",
        result: Err(BookError::Unavailable("网络不可用".into())),
    };
    let chain = BookChain::new(Box::new(local), Some(Box::new(cloud)));

    // 网络失败：返回空结果而不是报错/panic，软件继续正常工作
    assert!(chain.lookup(&pos).is_empty());
    assert_eq!(chain.recommend(&pos, BookStrategy::BestScore), None);
}

#[test]
fn local_book_json_roundtrip() {
    let pos = start();
    let mut book = LocalBookProvider::new();
    book.add_entry(&pos, mv("h2e2"), 100, Some(stats(40, 30, 30)));
    book.add_entry(&pos, mv("b0c2"), 10, None);

    let json = book.to_json().unwrap();
    let mut loaded = LocalBookProvider::new();
    loaded.load_json(&json).unwrap();
    assert_eq!(book.lookup(&pos).unwrap(), loaded.lookup(&pos).unwrap());
}

#[test]
fn auto_play_book_move_advances_game_tree() {
    // 开局库只收录「起始局面 → 中炮」；走库应把推荐着法插入棋谱树并推进局面。
    let pos = start();
    let mut local = LocalBookProvider::new();
    // h2e2 得分 (90+0.5*5)/100 = 0.925 > b0c2 0.90
    local.add_entry(&pos, mv("h2e2"), 100, Some(stats(90, 5, 5)));
    local.add_entry(&pos, mv("b0c2"), 10, Some(stats(9, 0, 1)));
    let chain = BookChain::local_only(Box::new(local));

    let rec = chain.recommend(&pos, BookStrategy::BestScore).unwrap();
    assert_eq!(rec.mv.uci(), "h2e2");

    let mut tree = GameTree::new(START_FEN).unwrap();
    tree.insert_move(rec.mv).unwrap();

    let main = tree.main_line();
    assert_eq!(main.len(), 2); // 根 + h2e2
    assert_eq!(tree.current_node().mv, Some(mv("h2e2")));
    // 新局面对应 h2e2 后的局面
    let expected = apply_move(&pos, mv("h2e2")).unwrap();
    let restored = tree.restore_position(tree.current_id()).unwrap();
    assert_eq!(restored.board, expected.board);
    assert_eq!(restored.side_to_move, expected.side_to_move);
}

#[test]
fn current_position_query_through_game_tree() {
    // 演示命令层「当前棋谱树节点 → 开局库查询」路径：
    // 树中走出 h2e2 后，查询该局面应命中黑方应着开局库。
    let mut tree = GameTree::new(START_FEN).unwrap();
    tree.insert_move(mv("h2e2")).unwrap();
    let pos = tree.restore_position(tree.current_id()).unwrap();

    let mut local = LocalBookProvider::new();
    local.add_entry(&pos, mv("h7e7"), 80, Some(stats(40, 20, 20))); // 顺炮
    local.add_entry(&pos, mv("h9g7"), 20, Some(stats(5, 5, 10))); // 屏风马
    let chain = BookChain::local_only(Box::new(local));

    let moves = chain.lookup(&pos);
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].mv.uci(), "h7e7"); // 更高胜率排在首位
    assert_eq!(
        chain
            .recommend(&pos, BookStrategy::BestScore)
            .unwrap()
            .mv
            .uci(),
        "h7e7"
    );
}

#[test]
fn game_tree_current_plies_counts_half_moves() {
    let mut tree = GameTree::new(START_FEN).unwrap();
    assert_eq!(tree.current_plies(), 0);
    tree.insert_move(mv("h2e2")).unwrap();
    tree.insert_move(mv("h7e7")).unwrap();
    tree.insert_move(mv("h0g2")).unwrap();
    assert_eq!(tree.current_plies(), 3);
    tree.go_to_start().unwrap();
    assert_eq!(tree.current_plies(), 0);
    tree.next_move().unwrap();
    assert_eq!(tree.current_plies(), 1);
}

#[test]
fn auto_play_book_move_respects_exit_plies() {
    let pos = start();
    let mut local = LocalBookProvider::new();
    local.add_entry(&pos, mv("h2e2"), 100, Some(stats(90, 5, 5)));
    let chain = BookChain::local_only(Box::new(local));

    let mut tree = GameTree::new(START_FEN).unwrap();
    // 第 0 半回合：脱库步数 4 内 → 走库
    let within = chain.recommend_book(&pos, BookStrategy::BestScore, tree.current_plies(), Some(4));
    assert!(within.is_some());
    // 模拟已走 5 个半回合：脱库步数 4 外 → 不走库
    let beyond = chain.recommend_book(&pos, BookStrategy::BestScore, 5, Some(4));
    assert_eq!(beyond, None);
    // 不限制脱库步数 → 始终可走库
    let unlimited = chain.recommend_book(&pos, BookStrategy::BestScore, 5, None);
    assert!(unlimited.is_some());
    // 走库并推进棋谱树
    let rec = chain
        .recommend_book(&pos, BookStrategy::BestScore, 0, Some(4))
        .unwrap();
    tree.insert_move(rec.mv).unwrap();
    assert_eq!(tree.current_plies(), 1);
}

/// 测试用：可控结果的 mock 提供者（模拟云库命中/失败）。
struct MockProvider {
    name: &'static str,
    result: Result<Vec<BookMove>, BookError>,
}

impl BookProvider for MockProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn lookup(&self, _pos: &Position) -> Result<Vec<BookMove>, BookError> {
        self.result.clone()
    }
}
