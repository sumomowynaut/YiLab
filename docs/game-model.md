# 局面与棋谱树数据模型（Game Model）

本文档定义两个核心模型：**Position**（局面）与 **Game Tree**（棋谱树），以及坐标/走法/FEN/规则约定。

## 1. 坐标系

- 棋盘 9 列（files）× 10 行（ranks）。
- 列用字母 `a`–`i`（a 为红方最左侧，即黑方视角最右侧）；行用数字，**从黑方底线到红方底线**方向为 `9`→`0`（与 UCI-Cyclone 一致：数字从 0 开始，`0` 为红方底线）。
- 走法用「起点格 + 终点格」四字符表示，如 `h2e2`。

```
黑方
  9  · · · · · · · · ·
  8  · · · · · · · · ·
  ...
  0  · · · · · · · · ·
红方（side to move = w 时，红方底线为 rank 0）
  files: a b c d e f g h i
```

> 注：皮卡鱼采用 UCI-Cyclone 坐标约定（行从 0 开始，见皮卡鱼 Wiki「UCI 协议」）。本项目与之一致，避免与引擎之间做坐标换算。

## 2. 棋子编码

红方棋子为大写、黑方为小写（与 FEN 一致）：

| 棋子 | 红 | 黑 |
|------|----|----|
| 帅/将 King | K | k |
| 仕/士 Advisor | A | a |
| 相/象 Elephant | B | b |
| 马 Horse | N | n |
| 车 Rook | R | r |
| 炮 Cannon | C | c |
| 兵/卒 Pawn | P | p |

## 3. Position 数据模型

### 3.1 Rust 权威表示

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color { Red, Black }

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind { King, Advisor, Elephant, Horse, Rook, Cannon, Pawn }

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece { pub color: Color, pub kind: PieceKind }

/// 10 ranks × 9 files。用 (rank, file) 索引，rank 0 为红方底线。
pub struct Position {
    pub board: [[Option<Piece>; 9]; 10],
    pub side_to_move: Color,
    pub halfmove_clock: u32,   // 自上次吃子以来的半步数（60 回合自然限招用）
    pub fullmove_number: u32,  // 回合数（FEN 第 6 字段）
    pub zobrist: u64,          // Zobrist 哈希（重复局面/开局库检索用）
}
```

### 3.2 不变式（Invariants）

- 红方有且仅有 1 个 `K`，黑方有且仅有 1 个 `k`。
- 同色至多 2 仕/2 相；兵/卒同色至多 5 个。
- `side_to_move` 一方当前不能正被将军（非法局面不会被构造；解析 FEN 时校验）。
- 两将不可「照面」（将帅不能在同列无阻挡，等价于被将军）。

### 3.3 前端镜像类型（TypeScript，仅用于渲染）

```ts
type Color = 'w' | 'b';                 // 与 FEN 一致
type Piece = { color: Color; kind: 'k'|'a'|'b'|'n'|'r'|'c'|'p' };
type Board = (Piece | null)[][];        // [rank][file]
type PositionSnapshot = {
  board: Board;
  sideToMove: Color;
  halfmoveClock: number;
  fullmoveNumber: number;
};
```

## 4. Move 数据模型

```rust
pub struct Move {
    pub from: (u8, u8),   // (rank, file)
    pub to:   (u8, u8),
}
```

- 中国象棋无升变，走法仅需 `from`/`to`。
- 文本表示采用 UCI-Cyclone 四字符：`{file}{rank}{file}{rank}`，如 `g0f0`。
- 与皮卡鱼 `position ... moves g0f0 d8d9` 格式直接互通。

## 5. 走法生成与合法性

在 `board/` 用 Rust 实现（不依赖引擎），供 UI 与引擎两端共用：

1. **伪合法生成**：按棋子规则生成候选（马蹩腿、相塞眼、炮需炮架、兵过河后平移等）。
2. **合法性过滤**：做走棋，若己方被将军则剪掉；再将帅照面视作被将军处理。
3. **将军/绝杀/困毙判定**：
   - 将军（check）：任一方着法能吃掉对方将/帅。
   - 绝杀（checkmate）：被将军且无任何合法应着。
   - 困毙（stalemate，象棋中判负）：未被将军但无合法着法 → 判负（象棋规则与国象不同）。
4. **重复局面**：用 Zobrist 哈希 + 历史局面计数判定长将/长捉/循环（规则细节见 `engine.md` 的 Repetition Rule 选项）。
5. **自然限招**：`halfmove_clock` 达阈值（默认 120 半步 = 60 回合）判和。

> 走法生成器必须能通过 **perft** 对拍（见 `testing.md`）。

## 6. FEN

### 6.1 格式

中国象棋 FEN 6 字段，与皮卡鱼一致：

```
<局面> <走子方> <未使用> <未使用> <半步钟> <回合数>
示例：3k1a3/4a4/5n3/9/9/9/9/9/9/4KR3 w - - 0 1
```

- 局面：10 段，每段一个 rank（从 rank 9 到 rank 0），`9` 表示空行；字母同 §2。
- 走子方：`w` 红 / `b` 黑。
- 第 3、4 字段在象棋中恒为 `-`（国象的易位/过路兵不适用），解析时兼容忽略。
- 起始局面：`rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1`。

### 6.2 能力要求

- 解析：严格校验 + 明确的错误信息（行数、棋子数、非法字符、将帅数量）。
- 序列化：从 `Position` 输出规范 FEN。
- 往返一致：`parse(serialize(p)) == p`（作为测试与导入导出基础）。
- FEN 是 Import/Export、局面链接、开局库检索、引擎 `position fen` 的统一交换格式。

## 7. Game Tree 数据模型

复盘的核心是「棋谱树」而非线性棋谱。

### 7.1 节点

```rust
pub struct GameNode {
    pub id: NodeId,              // 稳定唯一（UUID/自增，不随重排变化）
    pub mv: Option<Move>,        // 从父节点走到本节点的着法；根节点为 None
    pub fen: String,             // 本节点局面（缓存，便于展示与检索）
    pub comment: String,         // 本节点注释（可空）
    pub nags: Vec<Nag>,          // 注释符号（?! 等，可空）
    pub children: Vec<NodeId>,   // 有序子节点（第一项为主线续着）
}

