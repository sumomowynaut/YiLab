# STATUS.md

> 本文档记录 PikaXiangqi 项目的实时状态。提交代码前先更新这里。
> 最后更新：2026-08-23

## 当前阶段

**开局库（Opening Book）✅ 基础完成**

- ✅ `src-tauri/src/book/`：`BookProvider` trait + `BookMove`/`BookStats` + 推荐策略 + `BookChain`（本地优先，云库失败静默回退，**永不失败**）。
- ✅ `LocalBookProvider`：内存 `HashMap<u64, Vec<BookMove>>` + JSON 持久化（version 1）；查询按局面 Zobrist 哈希（`board::zobrist`，确定性），**过滤非法着法**后按「得分 → 出现次数 → 字典序」降序返回；支持胜/和/负统计（数据源提供时）。
- ✅ `CloudBookProvider`：设计占位（endpoint 保留，`lookup` 返回 `Unavailable`，**不发起网络请求**）；云库 API 未确认（`NEEDS_VERIFICATION`，见 docs/book.md §3）。
- ✅ 推荐/自动走库：`BookChain::recommend`（best_score / most_popular / first）+ Tauri 命令 `book_lookup` / `book_recommend` / `book_auto_move`（把推荐着法插入当前棋谱树，未命中返回 `applied=None`）。
- ✅ **与引擎完全解耦**：`book` 不依赖 `engine` 模块；走库是「开局库 → 棋谱树」直接路径。
- ✅ 测试：Rust 单元 17（zobrist 3 + book 14）+ 集成 8（排序/推荐/链回退/降级/JSON 往返/自动走库/树查询）。
- ⚪ 开局库导入格式（OBK/PFBook，`NEEDS_VERIFICATION`）、SQLite 存储（DB 阶段）、脱库步数与引擎回退循环（UI 阶段）。

**PGN 导入导出 ✅ 完成（Phase 2 收尾）**

- ✅ `src-tauri/src/io/pgn.rs`：PGN parser + exporter（`import` / `export`）。
- ✅ 支持：Game metadata（`White`/`Black`/`Event`/`Date`/`Result`/`FEN`/自定义 `PikaXiangqiTitle`）、Moves、Variations（含嵌套）、Comments `{...}`、NAG（`! ? !! ?? !? ?! = ~` 与 `$n`）。
- ✅ 走法记谱：UCI-Cyclone（如 `h2e2`）；中文纵线制导入暂不支持（`NEEDS_VERIFICATION`，见 `docs/import-export.md`）。
- ✅ **变例归属修复**：导入保留回合前缀（`N.` 红 / `N...` 黑），变例首着按前缀沿祖先链定位分支点；主线续着经 `GameTree::insert_main_at` 始终落在 `children[0]`，解决「变例写在主线续着之前导致主/变例顺序颠倒」的问题。
- ✅ 往返保证：`import(export(tree))` 在主线/变例/注释/NAG/头信息上等价；二次导出（Export → Import → Export）文本稳定。
- ✅ Tauri 命令：`pgn_import` / `pgn_export`（`commands.rs`）。
- ✅ 测试：Rust 单元 7（tokenize/NAG/round-trip 基础）+ 集成 8（树等价、二次导出稳定、手写 PGN、非法着法/括号/FEN 拒绝、根级变例、自定义 FEN）。
- ⚪ 通用导入导出框架（Codec trait）、文件/粘贴/复制 UI 入口、TXT/XQF/东萍：后续 Phase。

**Phase 3（Pikafish Engine）✅ 引擎层 + 分析 UI 完成**

- ✅ `src-tauri/src/engine/`：Engine interface（`EngineManager`）、Engine Process（tokio 异步）、UCI parser / command builder。
- ✅ 支持：`uci` / `isready` / `setoption` / `position` / `go` / `stop` / `quit`。
- ✅ 解析：`info`（depth/seldepth/score cp|mate/nodes/nps/time/pv/multipv/lowerbound/upperbound）、`bestmove`（含 ponder、(none)）。
- ✅ 选项：Threads / Hash / MultiPV / Depth / MoveTime / Nodes（`GoParams` + `setoption`）。
- ✅ 生命周期处理：启动失败（握手超时/提前退出）、崩溃（stdout EOF → Crashed 事件）、停止/等待超时（token 定时器）、restart、quit、**分析期间切换局面**（先 stop 等 bestmove 再 position+go）。
- ✅ Mock 引擎（`mock_engine` bin，可通过 `MOCK_BEHAVIOR` 注入 no_uciok/no_readyok/crash_on_go/hang_on_go）驱动 9 个集成测试。
- ✅ 真实 Pikafish 冒烟测试（`tests/pikafish_smoke.rs`，默认 `#[ignore]`）：官方 Pikafish-2026-01-02 本地运行通过（握手、Threads/Hash/MultiPV、分析出 info+bestmove）。
- ✅ 引擎分析 UI：Analysis Panel（评价/深度/节点/NPS/时间/MultiPV/PV）、引擎参数（Threads/Hash/Depth/MultiPV 1/2/3/5/10）、开始/停止/重启、**点击 PV 在棋盘预览**、**快速切换局面防旧分析覆盖（epoch + Searching 边界事件）**。
- ⛔ 评价曲线 / 自动复盘 / 人机对弈循环：后续 Phase 3 子任务。

