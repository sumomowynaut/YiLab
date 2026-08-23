# 皮卡鱼网页版功能与 UI 对标验收

> 日期：2026-08-23
> 审查性质：黑盒 + 代码对照验收（只读，不修改代码、不提交）。
> 对标基线：皮卡鱼网页版公开可用功能（以官方 Wiki 说明为准，见下），不凭记忆臆断。

## 0. 结论（TL;DR）

**最终判定：B —— 局部 UI 调整（不需要重做，更不需要大规模 UI 重做）。**

- 底层核心能力（棋盘/规则/FEN/棋谱树/变例/注释/PGN/引擎分析/MultiPV/自动复盘/OCR/GIF/深浅色/快捷键/保存载入）**已实现且测试全绿**。
- 与「皮卡鱼网页版功能基线」的主要差距不在架构，而在：**开箱引擎路径缺失、新建棋局入口缺失、人机对弈循环缺失、云库查询本体缺失、XQF/东萍/TXT 格式缺失、开局库无数据且无导入入口**。
- 其中真正影响 V1 可用性的是 **P0（引擎开箱不可用）** 与 **P1（新建棋局入口 / 开局库状态误导 / 人机对弈）**，均为「补入口 + 补发现逻辑 + 补状态区分」，不涉及核心架构重构。

---

## 1. 验收方法与可信度

1. **代码对照**：通读 `src/App.tsx`、`src/stores/*`、`src/components/*`、`src-tauri/src/commands.rs`、`src-tauri/src/engine/*`、`src-tauri/src/book/*`、`src-tauri/src/io/*`、`lib.rs`（命令注册表）、`docs/feature-matrix.md`、`STATUS.md`。
2. **测试基线**：依据 `STATUS.md` 记录，Rust 单元+集成测试与前端 vitest 全绿（棋盘规则 52 用例 + perft 44/1920/79666、Game Tree 24、UCI/引擎、Book、PGN、OCR、GIF、自动复盘、前端 100+ 用例）。
3. **网页版基线来源（可查证）**：皮卡鱼 Wiki《皮卡鱼网页版说明》`https://www.pikafish.com/wiki/index.php?title=皮卡鱼网页版说明`。
4. **限制声明**：当前会话无法自动执行真机 GUI 逐项点击（鼠标/视觉断言）。凡依赖「确切点击位置、视觉细节、真机引擎二进制存在性」的项，标注 `NEEDS_VERIFICATION` 或「建议人工复核」，不臆断。

---

## 2. 皮卡鱼网页版功能基线（来自官方 Wiki，可查证）

- 新建棋局（还原起始局面 + 清除当前棋局）
- 皮卡鱼执红 / 执黑、皮卡鱼分析、立即出招、变招（排除最佳着法的 searchmoves）
- 翻转棋盘（换边视角 / 左右对称）
- 导入：文件上传（pgn / xqf / 东萍）、棋盘图片识别（OCR，可手动摆棋修正）、粘贴文本棋谱或 FEN
- 复制 / 下载：复制局面（FEN）、复制局面链接、复制文字棋谱、复制东萍棋谱、复制棋谱链接、下载 XQF 棋谱、生成 GIF
- 编辑局面（摆棋、切换红方先行 / 黑方先行）
- 设置：置换表大小（Hash）、线程数、走棋条件、脱库步数（配合云库自动走棋）
- 导航：上个变招 / 开局、上一步（平替悔棋）、下一步、下个变招 / 终局
- 展示：深度、红分（网页版 50% 胜率 ≈ 100 分）、NPS
- 云库（自动走棋 / 脱库步数 / 走库招法）
- 注释：棋谱注释显示 + 可编辑
- 打谱：保存到本地、打开本地棋谱
- 变招标记「变」、注释标记「*」、点变招切换、显示变招数 / 注释数、「^ / ⅴ」调变招顺序、「🗑」删除变招

---

## 3. 黑盒 + 代码对照测试（22 项）

结果词汇：`PASS` / `FAIL` / `PARTIAL` / `MISSING` / `NEEDS_VERIFICATION`

