//! Tauri IPC 命令层：棋盘核心（board）与棋谱树（game）的薄封装。

use crate::board::{
    dto::PositionDto,
    fen::{parse_fen, to_fen},
    rules::{apply_move, legal_moves},
    transform::{mirrored, rotated_180},
    types::{Color, Move, Piece, PieceKind, Position, Square, START_FEN},
    validate::ValidationResult,
};
use crate::game::{
    dto::{snapshot as game_snapshot_dto, GameSnapshot},
    nag::Nag,
    tree::GameTree,
};
use std::sync::{Mutex, OnceLock};

// ===================== 棋盘核心命令 =====================

/// 起始局面。
#[tauri::command]
pub fn board_startpos() -> PositionDto {
    let pos = parse_fen(START_FEN).expect("起始 FEN 必须可解析");
    PositionDto::from_position(&pos)
}

/// 解析 FEN 得到局面快照。
#[tauri::command]
pub fn board_from_fen(fen: String) -> Result<PositionDto, String> {
    let pos = parse_fen(&fen)?;
    Ok(PositionDto::from_position(&pos))
}

/// 某局面的全部合法着法（UCI 字符串列表）。
#[tauri::command]
pub fn board_legal_moves(fen: String) -> Result<Vec<String>, String> {
    let pos = parse_fen(&fen)?;
    Ok(legal_moves(&pos).iter().map(|m| m.uci()).collect())
}

/// 执行一步合法着法，返回新局面。
#[tauri::command]
pub fn board_make_move(fen: String, mv: String) -> Result<PositionDto, String> {
    let pos = parse_fen(&fen)?;
    let m = Move::parse_uci(&mv).ok_or_else(|| format!("非法着法格式：{mv}"))?;
    let next = apply_move(&pos, m).ok_or_else(|| format!("非法着法：{mv}"))?;
    Ok(PositionDto::from_position(&next))
}

/// 规则校验局面。
#[tauri::command]
pub fn board_validate(fen: String) -> Result<ValidationResult, String> {
    let pos = parse_fen(&fen)?;
    Ok(crate::board::validate::validate_position(&pos))
}

/// 视图变换：`180` 换边 / `mirror` 左右镜像。
#[tauri::command]
pub fn board_rotate(fen: String, mode: String) -> Result<PositionDto, String> {
    let pos = parse_fen(&fen)?;
    let next = match mode.as_str() {
        "180" => rotated_180(&pos),
        "mirror" => mirrored(&pos),
        _ => return Err(format!("未知旋转模式：{mode}")),
    };
    Ok(PositionDto::from_position(&next))
}

/// 局面编辑器：放置棋子。
#[tauri::command]
pub fn board_edit_set_piece(
    fen: String,
    square: String,
    color: String,
    kind: String,
) -> Result<PositionDto, String> {
    let mut pos = parse_fen(&fen)?;
    let sq = Square::parse_uci(&square).ok_or_else(|| format!("非法坐标：{square}"))?;
    let color = Color::from_name(&color).ok_or_else(|| format!("非法颜色：{color}"))?;
    let kind = PieceKind::from_name(&kind).ok_or_else(|| format!("非法棋子：{kind}"))?;
    pos.board[sq.rank as usize][sq.file as usize] = Some(Piece { color, kind });
    Ok(PositionDto::from_position(&pos))
}

/// 局面编辑器：清除棋子。
#[tauri::command]
pub fn board_edit_clear(fen: String, square: String) -> Result<PositionDto, String> {
    let mut pos = parse_fen(&fen)?;
    let sq = Square::parse_uci(&square).ok_or_else(|| format!("非法坐标：{square}"))?;
    pos.board[sq.rank as usize][sq.file as usize] = None;
    Ok(PositionDto::from_position(&pos))
}

/// 局面编辑器：切换先手方（`w` / `b`）。
#[tauri::command]
pub fn board_edit_set_side(fen: String, side: String) -> Result<PositionDto, String> {
    let mut pos = parse_fen(&fen)?;
    pos.side_to_move = match side.as_str() {
        "w" => Color::Red,
        "b" => Color::Black,
        _ => return Err(format!("非法先手方：{side}")),
    };
    Ok(PositionDto::from_position(&pos))
}

/// 局面编辑器：清空棋盘。
#[tauri::command]
pub fn board_edit_clear_all() -> PositionDto {
    PositionDto::from_position(&Position::default())
}

/// 调试/测试用：输出局面 FEN（便于前端显示）。
#[tauri::command]
pub fn board_fen(fen: String) -> Result<String, String> {
    let pos = parse_fen(&fen)?;
    Ok(to_fen(&pos))
}

