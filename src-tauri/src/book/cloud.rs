//! 云库（CloudBookProvider）——设计占位。
//!
//! 皮卡鱼网页版「云库」的公开 API 端点/请求协议/使用条款/调用配额均未确认
//! （`NEEDS_VERIFICATION`，见 docs/book.md §3 与 docs/development-plan.md「未知项」）。
//!
//! 当前实现：
//! - 保留配置字段（endpoint 等）与查询入口，`lookup` 返回 `Unavailable`；
//! - **不发起任何网络请求**，不构成核心依赖；
//! - 组合链 `BookChain` 在云库失败/未命中时静默回退到本地库，软件正常工作。
//!
//! 待 API 确认后：在此实现「局面（FEN/Zobrist）→ HTTP 请求 → 候选着法 + W/D/L」，
//! 结果可缓存到本地（`source='cloud-cache'`）供离线复用；接入 `BookChain` 无需改动上层。

use crate::board::types::Position;

use super::{BookError, BookMove, BookProvider};

/// 云库提供者（当前为设计占位，不发起网络请求）。
pub struct CloudBookProvider {
    /// 云库端点（API 确认后使用）。
    endpoint: String,
}

impl CloudBookProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        CloudBookProvider {
            endpoint: endpoint.into(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl BookProvider for CloudBookProvider {
    fn name(&self) -> &'static str {
        "cloud"
    }

    fn lookup(&self, _pos: &Position) -> Result<Vec<BookMove>, BookError> {
        Err(BookError::Unavailable(
            "云库 API 未确认（NEEDS_VERIFICATION），当前不可用".to_string(),
        ))
    }
}
