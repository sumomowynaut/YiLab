# 引擎与 UCI 架构（Engine）

## 1. 引擎事实基线（已核实）

依据皮卡鱼官方 Wiki（`pikafish.com/wiki`）与官方仓库说明：

- Pikafish 是 **UCI** 象棋引擎，源自 Stockfish，采用 NNUE 评估。
- Pikafish **原生不支持 UCCI**（需代理转换），本项目直接使用 UCI，无需 UCCI 代理。
- 中国象棋走法采用 **UCI-Cyclone** 约定（行号从 0 开始，`position` 关键字可省略）。
- 许可：引擎代码 GPLv3；NNUE 权重另有独立许可（见 `docs/licensing.md`）。

## 2. Engine Manager 架构

职责：**进程生命周期 + UCI 编解码 + 请求串行化 + 崩溃恢复**。

```
                 Engine Manager（tokio 单任务）
┌──────────────┐   cmd channel   ┌─────────────────────────┐
│ Tauri 命令层   │ ─────────────► │  请求队列（串行）          │
│ commands.rs  │                 │  进程 spawn/handshake    │
└──────────────┘                 │  stdin 写入（命令）        │
                                 │  stdout 异步解析（info等） │
                                 │  结果分发（event / reply） │
                                 └───────────┬─────────────┘
                                             │ stdin/stdout
                                     ┌───────▼────────┐
                                     │ Pikafish.exe   │
                                     │ (+ pikafish.nnue)│
                                     └────────────────┘
```

### 2.1 生命周期

1. `spawn`：启动 Pikafish 二进制，工作目录设为引擎目录（保证默认读到 `pikafish.nnue`）。
2. `handshake`：发 `uci`，等待 `uciok`；解析 `id`/`option` 构建选项表。
3. 运行：接受 `setoption`、`isready`、`position`、`go`、`stop` 请求。
4. `shutdown`：发 `quit` 并回收进程（超时强制 kill）。

### 2.2 请求串行化

所有引擎交互经单入口队列，避免并发写 stdin 造成协议错乱：

- 「设置类」（setoption/isready/position）顺序执行；
- 「go/stop」成对管理：新 `go` 前先 `stop` 并等待当前搜索结束（或直接 `stop` 后丢弃旧结果）。

### 2.3 崩溃恢复

- 引擎进程退出/无响应 → 置状态为 `Crashed`，通过 event 通知 UI；
- UI 提供「重启引擎」；重启后自动恢复选项设置与当前局面；
- 主进程永不因引擎崩溃而退出。

## 3. UCI 架构

### 3.1 命令集（本项目使用范围）

| 命令 | 用途 |
|------|------|
| `uci` | 握手，获取 `id`/`option`，以 `uciok` 结束 |
| `isready` | 探活，应返回 `readyok` |
| `setoption name <id> [value <x>]` | 设置选项；button 类省略 value |
| `position fen <fen>` | 设局面（可省略 `position`，但本项目显式书写更清晰） |
| `position startpos` | 起始局面 |
| `position fen <fen> moves <mv1> <mv2>...` | 设局面 + 历史着法（沿棋谱树回溯时用） |
| `go ...` | 开始搜索（参数见 §3.3） |
| `stop` | 停止搜索，返回 `bestmove` |
| `quit` | 退出 |

调试命令 `d`/`eval` 不用于生产功能。

### 3.2 stdout 输出解析

需解析的 token：

- `info depth <d> seldepth <sd> multipv <i> score cp <x>|mate <m> [lowerbound|upperbound] nodes <n> nps <n> time <ms> pv <m1> <m2> ...`
- `bestmove <mv> [ponder <mv>]`

约定（本项目内部）：

- `score cp <x>`：以厘兵（centipawn）为单位的红方视角分数；`score mate <m>`：`m>0` 红方将在 m 步内杀。
- 开启 `UCI_WDLCentipawn` 时可用胜率式分值；首版统一使用 `cp`/`mate`，分数单位与展示的「红分」换算见 §5。
- 评价曲线数据按 `multipv == 1` 的主变 score 记录。

### 3.3 `go` 参数

| 参数 | 本项目用途 |
|------|-----------|
| `infinite` | 持续分析（用户点「分析」） |
| `searchmoves <m1> <m2>...` | 「变招」：排除最佳招，让引擎算次优变 |
| `depth <n>` / `movetime <ms>` | 自动复盘/定时分析 |
| `wtime/btime/winc/binc` | 皮卡鱼执红/黑走棋时的限时（可选） |
| `ponder` | 后台思考（可选，首版可不用） |

> 「变招」对应网页版语义：禁止当前最佳招 → 用 `searchmoves` 传入「除最佳招外的合法着法」，或直接对最佳招做 `searchmoves` 排除后重新 `go`。

## 4. MultiPV 架构

