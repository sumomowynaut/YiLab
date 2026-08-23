//! 中国象棋棋盘核心（Board Core）。
//!
//! 结构：`types`（类型）、`rules`（规则）、`fen`（FEN）、`validate`（校验）、
//! `transform`（视图变换）、`dto`（前端 DTO）。

pub mod chinese;
pub mod dto;
pub mod fen;
pub mod rules;
pub mod transform;
pub mod types;
pub mod validate;
pub mod zobrist;

#[cfg(test)]
mod tests;
