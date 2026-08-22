# STATUS.md

> 本文档记录 PikaXiangqi 项目的实时状态。提交代码前先更新这里。
> 最后更新：2026-08-22

## 当前阶段

**Phase 2（Game Tree 棋谱树）✅ 核心完成**

- ✅ Rust 棋谱树 `src-tauri/src/game/`：**真正的树结构**（非着法数组）——
  Root / MoveNode（含父指针与有序子节点）/ MainLine（children[0]）/ Variation / Nested Variation /
  CurrentNode / Undo（悔棋栈）/ Redo（重做栈）/ InsertMove（合法校验 + 相同着法复用）/ DeleteVariation（整棵子树）/ Navigate /
  Comments / Annotations（NAG ! ? !! ?? !? ?! = ~）。
- ✅ **从任意节点回放父链恢复完整 Position**（`restore_position`，与节点缓存 FEN 一致性测试通过）。
- ✅ Tauri 命令：`game_new / game_snapshot / game_insert_move / game_navigate / game_previous / game_next /
  game_undo / game_redo / game_go_to_start / game_go_to_end / game_delete_variation / game_set_comment / game_set_nag`。
- ✅ React Move Tree UI：点击棋步跳转、左右键导航（←/→）、Ctrl+Z/Y 悔棋/重做、
  Variation 展开/收起、变例删除（🗑）、注释显示（*）与编辑、NAG 按钮、当前棋步高亮、嵌套变例递归渲染。
- ✅ 测试：Rust 76 个（棋盘 52 + 棋谱树集成 24）、前端 28 个（含 MoveTree 7 个、game store 6 个）。
- ⛔ 尚未实现：PGN/XQF/东萍 导入导出（Phase 2 收尾）、Pikafish 引擎（Phase 3）、开局库/OCR（Phase 4/5）。

## 验收命令（全部通过）

| 命令 | 结果 | 说明 |
|------|------|------|
| `cargo test` | ✅ | 52 lib + 24 game_tree 集成测试（沙箱需清单变通，见下） |
| `cargo clippy --all-targets -- -D warnings` | ✅ | |
| `cargo fmt --check` | ✅ | |
| `cargo check` | ✅ | |
| `npm run test` | ✅ | 28 个前端用例 |
| `npm run lint` | ✅ | 0 error 0 warning |
| `npm run format:check` | ✅ | |
| `npm run build` | ✅ | tsc + vite build |
| `npm run check` | ✅ | tsc --noEmit |

## 本机（沙箱）环境说明

- 本机未安装 MSVC Build Tools；Rust 使用 **GNU 工具链**（rustup 安装在仓库内 `.toolchain/`，已 gitignore）。
- 项目路径含中文（`D:\Codex 项目\PikaXiangqi`），GNU 工具链无法处理非 ASCII 路径。
  **变通：设置 `CARGO_TARGET_DIR` 为纯 ASCII 目录**。
- **Tauri 运行时链接与清单（仅 GNU 沙箱需要）**：
  - `#[tauri::command]` 宏展开会保留 tauri 运行时，测试二进制因此需要 comctl32 v6（`TaskDialogIndirect`）与 `WebView2Loader.dll`；
  - tauri-build 只把清单资源链接到 bin（`link-arg-bins`），lib 测试二进制默认无清单；
  - 沙箱变通：① 重命名 mingw 的 `default-manifest.o`（避免清单合并冲突）；② 运行 `cargo test` 时设置
    `RUSTFLAGS="-C link-arg=<build>\libresource.a"`（把 comctl32 v6 清单链接进测试二进制）。
  - **MSVC / CI（windows-latest）无需任何变通，`cargo test` 直接可用**。
- `npm` 由沙箱内置 pnpm 全局安装（npm 12）；正常开发机自带 npm 即可。
- 依赖镜像：本机验证使用 npmmirror（npm）与 rsproxy（crates.io）；CI 使用默认源。
- 浏览器 `npm run dev` 使用内存回退 API（仅展示起始局面）；走子/编辑/棋谱树需在 Tauri 环境运行（Rust 核心）。

## 状态图例

| 标记 | 含义 |
|------|------|
| ✅ Done | 已完成并通过验收 |
| 🔵 In Progress | 正在进行 |
| ⚪ Not Started | 尚未开始 |
| 🟡 Blocked | 被阻塞（注明阻塞原因） |
| ❓ Needs Verification | 需要外部事实确认后才可推进 |

`docs/feature-matrix.md` 中的 `Status` 使用同一图例的文字形式。

## 下一步（Phase 2 收尾 / Phase 3）

1. Phase 2 收尾：PGN / TXT 导入导出（feature-matrix #13/#15/#16），棋盘→棋谱树联动的进一步完善。
2. Phase 3：Pikafish 引擎集成（Engine Manager + UCI + MultiPV + 评价曲线）。

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
| 2026-08-22 | Phase 2 棋谱树完成：真实树结构（Root/MoveNode/主线/变例/嵌套变例/Undo/Redo/InsertMove/DeleteVariation/Navigate/注释/NAG）、任意节点恢复局面、React Move Tree UI、Rust 76 + 前端 28 测试；提交 `feat: implement game tree` |