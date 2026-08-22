// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

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
        .invoke_handler(tauri::generate_handler![version, greet])
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
