//! 开局库（Book）：`BookProvider` trait + 本地/云库实现 + 组合链。
//!
//! 设计要点：
//! - `BookProvider` 抽象「当前位置 → 候选着法」查询；检索键为局面 Zobrist 哈希（`board::zobrist`）。
//! - `LocalBookProvider`：完全离线，内存 + JSON 持久化；SQLite 存储随 DB 阶段落地（见 docs/book.md）。
//! - `CloudBookProvider`：云库公开 API 未确认（`NEEDS_VERIFICATION`），当前查询返回 `Unavailable`；
//!   通过 `BookChain` 保证云库失败/未命中时静默回退，不构成核心依赖。
//! - **与引擎完全解耦**：开局库不依赖 `engine` 模块，走库是「开局库 → 棋谱树」的直接路径。

pub mod cloud;
pub mod dto;
pub mod local;

use crate::board::types::{Move, Position};
use std::cmp::Ordering;

/// 胜/和/负统计（数据源提供时使用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BookStats {
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl BookStats {
    /// 统计总对局数。
    pub fn total(&self) -> u32 {
        self.wins + self.draws + self.losses
    }

    /// 得分（胜=1、和=0.5、负=0），范围 [0,1]；无数据返回 0。
    pub fn score(&self) -> f64 {
        let t = self.total();
        if t == 0 {
            return 0.0;
        }
        (self.wins as f64 + 0.5 * self.draws as f64) / t as f64
    }

    /// 胜率 [0,1]；无数据返回 None。
    pub fn win_rate(&self) -> Option<f64> {
        let t = self.total();
        if t == 0 {
            None
        } else {
            Some(self.wins as f64 / t as f64)
        }
    }
}

/// 一条开局库候选着法。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookMove {
    pub mv: Move,
    /// 出现次数（总对局/命中数）。
    pub count: u32,
    /// 胜/和/负统计；数据源未提供时为 None。
    pub stats: Option<BookStats>,
}

impl BookMove {
    pub fn new(mv: Move, count: u32, stats: Option<BookStats>) -> Self {
        BookMove { mv, count, stats }
    }

    /// 推荐分：有统计时用统计得分；否则返回 0（排序时以出现次数兜底）。
    pub fn score(&self) -> f64 {
        match self.stats {
            Some(s) if s.total() > 0 => s.score(),
            _ => 0.0,
        }
    }
}

/// 开局库查询错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookError {
    /// 数据源不可用（云库未确认/网络失败等）。
    Unavailable(String),
    /// 数据损坏（JSON 解析失败、非法着法等）。
    Corrupt(String),
}

/// 开局库提供者抽象。
pub trait BookProvider: Send + Sync {
    /// 数据源名称（用于日志/UI 展示）。
    fn name(&self) -> &'static str;

    /// 查询当前局面的候选着法（按推荐度降序）。
    fn lookup(&self, pos: &Position) -> Result<Vec<BookMove>, BookError>;
}

/// 推荐着法策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookStrategy {
    /// 最高得分（胜率优先）。
    BestScore,
    /// 出现次数最多。
    MostPopular,
    /// 数据源首条（按顺序）。
    First,
}

impl BookStrategy {
    pub fn name(self) -> &'static str {
        match self {
            BookStrategy::BestScore => "best_score",
            BookStrategy::MostPopular => "most_popular",
            BookStrategy::First => "first",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "best_score" => Some(BookStrategy::BestScore),
            "most_popular" => Some(BookStrategy::MostPopular),
            "first" => Some(BookStrategy::First),
            _ => None,
        }
    }
}

/// 按策略从候选着法中选一条推荐着法。
pub fn recommend(moves: &[BookMove], strategy: BookStrategy) -> Option<BookMove> {
    if moves.is_empty() {
        return None;
    }
    match strategy {
        BookStrategy::First => Some(moves[0].clone()),
        BookStrategy::MostPopular => {
            let mut v = moves.to_vec();
            v.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.mv.uci().cmp(&b.mv.uci()))
            });
            v.into_iter().next()
        }
        BookStrategy::BestScore => {
            let mut v = moves.to_vec();
            v.sort_by(|a, b| {
                b.score()
                    .partial_cmp(&a.score())
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| b.count.cmp(&a.count))
                    .then_with(|| a.mv.uci().cmp(&b.mv.uci()))
            });
            v.into_iter().next()
        }
    }
}

/// 组合链：本地优先，未命中再查云库；云库失败/未命中静默回退。
///
/// 这是对外暴露的统一入口，**永不失败**：云库错误被吞掉，返回当前可获得的最佳结果。
pub struct BookChain {
    local: Box<dyn BookProvider>,
    cloud: Option<Box<dyn BookProvider>>,
}

