//! 棋谱树（Game Tree）：真正的树结构，而非着法数组。
//!
//! - Root：根节点（无着法，代表起始局面）。
//! - MoveNode：每个着法一个节点，含父指针与有序子节点。
//! - MainLine：始终沿 children[0] 行进。
//! - Variation / Nested Variation：children[1..] 为变例，变例内同样可再分支。
//! - 从任意节点均可通过 `restore_position` 回放父链恢复完整局面。
//!
//! # Document State 与 Session State
//!
//! - **Document State（棋谱文档数据）**：`startpos`、`root`、`nodes`（含着法/注释/NAG/子节点）、`headers`。
//!   这些是棋谱的持久化内容，序列化时保留。
//! - **Session State（会话/导航状态）**：`current`（当前浏览节点）、`redo_stack`（重做栈）。
//!   这些只属于当前编辑会话，**绝不进入棋谱持久化数据**（见 `game::serialize`）。
//!
//! 注意：`current`/`redo_stack` 与文档状态同存于本结构，是为了在当前阶段保持改动最小；
//! 未来若引入多文档/持久化，应拆分为 `GameSession { tree, current, redo_stack }`。

use std::collections::HashMap;
use std::fmt;

use crate::board::fen::{parse_fen, to_fen};
use crate::board::rules::{apply_move, make_unchecked};
use crate::board::types::{Color, Move, Position};

use super::nag::Nag;

pub type NodeId = u64;

impl GameNode {
    /// 本节点着法是否红方所走（红方着法后轮到黑方）。
    pub fn is_red(&self) -> bool {
        self.side_to_move == Color::Black
    }

