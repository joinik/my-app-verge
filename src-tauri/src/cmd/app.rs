/// App related commands
use crate::config::{APP_CONFIG, verge::VergeConfigData};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewWindow};

/// Get verge config
#[tauri::command]
pub async fn get_verge_config() -> Result<Arc<VergeConfigData>, String> {
    Ok(APP_CONFIG.load().get().clone())
}

/// patch verge config
#[tauri::command]
pub async fn patch_verge_config(config: VergeConfigData) -> Result<(), String> {
    // 从全局配置中克隆一个可变的副本
    let mut app_config = (*APP_CONFIG.load_full()).clone();

    // 在副本上应用更改
    app_config.patch(config);

    // 保存副本到磁盘，这将更新时间戳
    app_config.save().await.map_err(|e| e.to_string())?;

    // 用更新后的副本替换全局配置
    APP_CONFIG.store(Arc::new(app_config));

    Ok(())
}

/// 重启应用
#[tauri::command]
pub async fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}

#[tauri::command]
pub async fn exit_app(app: AppHandle) -> Result<(), String> {
    // 清理资源
    // 停止 Clash核心
    // 保存配置
    std::process::exit(0);
}

/// 获取应用目录
#[tauri::command]
pub fn get_app_dir() -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-new-app");
    Ok(path.to_str().ok_or("Invalid path")?.to_string())
}

/// 获取日志目录
#[tauri::command]
pub fn get_logs_dir() -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-new-app")
        .join("logs");
    Ok(path.to_str().ok_or("Invalid path")?.to_string())
}

/// 打开目录
#[tauri::command]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_count: u32,
    pub used_memory: u64,
    pub total_memory: u64,
}

/// 获取系统信息
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_all();
        Ok(SystemInfo {
            os_name: System::name().unwrap_or_else(|| "Unknown".into()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".into()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
            cpu_count: System::cpu_count(),
            used_memory: sys.used_memory(),
            total_memory: sys.total_memory(),
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        // 简化实现
        Ok(SystemInfo {
            os_name: "Unknown".into(),
            os_version: "Unknown".into(),
            kernel_version: "Unknown".into(),
            cpu_count: 4,
            total_memory: 8_000_000_000,
            used_memory: 4_000_000_000,
        })
    }
}