impl BookChain {
    pub fn new(local: Box<dyn BookProvider>, cloud: Option<Box<dyn BookProvider>>) -> Self {
        BookChain { local, cloud }
    }

    pub fn local_only(local: Box<dyn BookProvider>) -> Self {
        BookChain { local, cloud: None }
    }

    /// 查询候选着法（本地优先，未命中回退云库）。
    pub fn lookup(&self, pos: &Position) -> Vec<BookMove> {
        if let Ok(moves) = self.local.lookup(pos) {
            if !moves.is_empty() {
                return moves;
            }
        }
        if let Some(cloud) = &self.cloud {
            if let Ok(moves) = cloud.lookup(pos) {
                if !moves.is_empty() {
                    return moves;
                }
            }
        }
        Vec::new()
    }

    /// 推荐一步着法（按策略）。
    pub fn recommend(&self, pos: &Position, strategy: BookStrategy) -> Option<BookMove> {
        recommend(&self.lookup(pos), strategy)
    }
}

/// 测试用：可控结果的 mock 提供者。
#[cfg(test)]
struct MockProvider {
    name: &'static str,
    result: Result<Vec<BookMove>, BookError>,
}

#[cfg(test)]
impl BookProvider for MockProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn lookup(&self, _pos: &Position) -> Result<Vec<BookMove>, BookError> {
        self.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::types::START_FEN;

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

    #[test]
    fn stats_score_and_win_rate() {
        let s = stats(5, 3, 2);
        assert_eq!(s.total(), 10);
        assert!((s.score() - 0.65).abs() < 1e-9);
        assert_eq!(s.win_rate(), Some(0.5));
        assert_eq!(BookStats::default().score(), 0.0);
        assert_eq!(BookStats::default().win_rate(), None);
    }

    #[test]
    fn book_move_score_falls_back_to_zero_without_stats() {
        let with_stats = BookMove::new(mv("h2e2"), 10, Some(stats(8, 1, 1)));
        let without = BookMove::new(mv("h7e7"), 10, None);
        assert!(with_stats.score() > 0.8);
        assert_eq!(without.score(), 0.0);
    }

    #[test]
    fn recommend_by_strategy() {
        let a = BookMove::new(mv("h2e2"), 100, Some(stats(40, 30, 30))); // score 0.55
        let b = BookMove::new(mv("b0c2"), 10, Some(stats(9, 0, 1))); // score 0.9
        let c = BookMove::new(mv("h0g2"), 50, None); // count 50
        let list = vec![a.clone(), b.clone(), c.clone()];
        assert_eq!(
            recommend(&list, BookStrategy::BestScore).unwrap().mv,
            mv("b0c2")
        );
        assert_eq!(
            recommend(&list, BookStrategy::MostPopular).unwrap().mv,
            mv("h2e2")
        );
        assert_eq!(
            recommend(&list, BookStrategy::First).unwrap().mv,
            mv("h2e2")
        );
        assert_eq!(recommend(&[], BookStrategy::BestScore), None);
    }

    #[test]
    fn strategy_name_roundtrip() {
        for s in [
            BookStrategy::BestScore,
            BookStrategy::MostPopular,
            BookStrategy::First,
        ] {
            assert_eq!(BookStrategy::from_name(s.name()), Some(s));
        }
        assert_eq!(BookStrategy::from_name("bogus"), None);
    }

    #[test]
    fn chain_local_first_then_cloud() {
        let local = crate::book::local::LocalBookProvider::new();
        let cloud = MockProvider {
            name: "mock-cloud",
            result: Ok(vec![BookMove::new(mv("h7e7"), 1, None)]),
        };
        let chain = BookChain::new(Box::new(local), Some(Box::new(cloud)));
        let pos = crate::board::fen::parse_fen(START_FEN).unwrap();
        // 本地未命中 → 回退云库
        let moves = chain.lookup(&pos);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].mv, mv("h7e7"));
    }

    #[test]
    fn chain_cloud_failure_degrades_gracefully() {
        let local = crate::book::local::LocalBookProvider::new();
        let cloud = MockProvider {
            name: "mock-cloud",
            result: Err(BookError::Unavailable("network down".into())),
        };
        let chain = BookChain::new(Box::new(local), Some(Box::new(cloud)));
        let pos = crate::board::fen::parse_fen(START_FEN).unwrap();
        assert!(chain.lookup(&pos).is_empty());
        assert_eq!(chain.recommend(&pos, BookStrategy::BestScore), None);
    }
}
