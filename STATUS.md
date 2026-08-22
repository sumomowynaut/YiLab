# STATUS.md

> 本文档记录 PikaXiangqi 项目的实时状态。提交代码前先更新这里。
> 最后更新：2026-08-22

## 当前阶段

**Phase 0（Project Bootstrap）✅ 完成**

- ✅ 架构文档集：`docs/`（产品规格、架构、数据模型、引擎、导入导出、开局库、OCR、测试、许可、开发计划、功能矩阵）。
- ✅ 可运行项目骨架：Tauri 2 + React 19 + TypeScript + Rust + Tailwind CSS 4 + shadcn/ui（Button/Card）+ Zustand。
- ✅ 基础测试：Vitest + React Testing Library（5 个用例通过）、Rust 单元测试（2 个用例通过）。
- ✅ 工具链：ESLint 9（flat config）、Prettier、rustfmt、clippy、husky + lint-staged（pre-commit）。
- ✅ 基础 CI：`.github/workflows/ci.yml`（windows-latest，前端 + Rust 全量检查）。
- ⛔ 尚未开始：任何象棋业务功能（Phase 1+）。

## 验收命令（全部通过）

| 命令 | 结果 | 说明 |
|------|------|------|
| `npm install` | ✅ | 341 个包，含 esbuild postinstall 已批准 |
| `npm run build` | ✅ | tsc + vite build |
| `npm run test` | ✅ | 2 个测试文件 / 5 个用例 |
| `npm run lint` | ✅ | ESLint 0 error 0 warning |
| `npm run format:check` | ✅ | Prettier 全绿 |
| `cargo check` | ✅ | 见下方环境说明 |
| `cargo test` | ✅ | 2 个用例通过 |
| `cargo fmt --check` | ✅ | |
| `cargo clippy -- -D warnings` | ✅ | |

## 本机（沙箱）环境说明

- 本机未安装 MSVC Build Tools；Rust 使用 **GNU 工具链**（rustup 安装在仓库内 `.toolchain/`，已 gitignore）。
- 项目路径含中文（`D:\Codex 项目\PikaXiangqi`），GNU 工具链（dlltool/ld）无法处理非 ASCII 路径。
  **变通：设置 `CARGO_TARGET_DIR` 为纯 ASCII 目录**（如 `%TEMP%\pika-build\target`）后，`cargo check` / `cargo test` 在真实仓库内全部通过。
- 正式开发机与 CI（`windows-latest`，MSVC toolchain，ASCII 路径）无需该变通。
- `npm` 由沙箱内置 pnpm 全局安装（npm 12）；正常开发机自带 npm 即可。
- 依赖镜像：本机验证使用 npmmirror（npm）与 rsproxy（crates.io）；CI 使用默认源。

## 状态图例

| 标记 | 含义 |
|------|------|
| ✅ Done | 已完成并通过验收 |
| 🔵 In Progress | 正在进行 |
| ⚪ Not Started | 尚未开始 |
| 🟡 Blocked | 被阻塞（注明阻塞原因） |
| ❓ Needs Verification | 需要外部事实确认后才可推进 |

`docs/feature-matrix.md` 中的 `Status` 使用同一图例的文字形式。

## 下一步（进入 Phase 1）

按 `docs/development-plan.md` 的 Phase 1 实现核心棋盘与规则：棋盘渲染、走法生成与合法性、FEN、perft 对拍。

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