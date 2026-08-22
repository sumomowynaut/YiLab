//! 开局库查询结果 DTO（发送给 React 前端）。

use serde::Serialize;

use super::BookMove;

/// 一条候选着法（前端展示用）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMoveDto {
    /// 着法（UCI-Cyclone）。
    pub mv: String,
    /// 出现次数。
    pub count: u32,
    /// 胜/和/负统计（数据源提供时才有）。
    pub wins: Option<u32>,
    pub draws: Option<u32>,
    pub losses: Option<u32>,
    /// 推荐分 [0,1]。
    pub score: f64,
    /// 是否带统计信息。
    pub has_stats: bool,
}

impl From<&BookMove> for BookMoveDto {
    fn from(bm: &BookMove) -> Self {
        BookMoveDto {
            mv: bm.mv.uci(),
            count: bm.count,
            wins: bm.stats.map(|s| s.wins),
            draws: bm.stats.map(|s| s.draws),
            losses: bm.stats.map(|s| s.losses),
            score: bm.score(),
            has_stats: bm.stats.is_some(),
        }
    }
}
