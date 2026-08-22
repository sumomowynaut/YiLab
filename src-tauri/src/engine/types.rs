//! 引擎层公共类型（UCI 相关）。

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// 分值：`cp` 厘兵（红方视角）或 `mate`（多少步内杀）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Score {
    Cp(i32),
    Mate(i32),
}

/// 一行 `info ...` 的解析结果。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoLine {
    pub depth: Option<u32>,
    pub seldepth: Option<u32>,
    pub multipv: u32,
    pub score: Option<Score>,
    pub nodes: Option<u64>,
    pub nps: Option<u64>,
    pub time_ms: Option<u64>,
    pub pv: Vec<String>,
    pub lowerbound: bool,
    pub upperbound: bool,
}

/// `bestmove` 行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BestMove {
    pub mv: String,
    pub ponder: Option<String>,
}

/// 引擎主动推送的事件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineEvent {
    Info(InfoLine),
    InfoString(String),
    BestMove(BestMove),
    Ready,
    Started,
    Stopped,
    OptionSet { name: String, value: Option<String> },
    Error(String),
    Crashed { code: Option<i32> },
}

/// 引擎生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineStatus {
    Stopped,
    Ready,
    Searching,
    Crashed,
}

/// `go` 参数。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoParams {
    pub infinite: bool,
    pub depth: Option<u32>,
    pub movetime_ms: Option<u64>,
    pub nodes: Option<u64>,
    pub searchmoves: Vec<String>,
    pub wtime_ms: Option<u64>,
    pub btime_ms: Option<u64>,
}

/// `uci` 握手返回的引擎选项定义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UciOption {
    pub name: String,
    pub kind: String,
    pub default: Option<String>,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub var: Vec<String>,
}

/// 启动配置。
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    /// 引擎工作目录（用于让引擎默认读到同目录下的 pikafish.nnue）。
    pub cwd: Option<PathBuf>,
    pub handshake_timeout: Duration,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            program: PathBuf::new(),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
            handshake_timeout: Duration::from_secs(10),
        }
    }
}
