// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod board;
pub mod book;
mod commands;
pub mod engine;
pub mod game;
pub mod io;

/// 返回应用版本（来自 Cargo.toml）。
#[tauri::command]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 示例命令：验证 React ↔ Rust IPC 通路。
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            version,
            greet,
            commands::board_startpos,
            commands::board_from_fen,
            commands::board_legal_moves,
            commands::board_make_move,
            commands::board_validate,
            commands::board_rotate,
            commands::board_edit_set_piece,
            commands::board_edit_clear,
            commands::board_edit_set_side,
            commands::board_edit_clear_all,
            commands::board_fen,
            commands::game_new,
            commands::game_snapshot,
            commands::game_insert_move,
            commands::game_navigate,
            commands::game_previous,
            commands::game_next,
            commands::game_undo,
            commands::game_redo,
            commands::game_go_to_start,
            commands::game_go_to_end,
            commands::game_delete_variation,
            commands::game_promote_variation,
            commands::game_reorder_variation,
            commands::game_set_comment,
            commands::game_set_nag,
            commands::board_apply_moves,
            commands::engine_start,
            commands::engine_status,
            commands::engine_set_option,
            commands::engine_set_position_and_go,
            commands::engine_stop,
            commands::engine_restart,
            commands::engine_quit,
            commands::pgn_import,
            commands::pgn_export,
            commands::book_lookup,
            commands::book_recommend,
            commands::book_auto_move,
            commands::io_import,
            commands::io_export
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        assert!(version().starts_with("0."));
    }

    #[test]
    fn greet_returns_hello() {
        assert_eq!(greet("Pika"), "Hello, Pika! You've been greeted from Rust!");
    }
}
