# 开局库架构（Book）

> 实现状态（2026-08-23）：`src-tauri/src/book/` 已落地——`BookProvider` trait、
> `LocalBookProvider`（内存 + JSON 持久化）、`CloudBookProvider`（设计占位）、`BookChain`、
> 推荐策略与 Tauri 命令（`book_lookup` / `book_recommend` / `book_auto_move`）。
> SQLite 存储随 DB 阶段落地，上层不感知存储后端。

## 1. 总体架构

开局库抽象为一个 trait，两种实现。检索键为局面的 Zobrist 哈希（`board::zobrist`，确定性生成，跨运行稳定）。

```rust
pub struct BookMove {
    pub mv: Move,
    pub count: u32,                    // 出现次数
    pub stats: Option<BookStats>,      // 胜/和/负统计（数据源提供时才有）
}

pub struct BookStats { pub wins: u32, pub draws: u32, pub losses: u32 }

pub trait BookProvider: Send + Sync {
    fn name(&self) -> &'static str;
    /// 查当前局面的候选着法（按推荐度降序）
    fn lookup(&self, pos: &Position) -> Result<Vec<BookMove>, BookError>;
}

// 实现：
//   LocalBookProvider  —— 离线，内存 + JSON（SQLite 存储随 DB 阶段）
//   CloudBookProvider  —— 在线，皮卡鱼云库（设计占位，可降级）
//   BookChain          —— 组合：本地优先，未命中再查云库；永不失败（云库错误静默回退）
```

## 2. 本地开局库（LocalBookProvider）✅ 已实现

- 当前存储：内存 `HashMap<u64, Vec<BookMove>>`（键为 Zobrist 哈希）+ JSON 持久化（version 1，`save_to`/`load_from`）。
- **计划存储**：SQLite `book_entries` 表（见 `architecture.md` §5），`pos_key` 为 Zobrist 哈希——随 DB 阶段落地，`BookProvider` 接口不变。
- 查询：按 Zobrist 键查候选，**过滤非法着法**后按「得分 → 出现次数 → 着法字典序」降序返回（确定性排序）。
- 得分：`(wins + 0.5*draws) / (wins+draws+losses)`；无统计（`stats=None`）时以出现次数兜底排序。
- 来源：用户导入开局库文件（常见格式如旋风 OBK/PFBook 等）或自行积累的复盘数据。
  - `NEEDS_VERIFICATION`：确定首版支持的开局库导入格式（OBK/PFBook 规格）与权重文件授权。
- 完全离线，是「本地优先」的兜底能力。

## 3. 云库（CloudBookProvider）🔵 设计占位（查询待 API 确认）

- 对应皮卡鱼网页版「云库」：收录极多局面，服务器计算打分；含开局库与残局库，命中时返回候选着法并标注 W/D/L（胜/和/负）。
- 当前实现：`CloudBookProvider` 保留配置（endpoint）与查询入口，`lookup` 返回 `Unavailable`，**不发起任何网络请求**。
- 计划架构：发起一次 HTTP 查询（局面 → 候选着法 + W/D/L），结果缓存到 `book_entries`（`source='cloud-cache'`）供离线复用。
- `NEEDS_VERIFICATION`：云库公开 API 端点、请求/响应协议、使用条款与调用配额均未确认。
- **可降级（已实现并测试）**：`BookChain` 在云库失败/未命中时静默回退到本地库，返回空结果而非报错，不影响核心功能。

## 4. 自动走库（Book Chain）✅ 基础已实现

对应网页版「云库自动走棋」+「脱库步数」：

- 启用自动走库后，轮到皮卡鱼（或自动落子）时，先查 `BookChain`：
  1. 命中 → 直接走库招（不调引擎）。
  2. 未命中 → 交给引擎计算。
- 已实现：`BookChain`（本地优先 + 云库回退）、推荐策略（`best_score` 最高胜率 / `most_popular` 出现次数最多 / `first` 首条）、
  Tauri 命令 `book_recommend` 与 `book_auto_move`（把推荐着法插入当前棋谱树，未命中返回 `applied=None`）。
- 未实现（UI/流程阶段）：「脱库步数 N」（前 N 个半回合内命中才走库）、走库与引擎的自动回退循环（命中走库、未命中交给引擎）。

## 5. 决策与边界

- 本地库只读优先；当前仅提供 `add_entry`/JSON 加载（测试与未来「从对局学习」用），不实现「开局库编辑/合并」。
- 云库命中结果缓存带 TTL，避免每次局面重复请求（随云库查询实现）。
- 开局库模块与引擎模块完全解耦：`book` 不依赖 `engine`，走库是「开局库 → 棋谱树」的直接路径，不经过 UCI。

## 6. 当前版本说明

«文档状态：历史设计方案»

本文档记录 YiLab 曾规划的开局库（Book）架构与实现方案。

当前版本已暂不提供开局库功能。 与开局库相关的 UI、Tauri 命令及运行时功能已从当前版本中移除，因此本文档中的 "BookProvider"、"LocalBookProvider"、"CloudBookProvider"、"BookChain"、"book_lookup"、"book_recommend"、"book_auto_move" 等内容不代表当前版本的实际功能，仅作为历史设计记录保留。

如未来重新实现开局库功能，将以当前项目架构为基础重新评估具体实现方式、数据来源、文件格式及相关授权。
