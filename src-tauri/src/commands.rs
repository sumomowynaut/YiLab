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

/// 设置当前节点注释。
#[tauri::command]
pub fn game_set_comment(comment: String) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    tree.set_comment(comment).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}

/// 为当前节点添加/移除注释符号（NAG）。
#[tauri::command]
pub fn game_set_nag(nag: String, add: bool) -> Result<GameSnapshot, String> {
    let mut tree = game_tree().lock().map_err(game_err)?;
    let symbol = Nag::from_symbol(&nag).ok_or_else(|| format!("未知注释符号：{nag}"))?;
    tree.set_nag(symbol, add).map_err(game_err)?;
    game_snapshot_dto(&tree).map_err(game_err)
}
