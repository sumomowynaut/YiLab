# 开发计划（Development Plan）

## 1. 阶段划分原则

- 每个 Phase 有明确目标、交付物与**验收标准（exit criteria）**，达到即可进入下一 Phase。
- 前一个 Phase 的交付物是后一个 Phase 的依赖，避免并行导致的返工。
- 不做跳跃式开发；未确认的格式/协议在对应 Phase 内用 `NEEDS_VERIFICATION` 先做调研 spike。

## 2. Phase 总览

| Phase | 主题 | 覆盖功能（feature-matrix） | 验收标准 |
|-------|------|---------------------------|----------|
| 0 | 工程初始化 | 30（CI 骨架） | 可运行的空窗口 + CI 全绿 |
| 1 | 核心棋盘与规则 | 1, 2, 3, 25, 26 | perft 通过；FEN 往返一致；可走子 |
| 2 | 棋谱与变例 | 4, 5, 6, 7, 8, 13, 15, 16 | 棋谱树操作完整；PGN/TXT 往返 |
| 3 | 引擎集成 | 9, 10, 11, 12, 22 | 引擎分析/停止、MultiPV、参数、评价曲线可用 |
| 4 | 格式与开局库 | 14, 17, 19, 20, 21 | XQF/东萍导入导出；本地库/云库/自动走库 |
| 5 | 智能分析 | 18, 23, 24, 27 | OCR、自动复盘、GIF、快捷键可用 |
| 6 | 发布与分发 | 28, 29, 30, 31 | 产出 installer + portable + Release |

## 3. 各 Phase 详情

### Phase 0 — 工程初始化

- 交付：`git init`；Tauri 2 + Vite + React + TS 脚手架；Tailwind + shadcn/ui；Zustand；Rust 模块骨架（board/game/engine/io/book/ocr/db）；SQLite 连接与迁移框架；CI 骨架（test workflow）；lint/format 配置。
- 验收：`pnpm dev` 打开空窗口；`cargo test`/`npm run test`/`cargo clippy`/`fmt` 通过；CI 在 windows-latest 通过。

### Phase 1 — 核心棋盘与规则

- 交付：坐标/棋子模型；走法生成与合法性；FEN 解析/序列化；起始局面渲染；基本深浅色主题。
- 验收：perft 对拍通过；FEN 往返一致；UI 可点击走合法子、非法子被拒。

### Phase 2 — 棋谱与变例

- 交付：棋谱树 + 主线/变例/注释/NAG；局面编辑器；导入导出框架 + FEN/PGN/TXT；走法列表与棋谱树 UI。
- 验收：树操作（增/删/提/换序/注释）单测 + E2E；PGN/TXT 往返语义等价。

### Phase 3 — 引擎集成

- 交付：Engine Manager；UCI 编解码；MultiPV；引擎参数面板；分析循环；评价曲线（主变分数序列）。
- 验收：真机启动 Pikafish，分析/停止正常；MultiPV 多条主变展示；参数 setoption 生效；崩溃恢复可用。

### Phase 4 — 格式与开局库

- 交付：XQF、东萍适配器；本地开局库导入/查询；云库 Provider（可降级）+ 缓存；自动走库（含脱库步数）。
- 验收：XQF/东萍 往返；本地库查询命中；云库失败回退本地；自动走库按脱库步数停止。

### Phase 5 — 智能分析

- 交付：截图识别（传统 CV 最低实现 + 人工校正）；自动复盘（整局批量分析 + 落库 + 逐着点评）；GIF 导出；全局快捷键。
- 验收：OCR 输出可校正局面；自动复盘生成评价曲线与点评；GIF 可播放；快捷键清单可用。

### Phase 6 — 发布与分发

- 交付：NSIS 安装版；便携版；WebView2 引导；引擎/权重捆绑方案落地（依许可决策）；Release workflow；GitHub Release 发布。
- 验收：安装包在干净 Win10/11 可安装运行；便携版解压即用；Release 页面产出正确产物。

## 4. 里程碑

