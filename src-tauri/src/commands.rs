//! Tauri IPC 命令层：棋盘核心（board）与棋谱树（game）的薄封装。

use crate::analysis::{AnalysisConfig, AutoAnalyzer, MoveAssessment, PlannedMove};
use crate::board::{
    dto::PositionDto,
    fen::{parse_fen, to_fen},
    rules::{apply_move, legal_moves},
    transform::{mirrored, rotated_180},
    types::{Color, Move, Piece, PieceKind, Position, Square, START_FEN},
    validate::ValidationResult,
};
use crate::book::{dto::BookMoveDto, local::LocalBookProvider, BookChain, BookMove, BookStrategy};
use crate::game::{
    dto::{snapshot as game_snapshot_dto, GameSnapshot},
    nag::Nag,
    tree::{GameTree, NodeId},
};
use crate::ocr::dto::OcrResultDto;
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

/// 把 FEN 局面中的一步着法（UCI）转成中文纵线制记谱（如 炮二平五）。
#[tauri::command]
pub fn board_move_to_chinese(fen: String, uci: String) -> Result<String, String> {
    let pos = parse_fen(&fen)?;
    let mv = Move::parse_uci(&uci).ok_or_else(|| format!("非法着法格式：{uci}"))?;
    Ok(crate::board::chinese::move_to_chinese(&pos, &mv))
}

/// 把从 `fen` 局面开始的一串着法（UCI）依次转成中文纵线制记谱。
#[tauri::command]
pub fn board_moves_to_chinese(fen: String, moves: Vec<String>) -> Result<Vec<String>, String> {
    let mut pos = parse_fen(&fen)?;
    let mut out = Vec::with_capacity(moves.len());
    for uci in moves {
        let mv = Move::parse_uci(&uci).ok_or_else(|| format!("非法着法格式：{uci}"))?;
        out.push(crate::board::chinese::move_to_chinese(&pos, &mv));
        pos = apply_move(&pos, mv).ok_or_else(|| format!("非法着法：{uci}"))?;
    }
    Ok(out)
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

use crate::engine::manager::{discover_eval_file, EngineManager};
use crate::engine::types::{EngineConfig, EngineStatus, GoParams};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

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

/// 扫描常见目录中的 Pikafish 可执行文件（.exe），供引擎选择下拉框使用。
#[tauri::command]
pub fn engine_discover_binaries() -> Vec<String> {
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.clone());
        roots.push(cwd.join("engine"));
        roots.push(cwd.join("bin"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.to_path_buf());
            roots.push(parent.join("engine"));
        }
    }
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for root in roots {
        collect_pikafish_exes(&root, 0, &mut out, &mut seen);
    }
    out.sort();
    out
}

