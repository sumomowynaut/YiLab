# 测试架构（Testing）

## 1. 策略

测试金字塔：大量单元测试 + 关键集成测试 + 少量端到端测试。核心原则：**规则与棋谱树逻辑必须可离线、可确定性验证**。

```
          ▲ 少
      E2E（关键用户流程）
    ┌──────────────────┐
    │ 集成（引擎/UCI/DB/格式往返）│
  ┌────────────────────────┐
  │ 单元（走法/规则/FEN/棋谱树/前端组件）│
  └────────────────────────────┘
              ▲ 多
```

## 2. Rust 单元测试（`cargo test`）

| 模块 | 测试重点 |
|------|----------|
| `board` | 走法生成、每类棋子规则（马腿/相眼/炮架/过河兵）、合法性过滤、将帅照面、FEN 解析/序列化往返 |
| `game` | 棋谱树全部操作：make_move/add_variation/delete/promote/reorder/set_comment/truncate |
| `engine::uci` | `info` 行解析（cp/mate/multipv/lowerbound）、`bestmove`、option 解析 |
| `io` | 各格式的 parse/serialize（含错误定位） |
| `book` | 本地库查询排序、W/D/L 统计 |

### 2.1 Perft（走法生成对拍）

走法生成器必须通过已知中国象棋 perft 值校验：

- 起始局面 perft(1..N) 与权威参考值一致（N 视性能取 3~5）。
- 若干固定测试局面（含将军/禁着/蹩腿/塞眼）的 perft 值。
- `NEEDS_VERIFICATION`：确认采用哪套权威 perft 参考值（需与所选 FEN/坐标约定一致）。

### 2.2 属性测试

- 使用 `proptest`：随机合法局面 → `parse(serialize(p)) == p`；make/unmake 对称性；走法生成不产生自将。

## 3. 前端测试（Vitest + React Testing Library）

| 目标 | 测试重点 |
|------|----------|
| 组件 | 棋盘渲染与高亮、走法列表、棋谱树展开/折叠、注释编辑、设置面板 |
| Zustand store | `gameStore`/`engineStore` 的状态转移（含从 mock IPC 返回快照） |
| 工具 | 坐标换算、FEN 展示、分数格式化（cp/mate → 展示文本） |

- `lib/ipc.ts` 通过 mock 的 `invoke`/event 测试，不依赖真实 Tauri。

## 4. 集成测试

| 场景 | 方式 |
|------|------|
| Engine Manager ↔ 真实 Pikafish | 用固定 Fixture 的 `info` 输出做**离线回放**；可选对真引擎做握手/分析冒烟 |
| SQLite 迁移与读写 | 临时目录数据库，验证迁移幂等与 CRUD |
| 格式往返 | `parse(serialize(tree))` 语义等价（黄金文件基线，见 `import-export.md`） |
| Book | 注入固定库数据验证查询与排序 |

## 5. 端到端测试（E2E）

- 工具：Tauri 的 `tauri-driver`（WebDriver）+ 测试框架。
- 覆盖**关键用户流程**（而非全量）：
  1. 启动 → 默认起始局面 → 走子 → 显示合法走法。
  2. 导入 PGN → 显示主线/变例 → 导出 PGN 往返。
  3. 启动引擎 → 分析 → 显示评分与主变 → 停止。
  4. 编辑局面 → 切换先手方 → 校验 FEN。
- E2E 在 CI 的 `windows-latest` 上运行（需 GUI/WebView2），失败可标记 `allow-failure` 以免阻塞核心 CI。

## 6. 门槛与度量

- Rust：关键模块（board/game/io）行覆盖率 ≥ 80% 为目标（不强制硬性门禁，避免形式主义）。
- CI 门禁：`cargo test`、`cargo clippy -- -D warnings`、`cargo fmt --check`、`pnpm test`、`pnpm build` 全绿才可合并。
- 每个 `feature-matrix.md` 条目的 `Test` 列必须对应可执行测试，功能「Done」= 测试通过。

## 7. 与 CI 的关系

测试在 GitHub Actions 的 `ci` workflow 中执行（见 `development-plan.md` §6）。引擎真实进程测试需要下载/捆绑 Pikafish，与许可决策绑定；在许可确认前，引擎测试仅用 Fixture 离线回放。