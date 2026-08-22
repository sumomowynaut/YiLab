# STATUS.md

> 本文档记录 PikaXiangqi 项目的实时状态。提交代码前先更新这里。
> 最后更新：2026-08-22

## 当前阶段

**Phase 2（Game Tree 棋谱树）✅ 核心完成 + 架构审查修复完成**

- ✅ Rust 棋谱树 `src-tauri/src/game/`：真实树结构（Root / MoveNode / MainLine / Variation / Nested Variation / Undo / Redo / InsertMove / DeleteVariation / Navigate / Comments / NAG / Promote / Reorder / 文档序列化）。
- ✅ 从任意节点回放父链恢复完整 Position（`restore_position`，与缓存 FEN 一致性测试通过）。
- ✅ React Move Tree UI：点击跳转、←/→ 导航、Ctrl+Z/Y 悔棋/重做、变例展开/删除/**提升主线/上移/下移**、注释显示与编辑、NAG、当前棋步高亮。
- ✅ 测试：Rust 83（棋盘 53 + 棋谱树集成 30）、前端 30。

### 架构审查修复（本轮，commit `fix: address game tree architecture review`）

**已修复**

| 项 | 内容 |
|----|------|
| H1 | 注释/NAG 修改改为**按 node_id 显式定位**（`game_set_comment(node_id, ...)` / `game_set_nag(node_id, ...)`，Rust `set_comment_at`/`set_nag_at`），不再依赖全局 current；附回归测试（节点 A 写注释→导航到 B→注释仍在 A） |
| H2 | 明确 **Document State**（startpos/root/nodes/headers）与 **Session State**（current/redo_stack）边界；新增 `game::serialize`（tree_json v1）**只序列化文档字段**，导入时校验结构并重置会话状态；往返测试证明 current/redo_stack 不进入持久化 |
| H3 | `GameNode` 插入时缓存 `side_to_move`/`fullmove_number`，快照 `build_node` 不再逐节点 `parse_fen`；一致性测试保证缓存与局面相符 |
| M1 | store 非编辑态 position 由 `snapshot.position` 派生（`selectDisplayPosition`），编辑态使用独立 `editPosition`；clearAll/toggleSide 限定编辑态 |
| M2 | 实现 `promote_variation` / `reorder_variation`（Rust + 命令 + React UI 提升/上移/下移 + 测试） |
| M3 | 新增 `attacks_square`（免临时 Vec 分配）并接入 `is_attacked`；等价性测试保证与走法生成一致；**perft 44/1920/79666 全部保持通过** |

**暂缓/记录（不强行实现）**

- `GameTree` 拆分为 `GameSession { tree, current, redo_stack }`：当前改动最小方案是保留同结构 + 明确文档/会话边界 + 序列化排除会话字段；拆分记为后续重构项。
- 全树快照 → 增量/可见范围快照：当前棋谱规模小，暂缓；已在 `docs/architecture.md` 记录。
- `truncate`：非本轮必要范围，记录为后续 Phase（导入导出时）。
- store 进一步拆分（`useGameStore`/`useBoardStore` 分离）：记为后续重构项。
- `apply_move`（`legal_moves().contains`）分配优化：收益低、风险高于收益，记录为后续性能项。

## 验收命令（全部通过）

| 命令 | 结果 | 说明 |
|------|------|------|
| `cargo test` | ✅ | 53 lib + 30 game_tree 集成测试（沙箱需清单变通，见下） |
| `cargo clippy --all-targets -- -D warnings` | ✅ | |
| `cargo fmt --check` | ✅ | |
| `cargo check` | ✅ | |
| `npm run test` | ✅ | 30 个前端用例 |
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

1. Phase 2 收尾：PGN / TXT 导入导出（feature-matrix #13/#15/#16）；`truncate` 随导入导出落地。
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
| 2026-08-22 | Phase 2 棋谱树完成：真实树结构、任意节点恢复局面、React Move Tree UI；提交 `feat: implement game tree` |
| 2026-08-22 | 架构审查修复：H1 注释/NAG 按节点定位、H2 文档/会话状态边界 + tree_json 序列化、H3 快照去 parse_fen、M1 position 派生、M2 变例提升/排序、M3 attacks_square；提交 `fix: address game tree architecture review` |