pub struct GameTree {
    pub root: NodeId,
    pub nodes: HashMap<NodeId, GameNode>,
    pub startpos: String,        // 根局面 FEN（默认 startpos）
    pub headers: GameHeaders,    // 对局元数据（红/黑、事件、日期、结果）
}
```

### 7.2 主线与变例

- **主线（main line）**：从根节点开始，始终沿每个节点的 `children[0]` 走到叶子。
- **变例（variation）**：某节点的 `children[1..]` 分支。变例内同样遵循「第一子为主线」。
- 节点上的 `children` 顺序即变例顺序；「提为变例/降为变例/交换变例顺序」就是对该数组的 `promote/remove/swap` 操作。

### 7.3 操作（全部可单测）

| 操作 | 语义 |
|------|------|
| `make_move(node, mv)` | 若 `mv` 已是子节点则导航；否则校验合法后新建子节点 |
| `add_variation(node, mv)` | 在 `node` 下追加一条新变例（合法校验） |
| `delete_variation(node, child)` | 删除整棵子树 |
| `promote_variation(node, child)` | 将 `child` 移到 `children[0]`（变主线） |
| `reorder_variation(node, from, to)` | 交换变例顺序 |
| `set_comment(node, text)` | 编辑注释 |
| `truncate(node)` | 截断该节点之后的全部着法 |

### 7.4 序列化（tree_json）

采用稳定的 JSON schema（含版本号），用于 SQLite `games.tree_json` 与未来剪贴板复制棋谱：

```json
{
  "version": 1,
  "startpos": "rnbakabnr/...",
  "headers": { "red": "...", "black": "...", "result": "*" },
  "root": "n0",
  "nodes": {
    "n0": { "mv": null, "fen": "...", "comment": "", "nags": [], "children": ["n1"] },
    "n1": { "mv": "h2e2", "fen": "...", "comment": "", "nags": [], "children": [] }
  }
}
```

> 决策：棋谱树是「文档型」数据，用单 JSON 字段持久化，避免为变例/注释拆多张关系表（见 `architecture.md` §5）。

## 8. 规则边界与待确认

- 象棋的「长将/长捉/循环」裁决规则不统一（亚规/中规/各平台差异）。本项目**不自行实现完整棋规裁决**，只做基础的重复局面检测；复杂棋例裁决交给引擎（Pikafish 的 `Repetition Rule` 选项，见 `engine.md`）或仅提示用户。
- 首版 `Position` 不含「是否将军中」缓存；是否将军按需计算（避免过度优化）。