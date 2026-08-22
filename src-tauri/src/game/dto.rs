//! 棋谱树快照 DTO（发送给 React 前端）。

use serde::Serialize;

use crate::board::dto::PositionDto;
use crate::board::fen::parse_fen;
use crate::board::types::Color;

use super::tree::{GameTree, NodeId};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNodeDto {
    pub id: u64,
    /// 本节点着法（UCI），根节点为 None。
    pub mv: Option<String>,
    /// 显示回合数（红方着法 N.，黑方着法 N…）。
    pub move_number: u32,
    pub is_red: bool,
    pub comment: String,
    pub nags: Vec<String>,
    pub children: Vec<TreeNodeDto>,
    pub is_variation: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSnapshot {
    pub tree: TreeNodeDto,
    pub current_id: u64,
    pub current_fen: String,
    pub position: PositionDto,
    pub comment: String,
    pub nags: Vec<String>,
    pub has_parent: bool,
    pub previous_id: Option<u64>,
    pub next_main_id: Option<u64>,
    pub undo_available: bool,
    pub redo_available: bool,
}

/// 生成当前棋谱树快照。
pub fn snapshot(tree: &GameTree) -> Result<GameSnapshot, String> {
    let current = tree.current_node();
    let position = PositionDto::from_position(&parse_fen(&current.fen)?);
    let previous_id = current.parent;
    let next_main_id = current.children.first().copied();
    Ok(GameSnapshot {
        tree: build_node(tree, tree.root)?,
        current_id: current.id,
        current_fen: current.fen.clone(),
        position,
        comment: current.comment.clone(),
        nags: current
            .nags
            .iter()
            .map(|n| n.symbol().to_string())
            .collect(),
        has_parent: current.parent.is_some(),
        previous_id,
        next_main_id,
        undo_available: tree.undo_available(),
        redo_available: tree.redo_available(),
    })
}

fn build_node(tree: &GameTree, id: NodeId) -> Result<TreeNodeDto, String> {
    let n = tree.node(id).map_err(|e| format!("{e}"))?;
    // 使用插入时缓存的元数据，避免快照时逐节点 parse_fen（H3）。
    // 红方着法后轮到黑方（side 'b'）；黑方着法后 fullmove 已 +1。
    let (move_number, is_red) = match n.mv {
        Some(_) => {
            let red = n.side_to_move == Color::Black;
            let num = if red {
                n.fullmove_number
            } else {
                n.fullmove_number.saturating_sub(1).max(1)
            };
            (num, red)
        }
        None => (0, true),
    };
    let children = n
        .children
        .iter()
        .map(|c| build_node(tree, *c))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TreeNodeDto {
        id,
        mv: n.mv.map(|m| m.uci()),
        move_number,
        is_red,
        comment: n.comment.clone(),
        nags: n.nags.iter().map(|x| x.symbol().to_string()).collect(),
        children,
        is_variation: tree.is_variation(id),
    })
}