/// 依序应用一串着法（PV 预览用），返回最终局面。
#[tauri::command]
pub fn board_apply_moves(fen: String, moves: Vec<String>) -> Result<PositionDto, String> {
    let pos = parse_fen(&fen)?;
    let parsed: Vec<Move> = moves
        .iter()
        .map(|m| Move::parse_uci(m).ok_or_else(|| format!("非法着法格式：{m}")))
        .collect::<Result<_, _>>()?;
    let end = crate::board::rules::apply_moves(&pos, &parsed)?;
    Ok(PositionDto::from_position(&end))
}

// ===================== 棋谱树（Game Tree）命令 =====================

fn game_err<E: std::fmt::Display>(e: E) -> String {
    format!("{e}")
}

/// 应用级棋谱树状态（单文档会话；静态全局，避免在测试二进制中引入 tauri 运行时链接）。
fn game_tree() -> &'static Mutex<GameTree> {
    static TREE: OnceLock<Mutex<GameTree>> = OnceLock::new();
    TREE.get_or_init(|| Mutex::new(GameTree::new(START_FEN).expect("start FEN must parse")))
}

/// 新建棋谱树（空 fen 使用起始局面）。
#[tauri::command]
pub fn game_new(fen: String) -> Result<GameSnapshot, String> {
    let start = if fen.trim().is_empty() {
        START_FEN
    } else {
        fen.as_str()
    };
    let tree = GameTree::new(start).map_err(game_err)?;
    *game_tree().lock().map_err(game_err)? = tree;
    game_snapshot_dto(&*game_tree().lock().map_err(game_err)?).map_err(game_err)
}

/// 获取当前棋谱树快照。
#[tauri::command]
pub fn game_snapshot() -> Result<GameSnapshot, String> {
    game_snapshot_dto(&*game_tree().lock().map_err(game_err)?).map_err(game_err)
}

/// 在当前节点插入着法（UCI）。
#[tauri::command]
pub fn game_insert_move(mv: String) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    let m = Move::parse_uci(&mv).ok_or_else(|| format!("非法着法格式：{mv}"))?;
    tree.insert_move(m).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 跳转到任意节点。
#[tauri::command]
pub fn game_navigate(node_id: u64) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.set_current(node_id).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 上一步（父节点）。
#[tauri::command]
pub fn game_previous() -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.previous().map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 下一步（主线首子）。
#[tauri::command]
pub fn game_next() -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.next_move().map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 悔棋（回到父节点，可重做）。
#[tauri::command]
pub fn game_undo() -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.undo().map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 重做。
#[tauri::command]
pub fn game_redo() -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.redo().map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 回到起点。
#[tauri::command]
pub fn game_go_to_start() -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.go_to_start().map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 走到终点（主线末尾）。
#[tauri::command]
pub fn game_go_to_end() -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.go_to_end().map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 删除一支变例（节点为其父节点的非首个子节点）。
#[tauri::command]
pub fn game_delete_variation(node_id: u64) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.delete_variation(node_id).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 设置指定节点注释（H1：修改棋谱节点数据的命令必须显式传 node_id，不依赖 current）。
#[tauri::command]
pub fn game_set_comment(node_id: u64, comment: String) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.set_comment_at(node_id, comment).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 为指定节点添加/移除注释符号（NAG，H1：显式 node_id）。
#[tauri::command]
pub fn game_set_nag(node_id: u64, nag: String, add: bool) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    let symbol = Nag::from_symbol(&nag).ok_or_else(|| format!("未知注释符号：{nag}"))?;
    tree.set_nag_at(node_id, symbol, add).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 把一支变例提升为主线（M2）。
#[tauri::command]
pub fn game_promote_variation(node_id: u64) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.promote_variation(node_id).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 调整变例顺序（M2）：把 parent 的 children[from] 移到 children[to]（from/to >= 1）。
#[tauri::command]
pub fn game_reorder_variation(
    parent_id: u64,
    from: usize,
    to: usize,
) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.reorder_variation(parent_id, from, to)
        .map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

// ===================== 引擎分析（Engine Analysis）命令 =====================

use crate::engine::manager::EngineManager;
use crate::engine::types::{EngineConfig, EngineStatus, GoParams};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

struct EngineInstance {
    mgr: Arc<EngineManager>,
    forwarder: tokio::task::JoinHandle<()>,
}

fn engine_instance() -> &'static Mutex<Option<EngineInstance>> {
    static INSTANCE: OnceLock<Mutex<Option<EngineInstance>>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(None))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatusDto {
    pub status: &'static str,
    pub engine_id: Option<String>,
}