| # | 测试项 | 结果 | 依据 / 说明 |
|---|--------|------|-------------|
| 1 | 新建棋盘 / 新对局 | **FAIL**（用户流程）| 后端 `game_new(fen)` 已存在；`useGameStore` 无独立 reset/new 动作；UI 无「新建棋局」按钮。结论 `FUNCTION EXISTS / UI ENTRY MISSING`（见 §4-A）。 |
| 2 | 摆棋 | **PASS** | 「编辑局面」→ PiecePalette 摆放 + 清空棋盘 + 切换先手方；`board_edit_*` 命令已注册。 |
| 3 | 正常走棋 | **PASS** | Board 点击选中 → 合法着法高亮 → `game_insert_move`；规则合法性与 FEN 由 Rust 单一事实源保证。 |
| 4 | 悔棋 / 重做 | **PASS** | 顶部「↶ / ↷」按钮 + `Ctrl+Z / Ctrl+Y`；`game_undo/game_redo`。 |
| 5 | 导航棋谱 | **PASS** | 「⏮ / ← / → / ⏭」+ Home/End 快捷键 + MoveTree 点击跳转。 |
| 6 | 添加变例 | **PASS** | 在非主线分支走子即生成变例；MoveTree 展开/收起，支持嵌套变例。 |
| 7 | 注释 / NAG | **PASS** | 注释 textarea（显式 node_id 写入）+ NAG 按钮（! ? !! ?? !? ?!）；着法带注释标「*」。 |
| 8 | 导入 FEN | **PASS** | 棋谱页 FEN 输入框「载入」+ 导入导出页 FEN 自动嗅探。 |
| 9 | 导入 PGN | **PASS** | `GameCodec` 粘贴/文件导入；Rust PGN parser（metadata/moves/变例/注释/NAG）。 |
| 10 | 导出棋谱 | **PARTIAL** | PGN / FEN 导出（复制 + 下载）已实现；**XQF、东萍、TXT 均未实现**。 |
| 11 | 启动 Pikafish | **FAIL** | 需手动填 `settings.programPath` 或设 `PIKAFISH_BIN` 环境变量；UI 无引擎 exe 自动发现/预填（见 §4-B，Root Cause）。 |
| 12 | 正常分析 | **PARTIAL** | 引擎路径正确时可用（AnalysisPanel 展示 Evaluation/Depth/Nodes/NPS/Time/MultiPV/PV）；但受 #11 阻塞，开箱不可用。 |
| 13 | 切换 MultiPV | **PASS** | 设置页 MultiPV 1/2/3/5/10 + AnalysisPanel 多行渲染。 |
| 14 | 停止分析 | **PASS** | AnalysisPanel Stop + `engine_stop`。 |
| 15 | 自动复盘 | **PASS** | AnalysisReport：逐着分析、评价曲线点击跳转、停止/继续/重新分析、Best~Blunder 分类（阈值可配置）。 |
| 16 | 开局库 | **PARTIAL** | Provider/BookChain/`book_lookup` 命令已实现；但**无内置数据、无导入入口**，且「无数据」被 UI 表现为「按钮禁用」（见 §4-C）。 |
| 17 | 截图识别 | **PASS** | OcrPanel：传统 CV 模板识别 + 置信度/问题提示 + 载入后走本地规则校验 + 手动修正。 |
| 18 | GIF 导出 | **PASS** | GifExportPanel：当前局面/主线/指定变例 + 帧间隔/尺寸/坐标/棋步。 |
| 19 | 保存 / 载入棋局 | **PASS** | 「保存棋局 / 载入棋局」按钮（`game_save/game_load`，应用数据目录 `current-game.json`）。 |
| 20 | 深浅色模式 | **PASS** | 设置页切换 + localStorage 持久化 + 跟随系统偏好。 |
| 21 | 快捷键 | **PASS** | ←/→、Home/End、F/M、Space、Ctrl+Z/Y；设置页有说明清单。 |
| 22 | 窗口缩放 | **PARTIAL** | 响应式布局已实现（`max-w-7xl` + `xl` 断点 + `flex-wrap` + `min-w-0`）；视觉细节建议人工复核。 |

> 注：#1/#11/#16 是本次验收重点，详见 §4。

---

## 4. 三个重点问题深挖

### A. 为什么没有明显的「新建棋盘 / 新对局」

- **后端**：存在。`game_new(fen)` 命令已注册（`commands.rs`，空 fen 用起始局面）；`board_edit_clear_all` 也能清空棋盘。
- **GameStore**：`loadFen` 调 `api.newGame(fen)`，`toggleEditing` 退出编辑时用 `editPosition.fen` 调 `newGame`。**但没有任何独立的 `reset/newGame` 动作**。
- **UI**：棋谱标签页只有「保存棋局 / 载入棋局」+ FEN 输入框；**没有「新建棋局 / 新对局」按钮**。
- **判定**：`FUNCTION EXISTS / UI ENTRY MISSING`。这是**入口缺失**，不是功能缺失。属于 **P1**（用户打开后不知道如何开始一盘新棋）。

### B. 引擎显示「未指定引擎程序路径」的 Root Cause

