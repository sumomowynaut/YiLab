# AGENTS.md

本文件是 AI 编码代理（及人类协作者）在本仓库工作时的最高优先级上下文。开始任何工作前请先阅读本文件、`STATUS.md` 与 `docs/` 目录下的架构文档。

## 项目定位

PikaXiangqi 是一个现代化、开源、本地优先的 Windows 桌面中国象棋复盘与 AI 分析软件。

- 技术栈（已定）：Tauri 2 / React / TypeScript / Rust / SQLite / Zustand / Tailwind CSS / shadcn/ui。
- 引擎：本地 Pikafish（UCI 象棋引擎，源自 Stockfish），通过 UCI 协议通信。
- 当前阶段：**仅规划/架构阶段，尚未开始编写业务代码**（见 `STATUS.md`）。

## 角色约定

当被要求作为「Principal Software Architect」工作时，产出应是决策、模型、接口与风险分析，而不是实现代码。数据模型的类型/结构草图与协议示例属于架构交付物，可以写；但不要开始实现业务逻辑。

## 必须遵守的原则

1. **不要过度工程化**：只为当前已确认的需求设计；不为臆想的未来需求预建抽象层。
2. **稳定 > 可测试 > 可维护 > 可扩展**：这是优先级顺序，设计取舍时按此排序。
3. **本地优先**：核心功能（棋盘、走法、复盘、引擎、开局库）必须离线可用；云库/OCR 等联网能力是可降级的增强项。
4. **单一事实来源**：局面与规则、走法合法性、棋谱树由 Rust Core 持有；React UI 是其投影。
5. **事实可溯源**：涉及皮卡鱼、协议、格式、许可的结论必须基于可查证来源。凡无法确认的事项，一律标记 `NEEDS_VERIFICATION`，不得凭记忆臆断。
6. **许可只做风险分析**：不替项目所有者判断 Pikafish 或 NNUE 权重是否可按某种方式分发。相关结论见 `docs/licensing.md`。

## 仓库结构（规划）

```
PikaXiangqi/
├── AGENTS.md
├── STATUS.md
├── docs/                  # 全部架构与规格文档
├── src/                   # React + TypeScript UI（Phase 0 起）
├── src-tauri/             # Tauri 2 + Rust Core（Phase 0 起）
│   └── src/
│       ├── board/         # 棋盘、走法生成、合法性、FEN
│       ├── game/          # 棋谱树、主线、变例、注释
│       ├── engine/        # Engine Manager、UCI
│       ├── book/          # 开局库 Provider
│       ├── io/            # 导入导出
│       ├── ocr/           # 截图识别
│       ├── db/            # SQLite
│       └── commands.rs    # Tauri IPC 命令层
├── .github/workflows/     # CI/CD（Phase 0 起）
└── engine/                # 捆绑的 Pikafish 二进制 + NNUE 权重（见 licensing）
```

## 文档索引

| 文档 | 内容 |
|------|------|
| `docs/product-spec.md` | 产品定位、功能基线（对照皮卡鱼网页版）、产品架构 |
| `docs/architecture.md` | 技术架构、模块划分、数据库、UI、打包、CI/CD 概览 |
| `docs/game-model.md` | Position 与 Game Tree 数据模型、规则、FEN |
| `docs/engine.md` | Engine Manager 与 UCI 架构、引擎参数 |
| `docs/import-export.md` | 导入导出架构、各格式（FEN/PGN/XQF/TXT/东萍） |
| `docs/book.md` | 本地开局库 / 云库 / 自动走库 |
| `docs/ocr.md` | 截图识别架构 |
| `docs/testing.md` | 测试架构与策略 |
| `docs/licensing.md` | 许可风险分析（非法律意见） |
| `docs/development-plan.md` | Phase 拆分、里程碑、打包与 CI/CD 细节 |
| `docs/feature-matrix.md` | 31 项目标功能的可验收拆解表 |

## 工作约定

- 文档与注释使用中文；代码标识符使用英文。
- TypeScript 开启 strict 模式；Rust 通过 `cargo clippy` 与 `cargo fmt --check`。
- 每完成一项功能，同步更新 `docs/feature-matrix.md` 的 `Status` 与 `STATUS.md`。
- 新引入的未确认假设必须在文档中以 `NEEDS_VERIFICATION` 标注，并在 `docs/development-plan.md` 的「未知项」清单登记。
- 不要在被要求「停止/不要开始开发」时继续写业务代码。

## 常用命令（Phase 0 起可用）

> Phase 0 已建立脚手架，以下命令当前全部可用。

```powershell
# 前端
npm install
npm run dev
npm run test          # vitest
npm run build

# Rust
cargo test         # 单元 + 集成测试
cargo clippy -- -D warnings
cargo fmt --check

# 打包
npm run tauri build   # 生成 installer / portable
```

## 决策记录

架构决策直接写入对应文档（不另设 ADR 目录，避免过度工程）。若一项决策影响多个文档，以 `docs/architecture.md` 为权威，并在其他文档中引用。