    /// 显示回合数（红方着法 N.，黑方着法 N…）。
    pub fn move_number(&self) -> u32 {
        if self.is_red() {
            self.fullmove_number
        } else {
            self.fullmove_number.saturating_sub(1).max(1)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameNode {
    pub id: NodeId,
    /// 从父节点走到本节点的着法；根节点为 None。
    pub mv: Option<Move>,
    pub parent: Option<NodeId>,
    /// 本节点局面 FEN（插入时计算并缓存）。
    pub fen: String,
    /// 本节点局面下「轮到谁走」（缓存，避免快照时逐节点 parse_fen）。
    pub side_to_move: Color,
    /// 本节点局面下的回合数（缓存，避免快照时逐节点 parse_fen）。
    pub fullmove_number: u32,
    pub comment: String,
    pub nags: Vec<Nag>,
    /// 有序子节点；children[0] 为主线续着。
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameHeaders {
    pub title: String,
    pub red: String,
    pub black: String,
    pub event: String,
    pub date: String,
    pub result: String,
}

#[derive(Debug, Clone)]
pub struct GameTree {
    /// 【文档】起始局面 FEN。
    pub startpos: String,
    /// 【文档】根节点 id。
    pub root: NodeId,
    /// 【文档】节点表。
    pub nodes: HashMap<NodeId, GameNode>,
    /// 【会话】当前浏览节点（CurrentNode）。
    pub current: NodeId,
    /// 【文档】对局元数据。
    pub headers: GameHeaders,
    next_id: NodeId,
    /// 【会话】连续「悔棋」时记录的可重做节点栈。
    redo_stack: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NodeNotFound(NodeId),
    IllegalMove(String),
    CannotDeleteRoot,
    NotAVariation(NodeId),
    InvalidStartFen(String),
    NoParent,
    NoNext,
    NothingToRedo,
    InvalidIndex {
        parent: NodeId,
        index: usize,
        len: usize,
    },
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::NodeNotFound(id) => write!(f, "棋谱节点不存在：{id}"),
            GameError::IllegalMove(m) => write!(f, "非法着法：{m}"),
            GameError::CannotDeleteRoot => write!(f, "不能删除根节点"),
            GameError::NotAVariation(id) => write!(f, "节点不是变例起点：{id}"),
            GameError::InvalidStartFen(e) => write!(f, "起始局面 FEN 无效：{e}"),
            GameError::NoParent => write!(f, "已到棋谱起点"),
            GameError::NoNext => write!(f, "已到棋谱终点"),
            GameError::NothingToRedo => write!(f, "无可重做"),
            GameError::InvalidIndex { parent, index, len } => {
                write!(f, "节点 {parent} 的子节点索引 {index} 越界（共 {len}）")
            }
        }
    }
}

impl GameTree {
    /// 从已构建的文档部件重建棋谱树（用于反序列化）：会话状态重置为根。
    pub(crate) fn from_document(
        startpos: String,
        root: NodeId,
        nodes: HashMap<NodeId, GameNode>,
        headers: GameHeaders,
    ) -> GameTree {
        let max_id = nodes.keys().copied().max().unwrap_or(root);
        GameTree {
            startpos,
            root,
            nodes,
            current: root,
            headers,
            next_id: max_id + 1,
            redo_stack: Vec::new(),
        }
    }

    /// 以指定起始局面创建新棋谱树（校验 FEN）。
    pub fn new(startpos: &str) -> Result<GameTree, GameError> {
        let start = parse_fen(startpos).map_err(GameError::InvalidStartFen)?;
        let root = 0;
        let mut nodes = HashMap::new();
        nodes.insert(
            root,
            GameNode {
                id: root,
                mv: None,
                parent: None,
                fen: startpos.to_string(),
                side_to_move: start.side_to_move,
                fullmove_number: start.fullmove_number,
                comment: String::new(),
                nags: Vec::new(),
                children: Vec::new(),
            },
        );
        Ok(GameTree {
            startpos: startpos.to_string(),
            root,
            nodes,
            current: root,
            headers: GameHeaders::default(),
            next_id: 1,
            redo_stack: Vec::new(),
        })
    }

    pub fn node(&self, id: NodeId) -> Result<&GameNode, GameError> {
        self.nodes.get(&id).ok_or(GameError::NodeNotFound(id))
    }

    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut GameNode, GameError> {
        self.nodes.get_mut(&id).ok_or(GameError::NodeNotFound(id))
    }

    pub fn current_node(&self) -> &GameNode {
        &self.nodes[&self.current]
    }

    pub fn current_id(&self) -> NodeId {
        self.current
    }

    /// 当前节点相对根节点的着法数（半回合数，供「脱库步数」等使用）。
    pub fn current_plies(&self) -> u32 {
        let mut plies = 0;
        let mut cur = Some(self.current);
        while let Some(id) = cur {
            if let Some(n) = self.nodes.get(&id) {
                if n.mv.is_some() {
                    plies += 1;
                }
                cur = n.parent;
            } else {
                break;
            }
        }
        plies
    }

    pub fn redo_available(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_available(&self) -> bool {
        self.current_node().parent.is_some()
    }

    /// 从任意节点回放父链恢复完整局面（不依赖节点缓存的 FEN，验证树的一致性）。
    pub fn restore_position(&self, node: NodeId) -> Result<Position, GameError> {
        let mut moves = Vec::new();
        let mut cur = Some(node);
        while let Some(id) = cur {
            let n = self.node(id)?;
            if let Some(mv) = n.mv {
                moves.push(mv);
            }
            cur = n.parent;
        }
        let mut pos = parse_fen(&self.startpos).map_err(GameError::InvalidStartFen)?;
        for mv in moves.iter().rev() {
            pos = make_unchecked(&pos, *mv);
        }
        Ok(pos)
    }

    /// 在当前节点插入着法；若该着法已存在则复用子节点。
    pub fn insert_move(&mut self, mv: Move) -> Result<NodeId, GameError> {
        self.insert_move_at(self.current, mv)
    }

    /// 在父节点插入着法（追加为子节点末尾）；若该着法已存在则复用子节点。
    pub fn insert_move_at(&mut self, parent: NodeId, mv: Move) -> Result<NodeId, GameError> {
        self.insert_child(parent, mv, false)
    }

    /// 在父节点插入着法作为主线续着（children[0]，已有子节点顺移）；同着法已存在则复用。
    pub fn insert_main_at(&mut self, parent: NodeId, mv: Move) -> Result<NodeId, GameError> {
        self.insert_child(parent, mv, true)
    }

    /// 共享插入逻辑：`main` 为 true 时插入到 children[0]，否则追加到末尾。
    fn insert_child(&mut self, parent: NodeId, mv: Move, main: bool) -> Result<NodeId, GameError> {
        let parent_pos = self.restore_position(parent)?;
        let next = apply_move(&parent_pos, mv).ok_or(GameError::IllegalMove(mv.uci()))?;
        // 复用相同着法的既有子节点（避免重复分支）
        let existing: Vec<NodeId> = self.node(parent)?.children.clone();
        for child in existing {
            if self.node(child)?.mv == Some(mv) {
                self.current = child;
                self.redo_stack.clear();
                return Ok(child);
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(
            id,
            GameNode {
                id,
                mv: Some(mv),
                parent: Some(parent),
                fen: to_fen(&next),
                side_to_move: next.side_to_move,
                fullmove_number: next.fullmove_number,
                comment: String::new(),
                nags: Vec::new(),
                children: Vec::new(),
            },
        );
        let children = &mut self.node_mut(parent)?.children;
        if main {
            children.insert(0, id);
        } else {
            children.push(id);
        }
        self.current = id;
        self.redo_stack.clear();
        Ok(id)
    }

    /// 删除整条变例（节点必须是其父节点的非首个子节点）。
    pub fn delete_variation(&mut self, node: NodeId) -> Result<(), GameError> {
        if node == self.root {
            return Err(GameError::CannotDeleteRoot);
        }
        let parent = self.node(node)?.parent.ok_or(GameError::CannotDeleteRoot)?;
        let is_first = self.node(parent)?.children.first() == Some(&node);
        if is_first {
            return Err(GameError::NotAVariation(node));
        }
        self.node_mut(parent)?.children.retain(|c| *c != node);
        let mut subtree = Vec::new();
        self.collect_subtree(node, &mut subtree);
        let current_inside = subtree.contains(&self.current);
        for id in subtree {
            self.nodes.remove(&id);
        }
        if current_inside {
            self.current = parent;
        }
        self.redo_stack.clear();
        Ok(())
    }

    /// 把一支变例提升为主线（移动到其父节点 children[0]）。
    pub fn promote_variation(&mut self, node: NodeId) -> Result<(), GameError> {
        if node == self.root {
            return Err(GameError::CannotDeleteRoot);
        }
        let parent = self.node(node)?.parent.ok_or(GameError::CannotDeleteRoot)?;
        let is_first = self.node(parent)?.children.first() == Some(&node);
        if is_first {
            return Err(GameError::NotAVariation(node));
        }
        self.node_mut(parent)?.children.retain(|c| *c != node);
        self.node_mut(parent)?.children.insert(0, node);
        self.redo_stack.clear();
        Ok(())
    }

    /// 调整变例顺序：把 `parent` 的 children[from] 移动到 children[to]。
    /// `from`/`to` 必须都位于变例区（index >= 1），避免把主线移出首位。
    pub fn reorder_variation(
        &mut self,
        parent: NodeId,
        from: usize,
        to: usize,
    ) -> Result<(), GameError> {
        let len = self.node(parent)?.children.len();
        if from >= len || to >= len {
            return Err(GameError::InvalidIndex {
                parent,
                index: if from >= len { from } else { to },
                len,
            });
        }
        if from == 0 || to == 0 {
            return Err(GameError::InvalidIndex {
                parent,
                index: from.min(to),
                len,
            });
        }
        if from == to {
            return Ok(());
        }
        let children = &mut self.node_mut(parent)?.children;
        let node = children.remove(from);
        children.insert(to, node);
        self.redo_stack.clear();
        Ok(())
    }

    fn collect_subtree(&self, id: NodeId, out: &mut Vec<NodeId>) {
        out.push(id);
        if let Ok(n) = self.node(id) {
            for child in &n.children {
                self.collect_subtree(*child, out);
            }
        }
    }

    /// 显式跳转到任意节点（清除重做栈）。
    pub fn set_current(&mut self, node: NodeId) -> Result<(), GameError> {
        self.node(node)?;
        self.current = node;
        self.redo_stack.clear();
        Ok(())
    }

    /// 悔棋：回到父节点，记录可重做节点。
    pub fn undo(&mut self) -> Result<NodeId, GameError> {
        let parent = self.current_node().parent.ok_or(GameError::NoParent)?;
        let prev = self.current;
        self.current = parent;
        self.redo_stack.push(prev);
        Ok(self.current)
    }

    /// 重做：回到最近一次悔棋前的节点。
    pub fn redo(&mut self) -> Result<NodeId, GameError> {
        let node = self.redo_stack.pop().ok_or(GameError::NothingToRedo)?;
        self.current = node;
        Ok(self.current)
    }

    /// 上一步（沿主线父节点，纯导航）。
    pub fn previous(&mut self) -> Result<NodeId, GameError> {
        let parent = self.current_node().parent.ok_or(GameError::NoParent)?;
        self.current = parent;
        self.redo_stack.clear();
        Ok(self.current)
    }

    /// 下一步（沿主线首子，纯导航）。
    pub fn next_move(&mut self) -> Result<NodeId, GameError> {
        let first = self
            .current_node()
            .children
            .first()
            .copied()
            .ok_or(GameError::NoNext)?;
        self.current = first;
        self.redo_stack.clear();
        Ok(self.current)
    }

    pub fn go_to_start(&mut self) -> Result<(), GameError> {
        self.current = self.root;
        self.redo_stack.clear();
        Ok(())
    }

    pub fn go_to_end(&mut self) -> Result<(), GameError> {
        while let Some(first) = self.current_node().children.first().copied() {
            self.current = first;
        }
        self.redo_stack.clear();
        Ok(())
    }

    /// 按节点 id 设置注释（H1：修改棋谱数据的操作按显式节点定位，不依赖 current）。
    pub fn set_comment_at(&mut self, node: NodeId, comment: String) -> Result<(), GameError> {
        self.node_mut(node)?.comment = comment;
        Ok(())
    }

    /// 按节点 id 添加/移除 NAG（H1：不依赖 current）。
    pub fn set_nag_at(&mut self, node: NodeId, nag: Nag, add: bool) -> Result<(), GameError> {
        let n = self.node_mut(node)?;
        if add {
            if !n.nags.contains(&nag) {
                n.nags.push(nag);
            }
        } else {
            n.nags.retain(|x| *x != nag);
        }
        Ok(())
    }

    /// 主线：从根沿 children[0] 到叶子。
    pub fn main_line(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = self.root;
        loop {
            out.push(cur);
            let Some(first) = self.nodes[&cur].children.first().copied() else {
                break;
            };
            cur = first;
        }
        out
    }

    /// 节点是否为一支变例的起点（非首个子节点）。
    pub fn is_variation(&self, node: NodeId) -> bool {
        if node == self.root {
            return false;
        }
        let Ok(n) = self.node(node) else {
            return false;
        };
        let Some(parent) = n.parent else {
            return false;
        };
        self.node(parent)
            .map(|p| p.children.first() != Some(&node))
            .unwrap_or(false)
    }
}
