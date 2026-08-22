# 功能矩阵（Feature Matrix）

> 本表把所有目标功能拆成可验收条目。每项包含：Feature、Description、Priority、Phase、Status、Test、Dependencies、Notes。

## 图例

- **Priority**：P0 必须 / P1 重要 / P2 增强。
- **Phase**：0~6（见 `development-plan.md` §2）。
- **Status**：`Not Started` / `In Progress` / `Done` / `Blocked` / `Needs Verification`。当前（规划阶段）全部为 `Not Started`。

## 矩阵

| # | Feature | Description | Priority | Phase | Status | Test | Dependencies | Notes |
|---|---------|-------------|----------|-------|--------|------|--------------|-------|
| 1 | 完整中国象棋棋盘 | 渲染 9×10 棋盘与 32 枚棋子，支持选中、走法高亮、翻转(180°)与左右镜像 | P0 | 1 | Done | Board 组件渲染测试（3 用例） | Phase 0 UI 脚手架 | SVG 渲染，视图翻转/镜像；数据不改变 |
| 2 | 合法走棋 | 走法生成 + 合法性校验（马腿/相眼/炮架/过河兵/将帅照面/应将） | P0 | 1 | Done | Rust 规则 52 用例 + perft(1..3)=44/1920/79666 | 棋盘数据模型 | Rust 原生实现，不依赖引擎 |
| 3 | FEN | 解析与序列化中国象棋 FEN | P0 | 1 | Done | 往返一致 + 错误样例测试 | 棋盘数据模型 | 起始局面见 `game-model.md` |
| 4 | 局面编辑 | 摆棋/清空/切换先手方并生成 FEN | P1 | 2 | Done | 编辑器组件 + Rust 命令测试 | FEN、棋盘模型 | 提前于 Phase 2 完成；含规则校验提示 |
| 5 | Game Tree | 棋谱树数据模型与持久化 | P0 | 2 | Done | Rust 24 集成测试 + 前端 store/MoveTree 测试 | 棋盘数据模型 | 真实树结构；JSON 持久化待 Phase 4（DB） |
| 6 | 主线 | 沿第一子节点走主线，主线导航 | P0 | 2 | Done | main_line 测试 + ←/→ 导航测试 | Game Tree | children[0] 为主线 |
| 7 | 多变例 | 增/删/提/换序变例，切换显示 | P0 | 2 | Done | 变例/嵌套变例测试 + MoveTree 展开/删除测试 | Game Tree | 增/删/嵌套已实现；提/换序待 Phase 4 |
| 8 | 棋谱注释 | 节点注释与 NAG（?! 等）编辑 | P1 | 2 | Not Started | 注释编辑测试 | Game Tree | 有注释着法标记「*」 |
| 9 | 本地 Pikafish | 定位/捆绑本地引擎二进制并启动 | P0 | 3 | Not Started | 引擎 spawn/握手集成测试 | Engine Manager、许可决策 | 工作目录解析 NNUE |
| 10 | UCI | UCI 协议实现（命令编解码 + info 解析） | P0 | 3 | Not Started | UCI 解析 Fixture 测试 | Engine Manager | UCI-Cyclone 坐标 |
| 11 | MultiPV | 多主变分析展示 | P1 | 3 | Not Started | MultiPV 集成测试 | UCI、引擎参数 | 默认关闭，上限运行时读取 |
| 12 | 引擎参数 | Threads/Hash/MultiPV 等选项读写与持久化 | P1 | 3 | Not Started | setoption 往返测试 | UCI | 高级选项透传 |
| 13 | 棋谱导入导出 | 通用导入导出框架 + 文件/粘贴/复制入口 | P0 | 2 | Not Started | 框架 + 往返测试 | 各格式适配器 | FEN/PGN/TXT 先行 |
| 14 | XQF | XQF 二进制导入导出 | P1 | 4 | Not Started | 往返 + 样本库 | 格式调研（NEEDS_VERIFICATION） | 先导出后导入 |
| 15 | PGN | 中国象棋 PGN 导入导出（变例/注释） | P0 | 2 | Done | 往返语义等价（8 集成 + 7 单元用例） | 导入导出框架 | UCI-Cyclone 记谱；中文纵线制导入不支持（NEEDS_VERIFICATION） |
| 16 | TXT | 中文纵线制文本棋谱导入导出 | P1 | 2 | Not Started | 往返 + 换算边界 | 导入导出框架 | 线性为主 |
| 17 | 东萍棋谱 | 东萍格式导入导出（支持变例） | P1 | 4 | Not Started | 往返 + 样本库 | 格式调研（NEEDS_VERIFICATION） | 语法待确认 |
| 18 | 截图识别 | 棋盘图片识别局面（可人工校正） | P2 | 5 | Not Started | OCR 结构 + 校正 E2E | OCR 管线 | 可降级 |
| 19 | 本地开局库 | 离线开局库导入与查询 | P1 | 4 | In Progress | 查询/排序/过滤/JSON 往返单测 | Book Provider、格式调研 | Provider/查询/推荐已实现（内存+JSON）；导入格式待确认（NEEDS_VERIFICATION），SQLite 存储随 DB 阶段 |
| 20 | 云库 | 皮卡鱼云库查询（W/D/L，可降级） | P2 | 4 | In Progress | 云库 mock + 回退测试 | 云库 API（NEEDS_VERIFICATION） | 接口与降级路径已实现并测试；查询本体待 API 确认 |
| 21 | 自动走库 | 命中走库 + 脱库步数控制 | P2 | 4 | In Progress | 走库链路单测 | 本地/云库 | Rust 侧走库原语（book_auto_move）已实现；脱库步数与引擎回退循环待 UI 阶段 |
| 22 | 评价曲线 | 主变分数随回合曲线图 | P1 | 3 | Not Started | 分数序列计算 + 图表渲染 | 引擎 info、analysis 表 | 评分可持久化 |
| 23 | 自动复盘 | 整局批量分析 + 逐着点评 + 落库 | P1 | 5 | Not Started | 批量分析流程 E2E | 引擎、评价曲线、DB | 异步执行 |
| 24 | GIF | 导出棋局动态图 | P2 | 5 | Not Started | GIF 产出可播放测试 | UI 渲染、导出 | 独立渲染管线 |
| 25 | 深色模式 | 深色主题 | P1 | 1 | Not Started | 主题切换测试 | Tailwind/shadcn 主题令牌 | 与浅色共主题体系 |
| 26 | 浅色模式 | 浅色主题 | P1 | 1 | Not Started | 主题切换测试 | 主题令牌 | 默认浅色 |
| 27 | 快捷键 | 全局键盘快捷键（导航/分析/翻转等） | P2 | 5 | Not Started | 快捷键映射单测 | UI 状态、命令层 | 提供可配置清单 |
| 28 | Windows Installer | NSIS/MSI 安装版 | P0 | 6 | Not Started | 安装包冒烟测试 | 打包、许可决策 | 含 WebView2 引导 |
| 29 | Portable | 便携版 | P1 | 6 | Not Started | 解压即用冒烟 | 打包方案（NEEDS_VERIFICATION） | 便携方案待定 |
| 30 | GitHub Actions | CI 与 Release 自动化 | P1 | 0 | In Progress | CI 全绿 + Release 触发 | 工程骨架 | Phase 0 CI 骨架已完成；Release 流水线待 Phase 6 |
| 31 | GitHub Release | 发布产物到 GitHub Release | P1 | 6 | Not Started | Release 产物校验 | CI/CD、打包 | 依赖许可决策 |

## 说明

- `Phase 0` 的 #30（GitHub Actions）在后续 Phase 中持续演进，最终于 Phase 6 交付完整 Release 流水线。
- #9/#28/#29/#31 与许可决策耦合：开发可先行，**公开分发在许可确认前标记为 Blocked**（见 `licensing.md`）。
- 每项 `Test` 对应 `testing.md` 中的可执行测试；功能「Done」= 该 Test 通过 + `Status` 更新为 `Done`。