fn collect_pikafish_exes(
    dir: &std::path::Path,
    depth: u32,
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    if depth > 2 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_pikafish_exes(&path, depth + 1, out, seen);
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("pikafish") && lower.ends_with(".exe") {
            if let Ok(canon) = path.canonicalize() {
                let s = canon.to_string_lossy().to_string();
                if seen.insert(s.clone()) {
                    out.push(s);
                }
            }
        }
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

    // 自动发现 NNUE 权重（exe 同目录或上一级目录），并把工作目录设为权重所在目录：
    // 引擎用相对默认值 `pikafish.nnue` 从 cwd 加载，避免传递含中文/空格的绝对路径。
    let eval_file = discover_eval_file(&PathBuf::from(&program));
    let cwd = eval_file
        .as_ref()
        .and_then(|p| p.parent())
        .map(|d| d.to_path_buf());
    let config = EngineConfig {
        program: PathBuf::from(program),
        args: Vec::new(),
        env: HashMap::new(),
        cwd,
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

// ===================== 开局库（Book）命令 =====================

/// 应用级开局库状态（本地优先 + 可选云库；云库失败静默回退）。
fn book_chain() -> &'static Mutex<BookChain> {
    static BOOK: OnceLock<Mutex<BookChain>> = OnceLock::new();
    BOOK.get_or_init(|| Mutex::new(BookChain::local_only(Box::new(LocalBookProvider::new()))))
}

fn parse_strategy(s: &str) -> Result<BookStrategy, String> {
    BookStrategy::from_name(s).ok_or_else(|| format!("未知走库策略：{s}"))
}

/// 查询当前局面的候选着法（本地优先，未命中回退云库）。
#[tauri::command]
pub fn book_lookup() -> Result<Vec<BookMoveDto>, String> {
    let tree = game_tree().lock().map_err(game_err)?;
    let pos = tree.restore_position(tree.current_id()).map_err(game_err)?;
    let chain = book_chain().lock().map_err(game_err)?;
    Ok(chain.lookup(&pos).iter().map(BookMoveDto::from).collect())
}

/// 查询当前局面的推荐着法（策略：best_score / most_popular / first）。
#[tauri::command]
pub fn book_recommend(strategy: String) -> Result<Option<BookMoveDto>, String> {
    let strategy = parse_strategy(&strategy)?;
    let tree = game_tree().lock().map_err(game_err)?;
    let pos = tree.restore_position(tree.current_id()).map_err(game_err)?;
    let chain = book_chain().lock().map_err(game_err)?;
    Ok(chain
        .recommend(&pos, strategy)
        .map(|b| BookMoveDto::from(&b)))
}

/// 自动走库：把推荐着法插入当前棋谱树。
/// `max_plies` 为「脱库步数」（半回合数），超过则不走库（未命中/脱库均返回 applied=None）。
#[tauri::command]
pub fn book_auto_move(strategy: String, max_plies: Option<u32>) -> Result<BookAutoMoveDto, String> {
    let strategy = parse_strategy(&strategy)?;
    let mut tree = game_tree().lock().map_err(game_err)?;
    let pos = tree.restore_position(tree.current_id()).map_err(game_err)?;
    let plies = tree.current_plies();
    let recommended = book_chain()
        .lock()
        .map_err(game_err)?
        .recommend_book(&pos, strategy, plies, max_plies);
    let applied = match recommended {
        Some(BookMove { mv, .. }) => {
            tree.insert_move(mv).map_err(game_err)?;
            Some(mv.uci())
        }
        None => None,
    };
    let snapshot = game_snapshot_dto(&tree).map_err(game_err)?;
    Ok(BookAutoMoveDto { applied, snapshot })
}

/// 自动走库命令结果。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookAutoMoveDto {
    /// 实际走出的着法（UCI）；开局库未命中时为 None。
    pub applied: Option<String>,
    pub snapshot: GameSnapshot,
}

// ===================== 导入导出（I/O）命令 =====================

/// 从文本导入棋谱（format: pgn / fen；空 format 时按内容嗅探）。
#[tauri::command]
pub fn io_import(format: String, text: String) -> Result<GameSnapshot, String> {
    let format = if format.trim().is_empty() {
        crate::io::sniff(&text)
    } else {
        crate::io::Format::from_name(&format).ok_or_else(|| format!("未知格式：{format}"))?
    };
    let tree = crate::io::codec(format).parse(&text)?;
    *game_tree().lock().map_err(game_err)? = tree;
    game_snapshot_dto(&*game_tree().lock().map_err(game_err)?).map_err(game_err)
}

/// 导出当前棋谱树为文本（format: pgn / fen）。
#[tauri::command]
pub fn io_export(format: String) -> Result<String, String> {
    let format =
        crate::io::Format::from_name(&format).ok_or_else(|| format!("未知格式：{format}"))?;
    let tree = game_tree().lock().map_err(game_err)?;
    crate::io::codec(format).serialize(&tree)
}

// ===================== 截图识别（OCR）命令 =====================

/// 从图片字节识别局面（视觉模型只识别；棋规校验在本地 Rust 完成）。
#[tauri::command]
pub fn ocr_recognize(image: Vec<u8>) -> Result<OcrResultDto, String> {
    const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024; // 20MB
    if image.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "image too large: {} bytes (max {} bytes)",
            image.len(),
            MAX_IMAGE_BYTES
        ));
    }
    let input = crate::ocr::OcrInput { image };
    let engine = crate::ocr::template::TemplateRecognizer::new();
    let output = crate::ocr::recognize(&engine, &input).map_err(|e| e.to_string())?;
    Ok(OcrResultDto::from(&output))
}