- 通过 `setoption name MultiPV value <n>` 启用（范围见 §5，Wiki 记 1~500）。
- 开启后同一 `depth` 会输出多条 `info ... multipv 1..n`，UI 按 `multipv` 分组展示多个候选着法。
- 决策：MultiPV 仅在用户主动开启时使用（默认 1），避免棋力下降与输出洪峰。

## 5. 引擎参数（UCI Options）

已核实的 Pikafish 常用选项（来源：皮卡鱼 Wiki「UCI 协议」「UCI 选项」）。首版只暴露下列子集，其余留作高级设置（仍可通过 `setoption` 透传）：

| 选项 | 类型 | 默认 | 说明 | 首版暴露 |
|------|------|------|------|----------|
| Threads | spin | 1 (1~1024) | 线程数 | ✅ |
| Hash | spin | 16 (1~33554432 MB) | 置换表大小 | ✅ |
| Clear Hash | button | — | 清空置换表 | ✅ |
| Ponder | check | false | 后台思考 | ✅（可选） |
| MultiPV | spin | 1 | 多主变数量（Wiki 记 1~500） | ✅ |
| Skill Level | spin | 20 (0~20) | 限棋力（人软对弈） | 🔵 高级 |
| UCI_LimitStrength | check | false | 启用 UCI_Elo | 🔵 高级 |
| UCI_Elo | spin | — (1280~3133) | 精细限棋力 | 🔵 高级 |
| Sixty Move Rule | check | on | 自然限招开关 | 🔵 高级 |
| Rule60MaxPly | spin | 120 (1~150) | 自然限招步数 | 🔵 高级 |
| Repetition Rule | combo | AsianRule 等 | 循环棋规 | 🔵 高级 |
| ScoreType | combo | Elo | 分数类型 | 🔵 高级 |
| DrawRule | combo | None | 和棋规则改写 | 🔵 高级 |
| EvalFile | string | pikafish.nnue | NNUE 权重路径 | 🔵 内部 |
| NumaPolicy | combo | auto | NUMA 策略 | ❌ 不暴露 |

> 说明：`MultiPV` 上限在 `uci` 输出示例中显示为 `max 128`，而 Wiki「UCI 选项」页写 1~500，二者不一致。**以实际引擎 `uciok` 输出的 option 定义为准**，本表默认值取 1，上限运行时动态读取，不硬编码。（`NEEDS_VERIFICATION`：确认当前发布版真实上限）

### 5.1 参数持久化

- 引擎参数存 SQLite `settings` 表；启动/重启引擎后回放 `setoption`。
- 仅保存「用户改动过」的选项；其余用引擎默认值，避免与引擎版本默认值漂移。

## 6. 分析循环

```
analysis_start(node):
    queue.stop()                      # 停当前搜索（如有）
    queue.position(fen + moves)       # 沿树回溯构造
    queue.go(infinite)                # 开始分析

on info(info):
    engineStore.push(node, info)      # UI 实时更新 + 评价曲线缓存

analysis_stop():
    queue.stop()                      # 等 bestmove，丢弃或记录
```

- 分析结果作为**瞬态 UI 状态**；「自动复盘」完成后按 `architecture.md` §5 落 `analysis` 表。

## 7. 并发与错误

- tokio：引擎 stdout 用 `BufReader::read_line` 异步读取，逐行解析，不阻塞主线程。
- 所有 `setoption`/`isready` 支持超时；超时视为引擎无响应，进入崩溃恢复。
- 引擎二进制缺失/无执行权限 → 启动失败，UI 提示配置引擎路径（支持用户自选引擎文件）。
## 8. 已实现（2026-08-22）

- `src-tauri/src/engine/`：`types`（Info/BestMove/Event/Status/GoParams/UciOption）、`uci`（命令构建 + stdout 解析，纯函数）、`manager`（EngineManager，tokio 单任务事件循环）。
- Mock 引擎 `mock_engine`（`[[bin]]`）用于集成测试；真实 Pikafish 冒烟测试 `tests/pikafish_smoke.rs`（默认 ignore，需 `PIKAFISH_BIN`/`PIKAFISH_CWD`）。
- 引擎工作目录通过 `EngineConfig.cwd` 指定，使引擎默认读到同目录 `pikafish.nnue`。
- 分发（安装包捆绑引擎/权重）受许可证决策约束，当前不实现；详见 `STATUS.md` 与 `licensing.md`。
- 引擎分析命令层（`engine_start/status/set_option/set_position_and_go/stop/restart/quit`）与事件转发（`engine://event`，`EngineEvent` camelCase 序列化）。
- `Searching` 事件作为「新分析」的边界：前端在切换局面后清空旧显示并进入 pending，收到 `Searching` 才开始接受事件，防止旧分析覆盖新局面。
- React Analysis Panel（评价/深度/节点/NPS/时间/MultiPV/PV + 参数 + 开始/停止/重启 + PV 预览），详见 `src/components/engine/AnalysisPanel.tsx` 与 `src/stores/useEngineStore.ts`。