| 里程碑 | 对应 Phase 完成 | 标志 |
|--------|----------------|------|
| M0 可运行骨架 | Phase 0 | 空窗口 + CI |
| M1 能下棋 | Phase 1 | 完整棋盘 + 合法走棋 |
| M2 能复盘 | Phase 2 | 棋谱树 + 变例 + 注释 + PGN/TXT |
| M3 能分析 | Phase 3 | 本地引擎 + MultiPV + 评价曲线 |
| M4 生态互通 | Phase 4 | XQF/东萍 + 开局库 |
| M5 智能增强 | Phase 5 | OCR + 自动复盘 + GIF + 快捷键 |
| M6 可发布 | Phase 6 | Installer + Portable + Release |

## 5. 依赖与阻塞

- **Release 阶段被许可决策阻塞**（见 `licensing.md` §3）：不阻塞开发，但阻塞 M6 的公开分发。
- XQF/东萍/云库/OCR 模型/便携版方案存在 `NEEDS_VERIFICATION`，在对应 Phase 先做调研 spike。

## 6. GitHub CI/CD 详细设计

### 6.1 `ci.yml`（push / PR）

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: npm }
      - run: npm ci
      - run: cargo test --manifest-path src-tauri/Cargo.toml
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
      - run: cargo fmt --manifest-path src-tauri/Cargo.toml --check
      - run: npm run test
      - run: npm run build
```

### 6.2 `release.yml`（tag `v*`）

```yaml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: npm }
      - run: npm ci
      # 许可确认后：在此拉取/校验 Pikafish 二进制与 NNUE 权重到 engine/
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'PikaXiangqi ${{ github.ref_name }}'
          releaseDraft: true
          prerelease: false
```

### 6.3 产物与矩阵

- 单一 `windows-latest`（首版仅 Windows；未来跨平台不预建矩阵，避免过度工程）。
- 产物：NSIS 安装器（`.exe`）、MSI（可选）、便携 zip。
- 版本号：以 Git tag 为准；`tauri.conf.json` 的版本由 tag 驱动。

## 7. 风险登记

| 风险 | 影响 | 缓解 |
|------|------|------|
| NNUE 权重商用限制 | 阻塞公开/商业化分发 | 许可决策前置；备选 CC0 权重（`NEEDS_VERIFICATION`） |
| XQF/东萍规格不明确 | 导入导出进度受阻 | Phase 4 前做调研 spike + 样本库 |
| 云库 API 不公开/不稳定 | 云库功能不可用 | 云库可降级；本地库兜底 |
| OCR 识别率低 | 截图功能价值受限 | 人工校正兜底；定位为可降级增强 |
| 引擎崩溃/兼容性 | 分析中断 | Engine Manager 崩溃恢复 + 用户可选引擎 |
| WebView2 缺失（Win10） | 安装后无法启动 | NSIS 内置 WebView2 bootstrapper |

## 8. 未知项与待确认（NEEDS_VERIFICATION 汇总）

1. XQF 二进制格式权威规格与字段定义。
2. 东萍棋谱格式完整语法（变例嵌套、注释定界）。
3. 皮卡鱼云库公开 API/协议与使用条款。
4. 截图识别本地模型选型、识别率与权重许可。
5. Tauri 2 便携版（Portable）官方/社区标准做法。
6. 中国象棋 PGN 方言与主流软件兼容性。
7. TXT 纵线制 ↔ UCI 坐标换算边界。
8. MultiPV 上限（Wiki 1~500 与 uci 示例 max 128 不一致）。
9. CC0 替代权重（Fairy-Stockfish 象棋变体）的真实性、棋力与版本匹配。
10. 权威 perft 参考值（须与所选坐标/FEN 约定一致）。
11. 本地开局库导入格式（OBK/PFBook）规格与授权。
12. 代码签名证书获取流程与资质要求。

## 9. 首版范围建议（防过度工程）

以下明确**不做**（除非后续用户明确要求）：
- 联机对弈、账号、云同步、社区分享。
- 开局库编辑器/合并器。
- 完整象棋循环棋规裁决器（长将/长捉判定交给引擎或仅提示）。
- 跨平台打包（macOS/Linux）矩阵。
- 插件系统、自定义主题引擎。