- **代码事实**：`engine_start(program, app)` 当 `program` 为空时回退 `std::env::var("PIKAFISH_BIN")`，再没有则报错「未指定引擎程序路径」。
- **前端**：`useEngineStore.start()` 传 `settings.programPath`，默认值 `""`；设置页「引擎路径」输入框默认空（placeholder 仅提示「留空使用 PIKAFISH_BIN」）。
- **为什么真实 smoke test 能过**：测试注入/设置了 `PIKAFISH_BIN` 环境变量；**普通用户 UI 既不设该环境变量、也没填路径**，因此必然启动失败。
- **关键误区澄清**：现有 `discover_eval_file(program)` 只自动发现 **NNUE 权重 `pikafish.nnue`**（在 exe 同目录/上一级），**并不自动发现引擎 exe 本身**。
- **Root Cause**：**引擎二进制路径本身没有被 UI 自动发现或预填**，而非 NNUE 权重问题。设置页虽有引擎路径输入框，但默认空且无「浏览/自动发现」提示，用户不知道填什么。
- **严重度**：**P0**。最终目标包含「本地 Pikafish」，开箱无法启动引擎 = 核心流程不可用。

### C. 开局库标签 / 按钮为什么点不动

- **代码事实**：`BookProvider` / `BookChain` / `LocalBookProvider` / `CloudBookProvider` 均已实现；`book_lookup` / `book_recommend` / `book_auto_move` 均已注册为 Tauri 命令；前端 `BookPanel` 正确调用 `useBookStore`。
- **关键区分（必须写清）**：
  - **「开局库没有数据」**：`LocalBookProvider` 是**内存空 map**，无内置数据，`lookup` 对起始局面返回空 → 状态 `empty`（UI 显示「未命中」）。这是**数据缺失**。
  - **「自动走库按钮点不动」**：`BookPanel` 的「自动走库」按钮 `disabled={status !== "hit"}`，无数据时 status 是 `empty`，所以按钮禁用。这是**「无数据」被 UI 表现为「按钮不可用」**，用户容易误以为功能坏了。
  - 同类问题：「指定变例」无变例时禁用、GIF 等也属「无数据=禁用」。
- **用户能否导入/加载开局库**：**不能**。没有「导入 PFBook / OBK」入口；导入格式本身仍 `NEEDS_VERIFICATION`。
- **判定**：功能本身存在（`FEATURE EXISTS`），但「无数据」与「功能不可用」界面混同 + 无可导入入口。**PARTIAL + UI_MISMATCH**，建议 **P1**。

---

## 5. 功能对标表（皮卡鱼网页版 vs PikaXiangqi）

Status 只使用：`MATCH` / `PARTIAL` / `MISSING` / `UI_MISMATCH` / `NEEDS_VERIFICATION`

