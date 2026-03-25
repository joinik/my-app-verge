/// App related commands
use crate::config::{APP_CONFIG, verge::VergeConfigData};

use std::sync::Arc;
use tauri::AppHandle;

/// Greet the user
#[tauri::command]
pub async fn greet(name: String) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Get verge config
#[tauri::command]
#[allow(dead_code)]
pub async fn get_verge_config() -> Result<Arc<VergeConfigData>, String> {
    Ok(APP_CONFIG.load().get().clone())
}

/// patch verge config
#[tauri::command]
#[allow(dead_code)]
pub async fn patch_verge_config(config: VergeConfigData) -> Result<(), String> {
    // 从全局配置中克隆一个可变的副本
    #[allow(unused_mut)]
    let mut app_config = (*APP_CONFIG.load_full()).clone();
    app_config.patch(config);
    match app_config.save().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// 重启应用
#[tauri::command]
#[allow(dead_code)]
pub async fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}

#[tauri::command]
#[allow(dead_code)]
pub async fn exit_app(_app: AppHandle) -> Result<(), String> {
    // 清理资源
    // 停止 Clash核心
    // 保存配置
    std::process::exit(0);
}

/// 获取应用目录
#[tauri::command]
#[allow(dead_code)]
pub fn get_app_dir() -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-new-app");
    Ok(path.to_str().ok_or("Invalid path")?.to_string())
}

/// 获取日志目录
#[tauri::command]
#[allow(dead_code)]
pub fn get_logs_dir() -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-new-app")
        .join("logs");
    Ok(path.to_str().ok_or("Invalid path")?.to_string())
}

/// 打开目录
#[tauri::command]
#[allow(dead_code)]
pub fn open_dir(path: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::{os::windows::process::CommandExt, process::Command};
        use winapi::um::winbase::CREATE_NO_WINDOW;
        Command::new("explorer.exe")
            .args(&[&path.to_str().unwrap_or("")])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
