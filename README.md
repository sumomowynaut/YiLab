# 弈研 YiLab

现代化、开源、本地优先的 **Windows 桌面中国象棋复盘与 AI 分析软件**。
<img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/edc4813f-ff55-48a5-8b0d-e736317c87ef" />


## 功能

- 完整中国象棋棋盘、合法走棋、FEN 导入导出
- 棋谱树：主线 / 变例 / 多层分支 / 注释 / NAG / 悔棋重做
- 本地 Pikafish 引擎分析（UCI）：MultiPV、线程/哈希/深度设置、持续分析
- 评价曲线（悬停查看每步着法与评分）
- 自动复盘：逐手评估、Best/Excellent/Good/Inaccuracy/Mistake/Blunder 分类
- PGN / FEN 导入导出
- 截图识别（本地，离线可用）
- 说明：截图识别为本地模板识别，受字体/光线/角度影响，**个别字可能识别不准**；识别后会标出不确定棋子，供手动修正
- GIF 导出（当前局面 / 主线 / 变例）
- 深浅色模式、快捷键
- Windows 安装版与免安装版
- <img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/683e6910-452b-4b65-b818-8c6ad5757364" />
- <img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/d46d4268-dcc9-4c40-8b3c-01e86321462e" />
<img width="500" height="500" alt="800" src="https://github.com/user-attachments/assets/08aa5f2e-baee-4b31-8ec1-ea4a2b5f7ec9" />



## 技术栈

Tauri 2 · React · TypeScript · Rust · SQLite · Zustand · Tailwind CSS · shadcn/ui

引擎：本地 [Pikafish](https://github.com/official-pikafish/Pikafish)（UCI 象棋引擎），需要你自己准备引擎程序路径。

## 文档

`docs/` 目录：产品规格、架构、数据模型、引擎、导入导出、开局库、OCR、测试、许可、开发计划、功能矩阵。

## 开发

前置要求：Node.js ≥ 20、Rust。

```bash
npm install
npm run tauri dev   # 以桌面应用方式运行
```

## 常用命令

```bash
npm run build        # 前端构建（tsc + vite build）
npm run test         # Vitest 单元测试
npm run lint         # ESLint
npm run format       # Prettier 格式化

cargo test                               # Rust 测试
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

## 打包 Windows 版

```bash
npm run tauri build        # 生成安装版（联网时会自动下载 NSIS/WiX）
npm run tauri build -- --no-bundle   # 仅生成免安装 exe
```

免安装版运行时需要把 `WebView2Loader.dll` 放在 exe 同目录。

## CI

`.github/workflows/ci.yml`：push / PR 时在 Windows 上运行全量检查。

## 许可

引擎（Pikafish）与 NNUE 权重存在独立的许可约束；本软件不内置引擎二进制，引擎由用户自行提供。发布/分发前请阅读 `docs/licensing.md`（非法律意见）。
