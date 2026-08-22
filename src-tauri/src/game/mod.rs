//! 棋谱树（Game Tree）。
//!
//! 结构：`tree`（树模型与操作）、`nag`（注释符号）、`dto`（前端快照）。

pub mod dto;
pub mod nag;
pub mod tree;

pub use nag::Nag;
pub use tree::{GameError, GameHeaders, GameNode, GameTree, NodeId};
