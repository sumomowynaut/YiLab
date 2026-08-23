# STATUS.md

> 本文档记录 PikaXiangqi 项目的实时状态。提交代码前先更新这里。
> 最后更新：2026-08-23

## 当前阶段

**GIF 导出 ✅ 完成（2026-08-23）**

- ✅ `src-tauri/src/gif_export.rs`：`GifRequest`（startpos + moves + 帧间隔/棋盘尺寸/坐标/棋步）+ `export_gif` → GIF 字节。
- ✅ 来源：**当前局面**（单帧）/ **主线**（startpos + 主线全部着法）/ **指定变例**（到分支点 + 变例着法）。
- ✅ 支持：**帧间隔**（毫秒 → 厘秒）、**棋盘尺寸**（格子像素）、**显示坐标**（a-i / 0-9）、**显示棋步**（最后一步高亮 + 标注）。
- ✅ 渲染复用 `ocr::render`（同一棋盘/棋子渲染），叠加坐标/高亮/标注；字库扩展数字 0-9 与小写 a-i。
- ✅ 编码：`gif` crate + 固定调色板就近量化（无损），`Repeat::Infinite` 循环。
- ✅ Tauri 命令：`gif_export_current` / `gif_export_mainline` / `gif_export_variation`。
- ✅ 前端 `GifExportPanel`：来源/变例选择 + 帧间隔/尺寸/坐标/棋步 + 导出下载（Blob）。
- ✅ 测试：Rust 单元 4（单帧/多帧/延迟取整/错误路径）+ 集成 3（主线/单帧/变例，GIF 回读校验帧数/尺寸/延迟）+ 命令层树遍历 2 + 前端 5。
- ⚪ 棋子图形为程序生成的字母圆盘（无中文字体）；真实棋子图形渲染留待后续。

**自动复盘（Automatic Game Analysis）✅ 完成（2026-08-23）**

- ✅ `src-tauri/src/analysis.rs`：`AutoAnalyzer`（tokio 异步运行器，**不阻塞 UI**）+ `MoveAssessment`。
- ✅ 对主线每一步记录：实际着法 / 最佳着法 / 走前评价 / 走后评价 / **评价损失** / 深度 / PV。
- ✅ **分类阈值可配置**（`ClassificationConfig`，不硬编码）：Best / Excellent / Good / Inaccuracy / Mistake / Blunder，
  按「走子方视角前后评价差」分类。
- ✅ 效率：n 步棋只需 n+1 次有限深度搜索（局面 i 的搜索同时是第 i-1 步的走后评价与第 i 步的走前评价）。
- ✅ 支持 停止（暂停）/ 继续 / 重新分析：单一持久运行任务 + `Notify` 唤醒，无并发重复。
- ✅ 事件流：`analysis://event`（StatusChanged / Progress / Assessment / Finished）→ 前端 store 实时更新。
- ✅ 前端 `AnalysisReport`：**评价曲线（点击跳转棋步）** + 汇总（关键失误 / 最佳着法 / 评价变化表 / PV）+ 开始/停止/继续/重新分析。
- ✅ Tauri 命令：`analysis_start` / `analysis_stop` / `analysis_continue` / `analysis_status`；mock 引擎扩展为按局面确定性出分。
- ✅ 测试：Rust 单元 3 + 集成 3（完整分析 / 停止继续 / 进度事件）+ 前端 store 4 + 报告 5。
- ⚪ 「落库」（评估结果持久化）随 DB（SQLite）阶段；`NEEDS_VERIFICATION`：真实 Pikafish 分数视角（红方 vs 行棋方）需冒烟核实。

**截图识别（Screenshot Recognition / OCR）✅ 完成（2026-08-23）**

- ✅ `src-tauri/src/ocr/`：`OcrEngine` trait（视觉模型抽象）+ `TemplateRecognizer`（传统 CV，确定性模板匹配）+ 合成截图生成器 `render`。
- ✅ 识别能力：棋盘检测（底色包围盒）、90 格切分、棋子分类（16 类 + 空）、**方向判定**（正立 vs 旋转 180° 模板，Flipped180 = 整图真实旋转）、行棋方（静态截图不可判断 → None + 提示）。
- ✅ 输出结构化 Position/FEN；**视觉模型只识别，棋规校验由本地 Rust 完成**（`board::validate::validate_position`）。
- ✅ 不确定处理：低置信度格**置空并标记**（不静默接受），输出整体置信度 + `issues`（行棋方未知/不确定格/规则校验），`valid` 标志；前端 OcrPanel 展示并「载入棋谱」走人工修正（现有局面编辑器）。
- ✅ Tauri 命令 `ocr_recognize`（`image: Vec<u8>` → DTO）。
- ✅ 测试：Rust 单元 8 + 集成 7（起始局面正立/翻转、自定义局面、空棋盘、缺将、损坏图/无棋盘/过小图错误路径）+ 前端 4。
- ⚪ 真实模型（ONNX 等）选型与权重许可：`NEEDS_VERIFICATION`（docs/ocr.md §3）；后台线程化/批量识别留待后续。

