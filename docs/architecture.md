# 技术架构（Architecture）

本文档定义 PikaXiangqi 的技术架构，覆盖：技术选型、分层与模块、数据流、数据库、UI、Windows 打包、CI/CD。数据模型见 `game-model.md`，引擎/UCI 见 `engine.md`。

## 1. 技术选型（已定）

| 层 | 选型 | 理由 |
|----|------|------|
| 桌面壳 | Tauri 2 | 小体积、Rust 后端、Windows 原生打包（NSIS/MSI） |
| 前端 | React + TypeScript | 生态成熟，strict 类型 |
| 状态 | Zustand | 轻量、无样板、适合命令式更新 |
| 样式 | Tailwind CSS + shadcn/ui | 快速构建、主题变量天然支持深浅色 |
| 持久化 | SQLite（经 Rust 侧 `rusqlite`） | 本地文件、单库、无服务进程 |
| 核心逻辑 | Rust | 规则/棋谱树/引擎/导入导出/开局库/OCR/DB 的单一事实来源 |
| 引擎 | Pikafish（外部二进制，UCI） | 最强开源象棋引擎，源自 Stockfish |
| 异步 | tokio（Rust 侧） | 引擎进程 IO、云库请求等 |

> 决策：**领域核心全部放 Rust**，React 只做展示与交互投影。理由见 §4。

## 2. 分层与模块

```
React UI（src/）
  ├─ components/      棋盘、棋谱树、走法列表、评价栏、设置面板、对话框
  ├─ stores/          Zustand：game / engine / settings / ui
  ├─ hooks/           键盘、主题、选区等
  └─ lib/ipc.ts       对 Tauri invoke 的类型化封装

        │ Tauri IPC（invoke，类型化 command）
        ▼

Rust Core（src-tauri/src/）
  ├─ commands.rs       Tauri 命令层（薄封装，只做参数校验与转调）
  ├─ board/           坐标、棋子、走法生成、合法性、FEN、将军/胜负/和棋
  ├─ game/            棋谱树：主线/变例/注释/NAG/导航
  ├─ engine/          Engine Manager + UCI 编解码 + 选项
  ├─ book/            BookProvider trait + 本地/云库实现
  ├─ io/              Import/Export trait + FEN/PGN/XQF/TXT/东萍适配器
  ├─ ocr/             截图识别管线
  └─ db/              SQLite（rusqlite + 迁移）

        │ 进程通信（stdin/stdout，异步）
        ▼
Pikafish（外部进程，UCI 引擎 + pikafish.nnue）
```

## 3. 数据流

### 3.1 用户落子 / 导航

```
UI 点击 → invoke("game_make_move", {mv}) → Rust 校验合法 → 更新棋谱树 → 返回新树快照 → UI 渲染
```

### 3.2 引擎分析

```
UI 切换局面 → invoke("analysis_start") → Engine Manager 发 stop → 设 position → 发 go infinite
Engine stdout → UCI 解码 → info 流 → 通过 Tauri event 推送到 UI → 评价栏/走法列表实时更新
UI 停止 → invoke("analysis_stop") → 发 stop
```

### 3.3 持久化

```
棋谱树/设置/开局库缓存 → Rust db 模块 → SQLite 文件（应用数据目录）
```

## 4. 关键决策：单一事实来源

**决策**：`board` 与 `game` 的权威状态在 Rust；React/Zustand 只保存轻量 UI 状态（当前选中节点、主题、正在拖拽等）与引擎分析结果的**瞬态镜像**。

理由：
- 合法性校验、FEN、棋谱树操作是高频复用的核心逻辑，放在 Rust 可单元测试且与 UI 解耦。
- 棋谱树编辑是低频用户动作（点击/拖拽），跨 IPC 开销可忽略。
- 持久化直接对 Rust 侧权威对象进行，避免「UI 状态 ↔ 存储」二次同步。

代价：每次树操作返回一份快照。快照只序列化当前可见范围（棋盘 + 走法列表 + 树结构摘要），不做全树深拷贝，控制 IPC 载荷。

## 5. 数据库架构（SQLite）

单文件数据库 `pikaxiangqi.db`，位于 Tauri 应用数据目录。使用 `rusqlite`，启动时跑迁移。

### 5.1 表设计（不提前过度规范化）

```sql
-- 棋谱：元数据 + 整棵树 JSON 快照 + 检索用的索引列
CREATE TABLE games (
  id            TEXT PRIMARY KEY,          -- UUID
  title         TEXT NOT NULL DEFAULT '',
  red_player    TEXT NOT NULL DEFAULT '',
  black_player  TEXT NOT NULL DEFAULT '',
  event         TEXT NOT NULL DEFAULT '',
  date          TEXT NOT NULL DEFAULT '',
  result        TEXT NOT NULL DEFAULT '*', -- 1-0 / 0-1 / 1/2-1/2 / *
  root_fen      TEXT NOT NULL,             -- 起始局面 FEN（默认 startpos）
  tree_json     TEXT NOT NULL,             -- 棋谱树序列化（含变例/注释/NAG）
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

-- 设置（键值对）
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- 本地开局库缓存（book 模块，见 book.md）
CREATE TABLE book_entries (
  pos_key INTEGER PRIMARY KEY,  -- Zobrist 哈希
  fen     TEXT NOT NULL,
  moves   TEXT NOT NULL,        -- JSON: [{uci, wins, draws, losses}]
  source  TEXT NOT NULL         -- 'local' | 'cloud-cache'
);

-- 复盘/分析结果缓存（评价曲线数据）
CREATE TABLE analysis (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id     TEXT NOT NULL,
  node_key    TEXT NOT NULL,    -- 棋谱树节点标识
  depth       INTEGER NOT NULL,
  score       INTEGER,          -- 分值（cp 或 mate，见 engine.md 约定）
  score_kind  TEXT NOT NULL,    -- 'cp' | 'mate'
  pv          TEXT,             -- 主变 UCI 序列
  nps         INTEGER,
  created_at  INTEGER NOT NULL
);
CREATE INDEX idx_analysis_game ON analysis(game_id, node_key);
```

