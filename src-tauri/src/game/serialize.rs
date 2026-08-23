//! 棋谱文档序列化（tree_json，版本 1）。
//!
//! **只序列化 Document State**（startpos / root / nodes / headers），
//! 明确排除 Session State（`current` / `redo_stack`）——它们是会话/导航状态，
//! 不属于棋谱持久化数据（见 `game::tree` 模块注释）。
//!
//! Schema（与 docs/game-model.md §7.4 一致）：
//! ```json
//! {
//!   "version": 1,
//!   "startpos": "rnbakabnr/...",
//!   "headers": { "title": "", "red": "", "black": "", "event": "", "date": "", "result": "*" },
//!   "root": 0,
//!   "nodes": {
//!     "0": { "mv": null, "comment": "", "nags": [], "children": [1] },
//!     "1": { "mv": "h2e2", "comment": "", "nags": [], "children": [] }
//!   }
//! }
//! ```
//! 节点上的 `fen`、`parent`、`side_to_move`、`fullmove_number` 均为派生数据，不持久化；
//! 导入时通过从起始局面回放着法重新推导并校验。

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::board::fen::{parse_fen, to_fen};
use crate::board::rules::apply_move;
use crate::board::types::{Move, Position};

use super::nag::Nag;
use super::tree::{GameHeaders, GameNode, GameTree, NodeId};

pub const TREE_JSON_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeDocument {
    pub version: u32,
    pub startpos: String,
    pub headers: GameHeaders,
    pub root: NodeId,
    /// id → 节点数据（mv/comment/nags/children）。
    pub nodes: BTreeMap<NodeId, SerializedNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedNode {
    pub mv: Option<String>,
    pub comment: String,
    pub nags: Vec<String>,
    pub children: Vec<NodeId>,
}

/// 构建棋谱文档（不含 current / redo_stack 等会话状态）。
fn build_document(tree: &GameTree) -> TreeDocument {
    TreeDocument {
        version: TREE_JSON_VERSION,
        startpos: tree.startpos.clone(),
        headers: tree.headers.clone(),
        root: tree.root,
        nodes: tree
            .nodes
            .iter()
            .map(|(id, n)| {
                (
                    *id,
                    SerializedNode {
                        mv: n.mv.map(|m| m.uci()),
                        comment: n.comment.clone(),
                        nags: n.nags.iter().map(|x| x.symbol().to_string()).collect(),
                        children: n.children.clone(),
                    },
                )
            })
            .collect(),
    }
}

/// 序列化棋谱文档（不含 current / redo_stack 等会话状态）。
pub fn to_tree_json(tree: &GameTree) -> Result<String, String> {
    let doc = build_document(tree);
    serde_json::to_string_pretty(&doc).map_err(|e| format!("序列化棋谱失败：{e}"))
}

/// 「当前棋局」持久化文件格式（B3 最小保存/恢复）：文档 + 当前节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedGame {
    version: u32,
    current: NodeId,
    document: TreeDocument,
}

pub const SAVED_GAME_VERSION: u32 = 1;

/// 保存当前棋局（文档 + 当前节点；重做栈不持久化）。
pub fn save_game(tree: &GameTree) -> Result<String, String> {
    let saved = SavedGame {
        version: SAVED_GAME_VERSION,
        current: tree.current,
        document: build_document(tree),
    };
    serde_json::to_string_pretty(&saved).map_err(|e| format!("序列化当前棋局失败：{e}"))
}

/// 恢复已保存的当前棋局（校验结构、着法合法性，并恢复当前节点）。
pub fn load_game(s: &str) -> Result<GameTree, String> {
    let saved: SavedGame = serde_json::from_str(s).map_err(|e| format!("解析棋局存档失败：{e}"))?;
    if saved.version != SAVED_GAME_VERSION {
        return Err(format!("不支持的棋局存档版本：{}", saved.version));
    }
    let doc_json =
        serde_json::to_string(&saved.document).map_err(|e| format!("解析棋局文档失败：{e}"))?;
    let mut tree = from_tree_json(&doc_json)?;
    tree.set_current(saved.current)
        .map_err(|e| format!("恢复当前节点 {} 失败：{e}", saved.current))?;
    Ok(tree)
}