// ===================== 自动复盘（Analysis）命令 =====================

fn analyzer() -> &'static AutoAnalyzer {
    static ANALYZER: OnceLock<AutoAnalyzer> = OnceLock::new();
    ANALYZER.get_or_init(AutoAnalyzer::new)
}

fn analysis_forwarder() -> &'static Mutex<Option<tokio::task::JoinHandle<()>>> {
    static F: OnceLock<Mutex<Option<tokio::task::JoinHandle<()>>>> = OnceLock::new();
    F.get_or_init(|| Mutex::new(None))
}

/// 确保有一个事件转发任务把分析事件推到前端（幂等）。
fn ensure_analysis_forwarder(app: AppHandle) {
    let mut guard = analysis_forwarder()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        let mut rx = analyzer().subscribe();
        *guard = Some(tokio::spawn(async move {
            while let Ok(ev) = rx.recv().await {
                let _ = app.emit("analysis://event", ev);
            }
        }));
    }
}

/// 单步评估 DTO。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveAssessmentDto {
    pub node_id: u64,
    pub mv: String,
    pub best_move: String,
    pub eval_before_cp: i32,
    pub eval_after_cp: i32,
    pub loss_cp: i32,
    pub depth: u32,
    pub pv: Vec<String>,
    pub category: String,
}

impl From<&MoveAssessment> for MoveAssessmentDto {
    fn from(a: &MoveAssessment) -> Self {
        MoveAssessmentDto {
            node_id: a.node_id,
            mv: a.mv.clone(),
            best_move: a.best_move.clone(),
            eval_before_cp: a.eval_before_cp,
            eval_after_cp: a.eval_after_cp,
            loss_cp: a.loss_cp,
            depth: a.depth,
            pv: a.pv.clone(),
            category: a.category.name().to_string(),
        }
    }
}

/// 自动复盘状态 DTO。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisStatusDto {
    pub status: String,
    pub progress: usize,
    pub total: usize,
    pub assessments: Vec<MoveAssessmentDto>,
}

fn analysis_dto() -> AnalysisStatusDto {
    let (status, progress, total, assessments) = analyzer().snapshot();
    AnalysisStatusDto {
        status: status.name().to_string(),
        progress,
        total,
        assessments: assessments.iter().map(MoveAssessmentDto::from).collect(),
    }
}

/// 快照当前棋谱主线（从棋谱树读取，短临界区）。
fn plan_from_tree() -> Result<(String, Vec<PlannedMove>), String> {
    let tree = game_tree().lock().map_err(game_err)?;
    let mainline = tree.main_line();
    let mut plan = Vec::new();
    for id in mainline.iter().skip(1) {
        let n = tree.node(*id).map_err(game_err)?;
        let mv = n.mv.ok_or("主线节点缺少着法")?;
        plan.push(PlannedMove {
            node_id: *id,
            mv: mv.uci(),
            is_red: n.is_red(),
        });
    }
    Ok((tree.startpos.clone(), plan))
}

/// 获取当前引擎管理器（克隆 Arc，不跨 await 持锁）。
fn current_engine() -> Result<std::sync::Arc<EngineManager>, String> {
    let guard = engine_instance().lock().map_err(game_err)?;
    guard
        .as_ref()
        .map(|i| i.mgr.clone())
        .ok_or_else(|| "引擎未启动".to_string())
}

/// 开始自动复盘（重新分析 = 再次调用）。
#[tauri::command]
pub async fn analysis_start(
    depth: Option<u32>,
    movetime_ms: Option<u64>,
    app: AppHandle,
) -> Result<AnalysisStatusDto, String> {
    if depth.is_none() && movetime_ms.is_none() {
        return Err("请指定 depth 或 movetime_ms".to_string());
    }
    let (startpos, moves) = plan_from_tree()?;
    let mgr = current_engine()?;
    let config = AnalysisConfig {
        depth,
        movetime_ms,
        ..AnalysisConfig::default()
    };
    ensure_analysis_forwarder(app);
    analyzer().start(mgr, startpos, moves, config);
    Ok(analysis_dto())
}

