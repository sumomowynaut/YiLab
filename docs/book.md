# 开局库架构（Book）

## 1. 总体架构

开局库抽象为一个 trait，两种实现。检索键为局面的 Zobrist 哈希（`game-model.md` §3）。

```rust
pub struct BookMove {
    pub mv: Move,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

pub trait BookProvider {
    fn name(&self) -> &'static str;
    /// 查当前局面的候选着法（按胜率/权重排序）
    fn lookup(&self, pos: &Position) -> BookResult<Vec<BookMove>>;
}

// 实现：
//   LocalBookProvider  —— 离线，SQLite 本地库
//   CloudBookProvider  —— 在线，皮卡鱼云库（可降级）
//   BookChain          —— 组合：本地优先，未命中再查云库
```

## 2. 本地开局库（LocalBookProvider）

- 存储：SQLite `book_entries` 表（见 `architecture.md` §5），`pos_key` 为 Zobrist 哈希，`moves` 为 JSON 数组 `[{uci, wins, draws, losses}]`。
- 来源：用户导入开局库文件（常见格式如旋风 OBK/PFBook 等）或自行积累的复盘数据。
  - `NEEDS_VERIFICATION`：确定首版支持的开局库导入格式（OBK/PFBook 规格）与权重文件授权。
- 查询：`SELECT moves FROM book_entries WHERE pos_key = ?`，按 `wins/(wins+draws+losses)` 或用户策略排序。
- 完全离线，是「本地优先」的兜底能力。

## 3. 云库（CloudBookProvider）

- 对应皮卡鱼网页版「云库」：收录极多局面，服务器计算打分；含开局库与残局库，命中时返回候选着法并标注 W/D/L（胜/和/负）。
- 架构：`CloudBookProvider` 发起一次 HTTP 查询（局面 → 候选着法 + W/D/L），结果缓存到 `book_entries`（`source='cloud-cache'`）供离线复用。
- `NEEDS_VERIFICATION`：云库公开 API 端点、请求/响应协议、使用条款与调用配额均未确认。
- **可降级**：网络不可用或查询失败时静默回退到本地库，不影响核心功能。

## 4. 自动走库（Book Chain）

对应网页版「云库自动走棋」+「脱库步数」：

- 启用自动走库后，轮到皮卡鱼（或自动落子）时，先查 `BookChain`：
  1. 命中 → 直接走库招（不调引擎）。
  2. 未命中 → 交给引擎计算。
- 「脱库步数 N」：前 N 步内命中才走库，超过 N 步后即使云库有招也由引擎计算（N 步 = N 个半回合，语义与网页版一致）。
- 走库候选选择策略可配置：最高胜率 / 加权随机 / 首推。

## 5. 决策与边界

- 本地库只读优先；首版不实现「开局库编辑/合并」，避免过度工程。
- 云库命中结果缓存带 TTL，避免每次局面重复请求。
- 开局库模块与引擎模块完全解耦：走库是「开局库 → 棋谱树」的直接路径，不经过 UCI。