| Feature | 皮卡鱼网页版 | PikaXiangqi | Status | Gap |
|---------|-------------|-------------|--------|-----|
| 首页 / 工作区布局 | 单页工作区：棋盘 + 操作栏 + 棋谱/注释 | 单页：棋盘（左）+ 标签页（右） | UI_MISMATCH | 棋盘仍是视觉中心，但核心操作被拆到「分析/开局库/导入导出/设置」标签页，入口位置不同 |
| 新建棋局 | 显式「新建棋局」按钮 | 无独立入口（后端 `game_new` 有） | UI_MISMATCH | FUNCTION EXISTS / UI ENTRY MISSING（P1） |
| 棋盘 | 可走子/翻转/镜像 | 可走子/翻转(180°)/镜像 | MATCH | — |
| 棋谱（主线/导航） | 上个变招/开局、上一步、下一步、下个变招/终局 | ⏮/←/→/⏭ + Home/End + 点击跳转 | MATCH | 语义等价 |
| 变例 | 增/删/调序（^/ⅴ）/切换/变招标记「变」 | 增/删/提升为主线/上移下移/展开 | MATCH | M2 已补齐 |
| 注释 | 注释显示 + 可编辑 + 标记「*」 | 注释 textarea + NAG + 标记「*」 | MATCH | — |
| FEN | 粘贴 FEN / 复制局面(FEN) | 粘贴 FEN / 导出 FEN | MATCH | 复制局面(FEN)等价于导出 FEN |
| PGN | 文件导入 / 复制文字棋谱 | PGN 导入/导出 | MATCH | — |
| 引擎（本地 Pikafish） | 网页版为服务端引擎，开箱即用 | 本地 Pikafish，需手动配路径 | MISSING（开箱） | **无引擎 exe 自动发现/预填（P0）** |
| MultiPV | 多主变展示 | 1/2/3/5/10 | MATCH | — |
| 分析（depth/score/nps） | 深度、红分(50%≈100)、NPS | Evaluation/Depth/Nodes/NPS/Time/PV | PARTIAL | 分数口径（网页版 50%≈100 分）未做网页版映射，属 P2 视觉/口径差异 |
| 评价曲线 | 无明确公开项（或并入分析） | 会话内评价曲线 + 点击跳转 | PARTIAL | 项目已有，超出网页版公开说明，定位为增强项 |
| 自动复盘 | 网页版公开说明未列完整复盘报告 | 完整逐着复盘 + 分类 + 曲线 | PARTIAL | 项目已有（增强项），网页版是否等价 `NEEDS_VERIFICATION` |
| 开局库 | 云库自动走棋 + 脱库步数 + 走库招法 | 本地空库 + 自动走库命令 + 脱库步数 | PARTIAL | 无数据、无导入入口；「无数据」与「不可用」混同（P1） |
| 云库 | 有（自动走棋/脱库） | CloudBookProvider 为占位，不发起网络请求 | MISSING | 查询本体未实现（`NEEDS_VERIFICATION`，API 未确认） |
| 截图识别（OCR） | 棋盘图片识别 + 手动摆棋修正 | 传统 CV 识别 + 置信度 + 手动修正 | MATCH | 视觉模型质量差异属 P2 |
| GIF | 生成 GIF | 当前局面/主线/指定变例 + 参数 | MATCH | — |
| 设置 | 置换表/线程/走棋条件/脱库步数 | 线程/哈希/深度/MultiPV/主题/引擎路径 | PARTIAL | 缺「走棋条件」「脱库步数」UI 入口（脱库步数后端已有，UI 未暴露） |
| 快捷键 | 未在 Wiki 明确列示 | ←/→/Home/End/F/M/Space/Ctrl+Z/Y + 设置页说明 | MATCH | 项目能力超出网页版公开说明 |
| 导入导出 | pgn/xqf/东萍/FEN/文本 + 复制链接 | PGN/FEN（粘贴/文件/复制/下载） | PARTIAL | XQF/东萍/TXT 缺失（`NEEDS_VERIFICATION`） |
| 深色模式 | 未在 Wiki 明确列示 | 深浅色 + 跟随系统 | MATCH | 项目能力 |
| **人机对弈**（皮卡鱼执红/黑、立即出招、变招 searchmoves） | **有** | **完全没有对弈循环** | **MISSING** | 网页版核心工作流，当前缺失（P1） |
| 复制局面/棋谱链接 | 有 | 无 | MISSING | 网页版公开功能，本项目无（P2，取决于定位） |
| 打谱（保存/打开本地） | 保存到本地/打开本地 | 保存棋局/载入棋局（current-game.json） | PARTIAL | 已有单存档最小实现；多对局管理/文件对话框未做 |

---

## 6. 用户操作流程对标

**皮卡鱼网页版（目标工作流）**：

```
打开 → 新建/载入棋局 → 棋盘（走子/摆棋） → 棋谱（主线/变例/注释）
     → 引擎分析/皮卡鱼执红黑/立即出招 → 操作（保存/导入导出/GIF/OCR）
```

**PikaXiangqi（当前实际工作流）**：

```
打开（默认起始局面） → 棋盘 → 「棋谱」标签（走子/变例/注释/FEN/保存载入）
     → 「分析」标签（引擎 Start + MultiPV + 自动复盘）
     → 「开局库」标签 → 「导入导出」标签 → 「设置」标签
```

**差异定性**：

- 底层能力基本一一对应，但**入口与操作路径不同**：网页版把「新建/引擎执红黑/立即出招/分析」放在工作区显眼位置，PikaXiangqi 把分析藏在「分析」标签、引擎启动藏在 AnalysisPanel。
- 网页版默认围绕「对弈 vs 引擎」组织；PikaXiangqi 默认是「复盘工具」，**没有对弈循环**。
- 因此：`UI_MISMATCH` 主要集中在「新建棋局入口」「引擎/对弈入口」，不是视觉风格问题。

---

## 7. P0 / P1 问题清单

> P2/P3（分数口径、视觉细节、复制链接、视觉模型质量）本轮不动。

### P0（功能缺失 / 核心流程不可用）

1. **引擎开箱不可用**：无引擎 exe 自动发现/预填，用户必须手填 `programPath` 或设 `PIKAFISH_BIN`。→ 补 `discover_engine`（exe 同目录/上一级/`engine/` 目录）并预填到设置页；首次启动引导选择引擎路径。

### P1（功能存在但 UI/交互严重偏离，或重要功能缺失）

