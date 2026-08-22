# STATUS.md

> 本文档记录 PikaXiangqi 项目的实时状态。提交代码前先更新这里。
> 最后更新：2026-08-22

## 当前阶段

**Phase 1（Board Core 棋盘核心）✅ 棋盘核心完成**

- ✅ Rust 棋盘核心 `src-tauri/src/board/`：类型（Color/Piece/Square/Move/Position）、规则引擎
  （走法生成/合法性/将军/将死/困毙/飞将/perft）、FEN 解析与序列化、局面校验、180°旋转与左右镜像。
- ✅ Rust 单元测试 **52 个**全部通过，覆盖：将军、将军应对、将死、吃子、特殊规则（无升变/飞将/困毙）、
  炮（炮架）、马腿、象眼、士（九宫）、将（九宫）、车、兵/卒、河界、九宫、FEN、局面校验、旋转/镜像、坐标、perft。
- ✅ perft 对拍：起始局面 perft(1)=44 / perft(2)=1,920 / perft(3)=79,666（参考 Chess Programming Wiki）。
- ✅ Tauri 命令：`board_startpos / board_from_fen / board_legal_moves / board_make_move / board_validate /
  board_rotate / board_edit_set_piece / board_edit_clear / board_edit_set_side / board_edit_clear_all`。
- ✅ React Board UI：SVG 棋盘（10×9、九宫斜线、楚河汉界）、棋子渲染、选中与合法落点提示、
  走子（经 Rust 校验）、翻转棋盘/左右镜像视图、局面编辑器（棋子面板/橡皮/清空/切换先手方/FEN 载入）、校验结果展示。
- ✅ 前端测试 **15 个**全部通过（notation、Board 组件、App、utils）。
- ⛔ 尚未实现：棋谱树/变例（Phase 2）、Pikafish 引擎（Phase 3）、开局库/OCR（Phase 4/5）等。

## 验收命令（全部通过）

| 命令 | 结果 | 说明 |
|------|------|------|
| `cargo test` | ✅ | 52 个 Rust 用例 |
| `cargo clippy -- -D warnings` | ✅ | |
| `cargo fmt --check` | ✅ | |
| `cargo check` | ✅ | |
| `npm run test` | ✅ | 15 个前端用例 |
| `npm run lint` | ✅ | 0 error 0 warning |
| `npm run format:check` | ✅ | |
| `npm run build` | ✅ | tsc + vite build |
| `npm run check` | ✅ | tsc --noEmit |

## 本机（沙箱）环境说明

- 本机未安装 MSVC Build Tools；Rust 使用 **GNU 工具链**（rustup 安装在仓库内 `.toolchain/`，已 gitignore）。
- 项目路径含中文（`D:\Codex 项目\PikaXiangqi`），GNU 工具链（dlltool/ld）无法处理非 ASCII 路径。
  **变通：设置 `CARGO_TARGET_DIR` 为纯 ASCII 目录**（如 `%TEMP%\pika-build\target`）后，`cargo check` / `cargo test` 在真实仓库内全部通过。
- 正式开发机与 CI（`windows-latest`，MSVC toolchain，ASCII 路径）无需该变通。
- `npm` 由沙箱内置 pnpm 全局安装（npm 12）；正常开发机自带 npm 即可。
- 依赖镜像：本机验证使用 npmmirror（npm）与 rsproxy（crates.io）；CI 使用默认源。
- 浏览器 `npm run dev` 使用内存回退 API（仅展示起始局面）；走子/编辑需在 Tauri 环境运行（Rust 规则核心）。

## 状态图例

| 标记 | 含义 |
|------|------|
| ✅ Done | 已完成并通过验收 |
| 🔵 In Progress | 正在进行 |
| ⚪ Not Started | 尚未开始 |
| 🟡 Blocked | 被阻塞（注明阻塞原因） |
| ❓ Needs Verification | 需要外部事实确认后才可推进 |

`docs/feature-matrix.md` 中的 `Status` 使用同一图例的文字形式。

## 下一步（Phase 1 收尾 / Phase 2）

1. Phase 1 收尾：深浅色主题切换（feature-matrix #25/#26）。
2. Phase 2：棋谱树 Game Tree（主线/变例/注释）+ PGN/TXT 导入导出 + 局面编辑器完善。

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