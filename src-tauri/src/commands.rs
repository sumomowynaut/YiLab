//! Tauri IPC 命令层：棋盘核心（board）的薄封装。

use crate::board::{
    dto::PositionDto,
    fen::{parse_fen, to_fen},
    rules::{apply_move, legal_moves},
    transform::{mirrored, rotated_180},
    types::{Color, Move, Piece, PieceKind, Position, Square, START_FEN},
    validate::ValidationResult,
};

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