/// 停止（暂停）自动复盘：先停引擎当前搜索，再暂停运行器。
#[tauri::command]
pub async fn analysis_stop() -> Result<(), String> {
    if let Ok(mgr) = current_engine() {
        let _ = mgr.stop().await;
    }
    analyzer().stop();
    Ok(())
}

/// 继续被暂停的自动复盘。
#[tauri::command]
pub async fn analysis_continue() -> Result<AnalysisStatusDto, String> {
    let mgr = current_engine()?;
    analyzer().resume(mgr);
    Ok(analysis_dto())
}

/// 查询自动复盘状态与已完成的评估。
#[tauri::command]
pub fn analysis_status() -> Result<AnalysisStatusDto, String> {
    Ok(analysis_dto())
}

// ===================== GIF 导出命令 =====================

/// 构建 GIF 请求参数（树 → startpos + moves）。
fn gif_request(
    tree: &GameTree,
    moves: Vec<String>,
    frame_delay_ms: u64,
    cell_size: u32,
    show_coordinates: bool,
    show_moves: bool,
) -> crate::gif_export::GifRequest {
    crate::gif_export::GifRequest {
        startpos: tree.startpos.clone(),
        moves,
        frame_delay_ms,
        cell_size,
        show_coordinates,
        show_moves,
    }
}

/// 主线着法（根 → children[0] 链）。
fn mainline_moves(tree: &GameTree) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = Some(tree.root);
    while let Some(id) = cur {
        let first = tree.node(id).ok().and_then(|n| n.children.first().copied());
        match first {
            Some(child) => {
                if let Some(mv) = tree.node(child).ok().and_then(|n| n.mv) {
                    out.push(mv.uci());
                }
                cur = Some(child);
            }
            None => break,
        }
    }
    out
}

/// 从根到某节点的着法（到达该节点的路径）。
fn path_to(tree: &GameTree, node: NodeId) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = Some(node);
    while let Some(id) = cur {
        let n = tree.node(id).map_err(game_err)?;
        if let Some(mv) = n.mv {
            out.push(mv.uci());
        }
        cur = n.parent;
    }
    out.reverse();
    Ok(out)
}

/// 从某节点出发沿其主线续着（含节点自身着法）。
fn line_from(tree: &GameTree, node: NodeId) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = Some(node);
    while let Some(id) = cur {
        let n = tree.node(id).map_err(game_err)?;
        if let Some(mv) = n.mv {
            out.push(mv.uci());
        }
        cur = n.children.first().copied();
    }
    Ok(out)
}

/// 导出「当前局面」GIF（单帧）。
#[tauri::command]
pub fn gif_export_current(
    frame_delay_ms: u64,
    cell_size: u32,
    show_coordinates: bool,
    show_moves: bool,
) -> Result<Vec<u8>, String> {
    let tree = game_tree().lock().map_err(game_err)?;
    let req = crate::gif_export::GifRequest {
        startpos: tree.current_node().fen.clone(),
        moves: Vec::new(),
        frame_delay_ms,
        cell_size,
        show_coordinates,
        show_moves,
    };
    crate::gif_export::export_gif(&req)
}

/// 导出「主线」GIF（startpos + 全部主线着法）。
#[tauri::command]
pub fn gif_export_mainline(
    frame_delay_ms: u64,
    cell_size: u32,
    show_coordinates: bool,
    show_moves: bool,
) -> Result<Vec<u8>, String> {
    let tree = game_tree().lock().map_err(game_err)?;
    let moves = mainline_moves(&tree);
    let req = gif_request(
        &tree,
        moves,
        frame_delay_ms,
        cell_size,
        show_coordinates,
        show_moves,
    );
    crate::gif_export::export_gif(&req)
}