> 说明：`games.tree_json` 直接序列化整棵棋谱树，避免为变例/注释做复杂的关系表——棋谱树是「文档」，不是高频关系查询对象；元数据列保留在关系列便于列表检索。

### 5.2 迁移

- 迁移脚本以递增序号管理，随应用启动执行，记录 `PRAGMA user_version`。
- 首版仅需要 `v1`；不预建复杂迁移框架。

## 6. UI 架构

### 6.1 布局（桌面单窗口，左棋盘 + 右面板）

```
┌───────────────────────────┬─────────────────────────┐
│                           │  标签页：分析 / 棋谱 / 开局库 │
│      棋盘（可翻转/镜像）      │  评价栏 + 深度/nps          │
│      走法箭头/选中高亮        │  走法列表（主线/变例）       │
│                           │  棋谱树（Game Tree 视图）   │
│      底部：导航/编辑工具栏      │  注释编辑区               │
└───────────────────────────┴─────────────────────────┘
```

### 6.2 组件与状态

| Zustand store | 职责 | 数据来源 |
|---------------|------|----------|
| `gameStore` | 当前棋谱树快照、选中节点、主线/变例导航 | `invoke` 命令返回值 |
| `engineStore` | 引擎状态、实时 info 行、MultiPV 结果、评价曲线缓存 | Tauri event 推送 |
| `settingsStore` | 引擎参数、主题、快捷键、开局库配置 | 启动时读取 + 持久化命令 |
| `uiStore` | 面板显隐、拖拽状态、当前标签页 | 纯前端 |

- 棋盘用 SVG 渲染（棋子用 Unicode 汉字 + 圆形，或内置字型），坐标映射集中在 `board/` 的 Rust 与前端共享约定。
- 主题：CSS 变量 + Tailwind `dark:` 策略；`shadcn/ui` 自带深色令牌。
- 快捷键：前端全局监听 + Rust 命令映射（导航、分析启停、翻转等），见 `feature-matrix.md`。

### 6.3 IPC 命令面（示例，最终以实现为准）

`game_make_move` / `game_navigate` / `game_add_variation` / `game_delete_variation` / `game_promote_variation` / `game_reorder_variation` / `game_set_comment` / `position_edit` / `fen_parse` / `analysis_start` / `analysis_stop` / `engine_set_option` / `book_lookup` / `io_import` / `io_export` / `ocr_recognize`。

## 7. Windows 打包架构

### 7.1 产物

| 产物 | 技术 | 备注 |
|------|------|------|
| 安装版 | Tauri 2 打包器，NSIS 或 MSI | 首推 NSIS（Tauri 默认，体积小） |
| 便携版 | 打包后应用目录打包为 zip/自解压 | `NEEDS_VERIFICATION`：Tauri 2 无内置 portable 目标，需确认官方/社区标准做法 |

### 7.2 WebView2 依赖

Tauri 在 Windows 依赖 WebView2 Runtime（Win11 已内置，Win10 需引导安装）。NSIS 安装脚本需包含 WebView2 bootstrapper（Tauri 已内置支持），或明确要求用户预装。

### 7.3 引擎捆绑

- 安装版在 `engine/` 目录捆绑 Pikafish Windows 二进制 + `pikafish.nnue`。
- 引擎路径通过 `EvalFile` 选项或启动工作目录解析；不写死绝对路径，支持用户后续替换引擎二进制。
- 许可影响见 `docs/licensing.md`（GPLv3 + NNUE 非商用条款），发布前必须完成许可决策。

### 7.4 代码签名

- 首版可免签名（Windows SmartScreen 会警告）；正式分发建议 EV/OV 代码签名。签名工具链与证书获取流程 `NEEDS_VERIFICATION`（涉及组织资质）。

## 8. GitHub CI/CD 架构

细节与工作流 YAML 见 `docs/development-plan.md` §6。概览：

```
push/PR ──► CI（test）：cargo test + clippy + fmt + vitest + tauri build（debug 冒烟）
push tag v* ──► Release：矩阵构建 Windows → tauri-action 生成 installer + portable → 上传 Release
```

- 单一 `windows-latest` runner（首版仅 Windows）。
- 使用 `tauri-apps/tauri-action` 生成并发布产物。
- 引擎二进制与 NNUE 权重不作为源码提交；Release 流水线在许可确认后从可信来源拉取并打包（或随仓库 submodule/asset 管理，方案待许可决策后定）。

## 9. 横切关注点

- **错误处理**：命令层返回结构化错误码（`Result<T, AppError>`），UI 统一展示；引擎崩溃走独立恢复流程（见 `engine.md`）。
- **日志**：Rust 侧 `tracing`，写入应用数据目录日志文件；前端控制台日志不进入生产构建。
- **配置**：引擎参数、主题、快捷键等存 SQLite `settings`；启动即载入。
- **安全**：Tauri 采用最小 capability，仅暴露白名单命令；不启用不必要的系统权限。
- **并发**：引擎进程 IO 跑在 tokio 独立任务，与 Tauri 命令线程隔离；所有对引擎的请求经 Engine Manager 单入口串行化（见 `engine.md`）。