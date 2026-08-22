# PikaXiangqi

现代化、开源、本地优先的 **Windows 桌面中国象棋复盘与 AI 分析软件**。

- 技术栈：Tauri 2 · React · TypeScript · Rust · SQLite · Zustand · Tailwind CSS · shadcn/ui
- 引擎：本地 Pikafish（UCI 象棋引擎）
- 文档：`docs/` 目录（产品规格、架构、数据模型、引擎、导入导出、开局库、OCR、测试、许可、开发计划、功能矩阵）

## 当前状态

Phase 0 项目骨架（可运行的空窗口 + CI）。详见 `STATUS.md`。

## 开发

前置要求：Node.js ≥ 20、Rust（Windows 建议 MSVC toolchain）。

```bash
npm install
npm run dev        # 浏览器/tauri dev 前端的 Vite 开发服务器
npm run tauri dev  # 以桌面应用方式运行
```

## 常用命令

```bash
npm run build        # 前端构建（tsc + vite build）
npm run test         # Vitest 单元测试
npm run lint         # ESLint
npm run format       # Prettier 格式化
npm run format:check # Prettier 检查

cargo test                            # Rust 测试
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

## CI / 发布

- `.github/workflows/ci.yml`：push/PR 全量检查（Windows）。
- GitHub Release 流水线在 Phase 6 落地（见 `docs/development-plan.md`）。

## 许可

引擎与 NNUE 权重存在独立的许可约束，发布前必须完成决策，详见 `docs/licensing.md`（非法律意见）。