/// 导出「指定变例」GIF（从根播放到分支点，再播放该变例）。
#[tauri::command]
pub fn gif_export_variation(
    node_id: u64,
    frame_delay_ms: u64,
    cell_size: u32,
    show_coordinates: bool,
    show_moves: bool,
) -> Result<Vec<u8>, String> {
    let tree = game_tree().lock().map_err(game_err)?;
    // 到变例起点之前（父位置）的路径 + 变例自身的着法
    let parent = tree.node(node_id).map_err(game_err)?.parent;
    let mut moves = match parent {
        Some(p) => path_to(&tree, p)?,
        None => Vec::new(),
    };
    moves.extend(line_from(&tree, node_id)?);
    let req = gif_request(
        &tree,
        moves,
        frame_delay_ms,
        cell_size,
        show_coordinates,
        show_moves,
    );
    crate::gif_export::export_gif(&req)
}

// ===================== 当前棋局保存 / 恢复（B3 最小持久化） =====================

/// 保存当前棋局到用户「文档」目录下的 弈研YiLab 文件夹，并返回完整路径。
#[tauri::command]
pub fn game_save(app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .document_dir()
        .map(|d| d.join("弈研YiLab"))
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("弈研棋谱.json");
    let tree = game_tree().lock().map_err(game_err)?;
    let json = crate::game::serialize::save_game(&tree).map_err(game_err)?;
    std::fs::write(&path, json).map_err(|e| format!("写入存档失败：{e}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// 从「文档/弈研YiLab」载入上次保存的棋局（恢复棋谱树 + 当前节点）。
#[tauri::command]
pub fn game_load(app: AppHandle) -> Result<GameSnapshot, String> {
    let dir = app
        .path()
        .document_dir()
        .map(|d| d.join("弈研YiLab"))
        .map_err(|e| e.to_string())?;
    let path = dir.join("弈研棋谱.json");
    let json = std::fs::read_to_string(&path).map_err(|e| format!("未找到存档：{e}"))?;
    let tree = crate::game::serialize::load_game(&json).map_err(game_err)?;
    *game_tree().lock().map_err(game_err)? = tree;
    game_snapshot_dto(&*game_tree().lock().map_err(game_err)?).map_err(game_err)
}

/// 在资源管理器中打开棋谱保存目录（文档/弈研YiLab）。
#[tauri::command]
pub fn open_save_dir(app: AppHandle) -> Result<(), String> {
    let dir = app
        .path()
        .document_dir()
        .map(|d| d.join("弈研YiLab"))
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    let _ = dir;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mv(uci: &str) -> Move {
        Move::parse_uci(uci).unwrap()
    }

    fn build_tree() -> GameTree {
        let mut tree = GameTree::new(START_FEN).unwrap();
        tree.insert_move(mv("h2e2")).unwrap();
        tree.insert_move(mv("h7e7")).unwrap();
        tree.insert_move(mv("h0g2")).unwrap();
        // h7e7 节点下的变例（红方走 b0c2 而非 h0g2）
        let n2 = tree.current_node().parent.unwrap();
        tree.set_current(n2).unwrap();
        tree.insert_move(mv("b0c2")).unwrap();
        tree.insert_move(mv("h9g7")).unwrap();
        tree
    }

    #[test]
    fn mainline_moves_walks_first_children() {
        let tree = build_tree();
        assert_eq!(mainline_moves(&tree), vec!["h2e2", "h7e7", "h0g2"]);
    }

    #[test]
    fn path_to_and_line_from_for_variation() {
        let tree = build_tree();
        // 结构：root → h2e2 → h7e7 → [h0g2(主线), b0c2(变例) → h9g7]
        let h2e2 = tree.node(tree.root).unwrap().children[0];
        let h7e7 = tree.node(h2e2).unwrap().children[0];
        let var = tree.node(h7e7).unwrap().children[1];
        assert_eq!(tree.node(var).unwrap().mv.unwrap().uci(), "b0c2");
        // 到变例起点之前（h7e7，含其自身着法）的路径
        let parent = tree.node(var).unwrap().parent.unwrap();
        assert_eq!(path_to(&tree, parent).unwrap(), vec!["h2e2", "h7e7"]);
        // 变例自身着法（b0c2 及其续着 h9g7）
        assert_eq!(line_from(&tree, var).unwrap(), vec!["b0c2", "h9g7"]);
    }
}
