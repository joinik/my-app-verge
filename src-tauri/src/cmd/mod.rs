/// Command handlers for Tauri IPC
pub mod app;
pub mod system;

/// Greet command
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}