fn status_name(s: EngineStatus) -> &'static str {
    match s {
        EngineStatus::Stopped => "stopped",
        EngineStatus::Ready => "ready",
        EngineStatus::Searching => "searching",
        EngineStatus::Crashed => "crashed",
    }
}

/// 启动引擎；`program` 为空时回退到 `PIKAFISH_BIN` 环境变量。
#[tauri::command]
pub async fn engine_start(program: Option<String>, app: AppHandle) -> Result<String, String> {
    let program = match program {
        Some(p) if !p.trim().is_empty() => p,
        _ => std::env::var("PIKAFISH_BIN")
            .map_err(|_| "未指定引擎程序路径（请传入 program 或设置 PIKAFISH_BIN）".to_string())?,
    };
    // 回收旧实例（先取走，避免持锁跨 await）
    let old = {
        let mut guard = engine_instance().lock().map_err(|e| e.to_string())?;
        guard.take()
    };
    if let Some(old) = old {
        let _ = old.mgr.quit().await;
        old.forwarder.abort();
    }

    let config = EngineConfig {
        program: PathBuf::from(program),
        args: Vec::new(),
        env: HashMap::new(),
        cwd: None,
        handshake_timeout: Duration::from_secs(10),
    };
    let mgr = Arc::new(EngineManager::spawn(config).await?);
    let mut rx = mgr.subscribe();
    let app2 = app.clone();
    let forwarder = tokio::spawn(async move {
        while let Ok(ev) = rx.recv().await {
            let _ = app2.emit("engine://event", ev);
        }
    });
    let engine_id = mgr.engine_id().unwrap_or_default();
    let mut guard = engine_instance().lock().map_err(|e| e.to_string())?;
    *guard = Some(EngineInstance { mgr, forwarder });
    Ok(engine_id)
}

#[tauri::command]
pub async fn engine_status() -> Result<EngineStatusDto, String> {
    let guard = engine_instance().lock().map_err(|e| e.to_string())?;
    let Some(inst) = guard.as_ref() else {
        return Ok(EngineStatusDto {
            status: "stopped",
            engine_id: None,
        });
    };
    Ok(EngineStatusDto {
        status: status_name(inst.mgr.status()),
        engine_id: inst.mgr.engine_id(),
    })
}

#[tauri::command]
pub async fn engine_set_option(name: String, value: Option<String>) -> Result<(), String> {
    let mgr = {
        let guard = engine_instance().lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("引擎未启动")?.mgr.clone()
    };
    mgr.set_option(&name, value.as_deref()).await
}

#[tauri::command]
pub async fn engine_set_position_and_go(
    fen: String,
    moves: Vec<String>,
    params: GoParams,
) -> Result<(), String> {
    let mgr = {
        let guard = engine_instance().lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("引擎未启动")?.mgr.clone()
    };
    mgr.set_position_and_go(Some(&fen), &moves, params).await
}

#[tauri::command]
pub async fn engine_stop() -> Result<(), String> {
    let mgr = {
        let guard = engine_instance().lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("引擎未启动")?.mgr.clone()
    };
    mgr.stop().await
}

#[tauri::command]
pub async fn engine_restart() -> Result<(), String> {
    let mgr = {
        let guard = engine_instance().lock().map_err(|e| e.to_string())?;
        guard.as_ref().ok_or("引擎未启动")?.mgr.clone()
    };
    mgr.restart().await
}

#[tauri::command]
pub async fn engine_quit() -> Result<(), String> {
    let old = {
        let mut guard = engine_instance().lock().map_err(|e| e.to_string())?;
        guard.take()
    };
    if let Some(old) = old {
        let _ = old.mgr.quit().await;
        old.forwarder.abort();
    }
    Ok(())
}

// ===================== PGN 导入导出命令 =====================

/// 从 PGN 文本导入棋谱并替换当前棋谱树。
#[tauri::command]
pub fn pgn_import(pgn: String) -> Result<GameSnapshot, String> {
    let tree = crate::io::pgn::import(&pgn).map_err(game_err)?;
    *game_tree().lock().map_err(game_err)? = tree;
    game_snapshot_dto(&*game_tree().lock().map_err(game_err)?).map_err(game_err)
}

/// 导出当前棋谱树为 PGN 文本。
#[tauri::command]
pub fn pgn_export() -> Result<String, String> {
    Ok(crate::io::pgn::export(
        &*game_tree().lock().map_err(game_err)?,
    ))
}
