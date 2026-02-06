/// App related commands
use anyhow::Result;
use tauri::AppHandle;

/// Check if app is admin
#[tauri::command]
pub fn app_is_admin() -> bool {
    false
}

/// Exit app
#[tauri::command]
pub fn exit_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}