2. **新建棋局入口缺失**：暴露 `game_new(START_FEN)` 为「新建棋局」按钮（顶部工具栏 + 棋谱页）。
3. **人机对弈循环缺失**：皮卡鱼执红/黑、立即出招、变招（searchmoves）——网页版核心工作流。
4. **开局库状态误导**：区分「无数据（empty，可解释 + 引导导入）」与「功能不可用」；提供开局库导入入口。
5. **云库查询本体缺失**：待 API 确认后实现（`NEEDS_VERIFICATION`），当前保持占位 + 降级。
6. **XQF / 东萍 / TXT 导入导出缺失**：先格式调研（`NEEDS_VERIFICATION`），再按 PGN 既有框架接入。

---

## 8. 最终结论（回答用户的 7 问 + A/B/C）

1. **距离「皮卡鱼网页版功能基线」还有多远？** 底层核心能力（棋盘/规则/FEN/棋谱树/变例/注释/PGN/引擎分析/MultiPV/自动复盘/OCR/GIF/深浅色/快捷键/保存）已基本达标；主要差距在「开箱引擎路径」「新建棋局入口」「人机对弈循环」「云库查询本体」「XQF/东萍/TXT」「开局库数据与导入」。属于**功能与入口补齐**，不是架构差距。
2. **重做 UI 还是补入口/调整布局？** **补入口 + 调整布局**。核心组件（Board/MoveTree/AnalysisPanel/BookPanel/GameCodec/OcrPanel/GifExportPanel/SettingsPanel）已连接真实功能，无需重做。
3. **哪些功能已存在、只是 UI 没暴露？** 新建棋局（`game_new`）、开局库查询/推荐/自动走库（命令已注册）、脱库步数（后端 `recommend_book`/`book_auto_move(max_plies)` 已有，UI 未暴露脱库步数控件）、引擎参数集中设置（已迁移到设置页）。
4. **哪些功能真正缺失？** 引擎 exe 自动发现/预填、人机对弈循环（执红黑/立即出招/变招）、云库查询本体、开局库数据与导入入口、XQF/东萍/TXT、复制局面/棋谱链接（P2）。
5. **哪些属于启动/环境问题？** `npm.ps1` 被 PowerShell 执行策略拦截（需 `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` 或改用 `npm.cmd`）；Vite IPv6/IPv4、GNU 中文路径（已修复）——均属**开发环境问题**，非产品 Bug。引擎 `PIKAFISH_BIN` 依赖属于**产品层面**（缺默认发现），不是环境问题。
6. **哪些属于产品设计问题？** A（新建入口缺失）、B（引擎路径无自动发现/无默认提示）、C（开局库「无数据」与「不可用」混同）。
7. **哪些属于代码 Bug？** 核心模块（Board/Game Tree/Engine Manager/UCI/Book/Import-Export/OCR/GIF）审查**未发现 Critical/High 功能性 Bug**，测试全绿；已知 Medium（`from_tree_json` 对非法棋谱可能 panic）已随 B3 修复（改用 `apply_move` 校验）。

**三选一：B —— 局部 UI 调整。**

---

## 9. 具体修改页面 / 组件 + 优先级

| 优先级 | 页面/组件 | 修改内容 |
|--------|-----------|----------|
| P0 | `SettingsPanel.tsx` + `engine_start`（Rust） | 新增 `discover_engine` 自动发现引擎 exe（exe 同目录/上一级/`engine/`），启动时预填 `programPath`；找不到时引导用户浏览选择 |
| P0 | `App.tsx` 顶部工具栏 | 新增「新建棋局」按钮 → `game_new(START_FEN)` |
| P1 | `App.tsx` + 新组件 `PlayPanel` | 人机对弈：皮卡鱼执红/黑、立即出招、变招（searchmoves 排除最佳） |
| P1 | `BookPanel.tsx` | 状态区分：`empty` 显示「当前无开局库数据，可导入」并提供导入入口；`自动走库` 在无数据时改为「可点击但提示无数据」，不误导为功能坏 |
| P1 | `BookPanel.tsx` / `设置` | 暴露「脱库步数」UI（后端已有 `max_plies`） |
| P1 | `GameCodec.tsx` + `io/`（Rust） | 新增 XQF / 东萍 / TXT 适配器（先格式调研，`NEEDS_VERIFICATION`） |
| P1 | `cloud.rs` | 待云库 API 确认后实现查询本体（保持 `BookChain` 降级路径） |

> 以上均为「补入口 + 补发现逻辑 + 补状态区分 + 补格式适配器」，不重构 Board Core / Game Tree / Engine Manager / UCI / Book Provider / Import-Export Core 的既有正确架构。
