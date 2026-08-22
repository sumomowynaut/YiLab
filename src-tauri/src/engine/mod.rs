//! Pikafish 引擎层（UCI）。
//!
//! - `types`：公共类型（Info/BestMove/Event/Status/GoParams/UciOption）。
//! - `uci`：UCI 命令构建与 stdout 解析（纯函数）。
//! - `manager`：Engine Manager（进程生命周期、异步 IO、串行化、崩溃/重启）。
//!
//! 约束：React 不直接管理 Pikafish；所有引擎交互都经 Rust Engine Manager。

pub mod manager;
pub mod types;
pub mod uci;

pub use manager::EngineManager;
pub use types::{
    BestMove, EngineConfig, EngineEvent, EngineStatus, GoParams, InfoLine, Score, UciOption,
};