**Web Feature Parity ✅ 已完成可确认项（2026-08-23）**

逐项核对 Feature Matrix，按优先级完成全部「可确认」功能（每项独立 test/build/commit）：

- ✅ #8 棋谱注释 / #10 UCI / #11 MultiPV：核对为已实现 → `Done`。
- ✅ #13 棋谱导入导出：Codec trait（FEN/PGN + 自动嗅探）+ 粘贴/文件导入 + 复制/下载导出 UI（无新依赖）。
- ✅ #25/#26 深浅色模式：主题 store + 切换按钮 + localStorage 持久化 + 跟随系统偏好。
- ✅ #27 快捷键：可配置清单 `src/lib/shortcuts.ts`（←/→、Home/End、F/M、Space、Ctrl+Z/Y）+ `useShortcuts` hook。
- ✅ #21 自动走库：脱库步数（半回合门控，`recommend_book` + `book_auto_move(max_plies)`）+ `GameTree::current_plies`。
- ✅ #12 引擎参数：设置持久化到 localStorage（重启恢复、损坏回退默认）。
- ✅ #22 评价曲线：会话内主变分数曲线（红方视角 cp，multipv=1）+ SVG 图表。
- ✅ #30 GitHub Actions：CI 配置校验（YAML 合法）+ clippy 对齐 `--all-targets`；真实 GitHub 运行与 Release 流水线待外部确认/Phase 6。
- ⚪ 未能确认/暂缓项：见本文件「需要人工确认 / 后续阶段」清单与 `docs/development-plan.md`「未知项」。

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
| `cargo test` | ✅ | 114 lib + 10 engine_manager + 30 game_tree + 3 analysis + 3 gif_export + 3 io_codec + 7 ocr + 10 opening_book + 8 pgn_roundtrip（+1 pikafish 冒烟默认 ignore；沙箱需清单变通） |
| `cargo clippy --all-targets -- -D warnings` | ✅ | |
| `cargo fmt --check` | ✅ | |
| `cargo check` | ✅ | |
| `npm run test` | ✅ | 93 个前端用例 |
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

1. **需要人工确认（NEEDS_VERIFICATION）**：#14 XQF 规格、#16 TXT 纵线制换算、#17 东萍语法、#18 OCR 模型与权重、#19 开局库导入格式（OBK/PFBook）、#20 云库 API、#29 Tauri 2 便携方案。
2. Phase 4：DB（SQLite）阶段——开局库/设置/分析结果落库、自动复盘（#23）依赖落库。
3. Phase 3 续：自动复盘（#23）、GIF 导出（#24，独立渲染管线）、人机对弈循环。
4. Phase 6：打包（#28 Installer / #31 Release）——受许可决策阻塞，发布前必须完成 `docs/licensing.md` 决策。

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
| 2026-08-23 | Web Feature Parity：核对已实现功能（#8 注释/#10 UCI/#11 MultiPV → Done）；实现 #13 导入导出框架（Codec trait + 嗅探 + 粘贴/文件导入 + 复制/下载导出 UI）；提交 `docs: reconcile…` 与 `feat: add import/export framework` |
| 2026-08-23 | Web Feature Parity（续）：#25/#26 深浅色主题、#27 可配置快捷键、#21 脱库步数、#12 引擎参数持久化、#22 评价曲线、#30 CI 对齐；每项独立提交 |
| 2026-08-23 | 截图识别完成：OcrEngine trait + TemplateRecognizer（传统 CV 模板匹配）+ 方向判定 + 合成截图生成器；识别只识别、棋规校验在本地 Rust（validate_position）；低置信度置空标记 + 置信度/问题清单 + OcrPanel 人工校正；提交 `feat: add screenshot recognition` |
| 2026-08-23 | 自动复盘完成：AutoAnalyzer 异步运行器（n+1 次搜索）、可配置分类阈值（Best~Blunder）、着法/最佳/评价变化/深度/PV、评价曲线点击跳转、停止/继续/重新分析、mock 引擎确定性出分；提交 `feat: add automatic game analysis` |
| 2026-08-23 | GIF 导出完成：当前局面/主线/指定变例来源、帧间隔/棋盘尺寸/坐标/棋步选项、gif crate 编码（固定调色板）、字库扩展数字/小写字母；提交 `feat: add gif export` |