/// 反序列化棋谱文档为新的棋谱树。
///
/// - 保留文档中的节点 id；重新推导 `fen` / `parent` / `side_to_move` / `fullmove_number`；
/// - 校验树结构完整（根存在、被引用的子节点存在、无环、无孤儿节点）；
/// - 会话状态重置：`current = root`、`redo_stack` 为空。
pub fn from_tree_json(s: &str) -> Result<GameTree, String> {
    let doc: TreeDocument =
        serde_json::from_str(s).map_err(|e| format!("解析棋谱 JSON 失败：{e}"))?;
    if doc.version != TREE_JSON_VERSION {
        return Err(format!("不支持的棋谱版本：{}", doc.version));
    }
    let root_pos = parse_fen(&doc.startpos).map_err(|e| format!("起始局面 FEN 无效：{e}"))?;

    if !doc.nodes.contains_key(&doc.root) {
        return Err(format!("根节点 {} 不存在", doc.root));
    }
    if doc.nodes[&doc.root].mv.is_some() {
        return Err("根节点不应有着法".to_string());
    }

    // 解析节点中间表示
    let mut moves: HashMap<NodeId, Option<Move>> = HashMap::new();
    let mut comments: HashMap<NodeId, String> = HashMap::new();
    let mut nags: HashMap<NodeId, Vec<Nag>> = HashMap::new();
    let mut children_map: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for (id, node) in &doc.nodes {
        let mv = match &node.mv {
            None => None,
            Some(u) => Some(Move::parse_uci(u).ok_or_else(|| format!("节点 {id} 着法非法：{u}"))?),
        };
        let mut nags_out = Vec::new();
        for s in &node.nags {
            nags_out
                .push(Nag::from_symbol(s).ok_or_else(|| format!("节点 {id} 注释符号非法：{s}"))?);
        }
        moves.insert(*id, mv);
        comments.insert(*id, node.comment.clone());
        nags.insert(*id, nags_out);
        children_map.insert(*id, node.children.clone());
    }

    // 从根 BFS：推导 parent / fen / 缓存元数据，并检查环与孤儿
    let mut nodes: HashMap<NodeId, GameNode> = HashMap::new();
    nodes.insert(
        doc.root,
        GameNode {
            id: doc.root,
            mv: None,
            parent: None,
            fen: doc.startpos.clone(),
            side_to_move: root_pos.side_to_move,
            fullmove_number: root_pos.fullmove_number,
            comment: comments[&doc.root].clone(),
            nags: nags[&doc.root].clone(),
            children: Vec::new(),
        },
    );

    let mut positions: HashMap<NodeId, Position> = HashMap::new();
    positions.insert(doc.root, root_pos);
    let mut visited: HashSet<NodeId> = HashSet::new();
    visited.insert(doc.root);
    let mut queue: VecDeque<NodeId> = VecDeque::new();
    queue.push_back(doc.root);

    while let Some(old_parent) = queue.pop_front() {
        let parent_pos = positions[&old_parent].clone();
        for old_child in &children_map[&old_parent] {
            if !doc.nodes.contains_key(old_child) {
                return Err(format!("节点 {old_child} 不存在但被引用"));
            }
            if !visited.insert(*old_child) {
                return Err(format!("棋谱树存在环或重复引用：{old_child}"));
            }
            let mv = moves[old_child].ok_or_else(|| format!("非根节点 {old_child} 缺少着法"))?;
            let pos = apply_move(&parent_pos, mv)
                .ok_or_else(|| format!("节点 {old_child} 着法非法：{}", mv.uci()))?;
            let fen = to_fen(&pos);
            let node = GameNode {
                id: *old_child,
                mv: Some(mv),
                parent: Some(old_parent),
                fen,
                side_to_move: pos.side_to_move,
                fullmove_number: pos.fullmove_number,
                comment: comments[old_child].clone(),
                nags: nags[old_child].clone(),
                children: children_map[old_child].clone(),
            };
            nodes.insert(*old_child, node);
            positions.insert(*old_child, pos);
            queue.push_back(*old_child);
        }
    }
    if visited.len() != doc.nodes.len() {
        return Err("棋谱树存在无法从根到达的节点（孤儿）".to_string());
    }
    // 根节点的 children 在 BFS 中未被写入，单独补齐
    if let Some(root_node) = nodes.get_mut(&doc.root) {
        root_node.children = children_map[&doc.root].clone();
    }

    Ok(GameTree::from_document(
        doc.startpos.clone(),
        doc.root,
        nodes,
        doc.headers.clone(),
    ))
}