**Phase 2（Game Tree 棋谱树）✅ 核心完成 + 架构审查修复完成**

- ✅ Rust 棋谱树 `src-tauri/src/game/`：真实树结构（Root / MoveNode / MainLine / Variation / Nested Variation / Undo / Redo / InsertMove / DeleteVariation / Navigate / Comments / NAG / Promote / Reorder / 文档序列化）。
- ✅ 从任意节点回放父链恢复完整 Position（`restore_position`，与缓存 FEN 一致性测试通过）。
- ✅ React Move Tree UI：点击跳转、←/→ 导航、Ctrl+Z/Y 悔棋/重做、变例展开/删除/**提升主线/上移/下移**、注释显示与编辑、NAG、当前棋步高亮。
- ✅ 测试：Rust 83（棋盘 53 + 棋谱树集成 30）、前端 30。

### 架构审查修复（commit `fix: address game tree architecture review`）

**已修复**

| 项 | 内容 |
|----|------|
| H1 | 注释/NAG 修改改为**按 node_id 显式定位**（`game_set_comment(node_id, ...)` / `game_set_nag(node_id, ...)`，Rust `set_comment_at`/`set_nag_at`），不再依赖全局 current；附回归测试 |
| H2 | 明确 **Document State** 与 **Session State** 边界；`game::serialize`（tree_json v1）只序列化文档字段，导入时校验结构并重置会话状态；往返测试证明 current/redo_stack 不进入持久化 |
| H3 | `GameNode` 插入时缓存 `side_to_move`/`fullmove_number`，快照不再逐节点 `parse_fen`；一致性测试 |
| M1 | store 非编辑态 position 由 `snapshot.position` 派生（`selectDisplayPosition`），编辑态使用独立 `editPosition` |
| M2 | 实现 `promote_variation` / `reorder_variation`（Rust + 命令 + React UI 提升/上移/下移 + 测试） |
| M3 | 新增 `attacks_square`（免临时 Vec 分配）并接入 `is_attacked`；等价性测试；perft 44/1920/79666 保持通过 |

**暂缓/记录**：`GameSession` 拆分、增量快照、`truncate`、store 拆分、`apply_move` 分配优化（均已记录于 docs/STATUS）。

## 引擎许可状态（只记录，不判断）

- 冒烟测试使用的 Pikafish 二进制与 NNUE 权重仅为**本机本地验证**，存放在 `.toolchain/pikafish/`（已 gitignore），**不提交、不参与任何分发路径**。
- 分发/捆绑（安装包内置引擎与权重）仍受许可证决策约束（见 `docs/licensing.md`），**在决策完成前不实现任何分发相关代码**。

## 验收命令（全部通过）

| 命令 | 结果 | 说明 |
|------|------|------|
| `cargo test` | ✅ | 90 lib + 10 engine_manager + 30 game_tree + 8 opening_book + 8 pgn_roundtrip（+1 pikafish 冒烟默认 ignore；沙箱需清单变通） |
| `cargo clippy --all-targets -- -D warnings` | ✅ | |
| `cargo fmt --check` | ✅ | |
| `cargo check` | ✅ | |
| `npm run test` | ✅ | 43 个前端用例 |
| `npm run lint` | ✅ | 0 error 0 warning |
| `npm run format:check` | ✅ | |
| `npm run build` | ✅ | tsc + vite build |
| `npm run check` | ✅ | tsc --noEmit |

## 本机（沙箱）环境说明

- 本机未安装 MSVC Build Tools；Rust 使用 **GNU 工具链**（rustup 安装在仓库内 `.toolchain/`，已 gitignore）。
- 项目路径含中文（`D:\Codex 项目\PikaXiangqi`），GNU 工具链无法处理非 ASCII 路径。
  **变通：设置 `CARGO_TARGET_DIR` 为纯 ASCII 目录**。
- **Tauri 运行时链接与清单（仅 GNU 沙箱需要）**：`cargo test` 时设置
  `RUSTFLAGS="-C link-arg=<build>\libresource.a"`；MSVC / CI（windows-latest）无需变通。
- 真实 Pikafish 冒烟测试：`PIKAFISH_BIN=<engine.exe> PIKAFISH_CWD=<目录(含 pikafish.nnue)> cargo test --test pikafish_smoke -- --ignored`。
- 浏览器 `npm run dev` 使用内存回退 API；走子/编辑/棋谱树/引擎需在 Tauri 环境运行（Rust 核心）。

## 状态图例

| 标记 | 含义 |
|------|------|
| ✅ Done | 已完成并通过验收 |
| 🔵 In Progress | 正在进行 |
| ⚪ Not Started | 尚未开始 |
| 🟡 Blocked | 被阻塞（注明阻塞原因） |
| ❓ Needs Verification | 需要外部事实确认后才可推进 |

`docs/feature-matrix.md` 中的 `Status` 使用同一图例的文字形式。

## 下一步

1. Phase 4：导入导出 UI 入口（文件/粘贴/复制）+ 通用导入导出框架（Codec trait）+ TXT 导入导出 + DB（SQLite）阶段。
2. 开局库 UI：走库面板（候选/推荐/命中提示）+ 自动走库开关 + 脱库步数 + 引擎回退循环。
3. Phase 3 续：评价曲线 / 自动复盘 / 人机对弈循环。
4. 开局库导入格式调研（OBK/PFBook）与云库 API 确认（NEEDS_VERIFICATION）。

## 关键开放问题（NEEDS_VERIFICATION）

详见 `docs/development-plan.md`「未知项与待确认」：

1. XQF 二进制格式的权威规格来源与字段定义。
2. 东萍棋谱格式的完整语法（变例/注释边界）。
3. 皮卡鱼云库（云库）的公开 API/协议与使用条款。
4. 截图识别本地模型的选型与权重来源/许可。
5. Tauri 2 便携版（Portable）的官方/社区标准做法。

## 变更历史

| 日期 | 变更 |
|------|------|
| 2026-08-22 | 建立架构文档集、AGENTS.md、STATUS.md、feature-matrix.md；确认 Phase 0 前状态 |
| 2026-08-22 | Phase 0 完成：Tauri 2 + React/TS + Rust 骨架、Tailwind/shadcn/ui、Zustand、测试、ESLint/Prettier/rustfmt、husky、基础 CI；提交 `chore: bootstrap project` |
| 2026-08-22 | Phase 1 棋盘核心完成：Rust 规则引擎（52 测试，perft 对拍 44/1920/79666）、FEN、校验、旋转；React 棋盘 UI + 局面编辑器；提交 `feat: implement xiangqi board core` |
| 2026-08-22 | Phase 2 棋谱树完成：真实树结构、任意节点恢复局面、React Move Tree UI；提交 `feat: implement game tree` |
| 2026-08-22 | 架构审查修复：H1 注释/NAG 按节点定位、H2 文档/会话状态边界 + tree_json 序列化、H3 快照去 parse_fen、M1 position 派生、M2 变例提升/排序、M3 attacks_square；提交 `fix: address game tree architecture review` |
| 2026-08-22 | Pikafish 引擎层完成：Engine Manager/UCI 编解码/Mock 引擎/崩溃重启/超时/分析切换局面；真实 Pikafish 冒烟通过；提交 `feat: integrate pikafish engine` |
| 2026-08-22 | 引擎分析 UI：Analysis Panel（评价/深度/NPS/PV/MultiPV）、参数面板、开始/停止/重启、PV 棋盘预览、切换局面防竞态；提交 `feat: add engine analysis interface` |
| 2026-08-22 | PGN 导入导出完成：parser/exporter（元数据/变例/嵌套变例/注释/NAG）、回合前缀分支定位 + `insert_main_at` 修复主/变例顺序、round-trip 等价与二次导出稳定测试；提交 `feat: add pgn support` |
| 2026-08-23 | 开局库基础完成：BookProvider/BookStats/BookStrategy/BookChain、LocalBookProvider（Zobrist 键 + 非法着法过滤 + JSON 持久化）、CloudBookProvider 设计占位（可降级）、book_lookup/recommend/auto_move 命令；提交 `feat: add opening book` |