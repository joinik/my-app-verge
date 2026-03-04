use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const VERGE_CONFIG_FILE: &str = "verge.yaml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VergeConfigData {
    pub theme_mode: Option<String>,
    pub language: Option<String>,
    pub enable_auto_launch: Option<bool>,
    pub enable_system_proxy: Option<bool>,
    pub system_proxy_bypass: Option<Vec<String>>,
    pub enable_tun_mode: Option<bool>,
    pub tun_fd: Option<i32>,
    pub enable_auto_check_update: Option<bool>,
    pub auto_check_update_interval: Option<u64>,
    pub enable_global_hotkey: Option<bool>,
    pub system_proxy_port: Option<u16>,
    pub clash_core_path: Option<String>,
    pub enable_service_mode: Option<bool>,
    pub enable_ipv6: Option<bool>,
    pub show_tray_icon: Option<bool>,
    pub start_minimized: Option<bool>,
    pub connect_on_start: Option<bool>,
    pub default_profile_id: Option<String>,
    pub enable_smart_routing: Option<bool>,
    pub enable_script_mode: Option<String>,
    pub log_level: Option<String>,
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("my-new-app");
    path.push(VERGE_CONFIG_FILE);
    path
}

pub async fn load_config() -> Result<VergeConfigData> {
    let path = get_config_path();

    if path.exists() {
        let content = tokio::fs::read_to_string(&path)
            .await
            .context("Failed to read config file")?;

        let data = serde_yaml_ng::from_str(&content).context("Failed to parse config file")?;

        Ok(data)
    } else {
        Ok(VergeConfigData::default())
    }
}

pub async fn save_config(data: &VergeConfigData) -> Result<()> {
    let path = get_config_path();

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create config directory")?;
    }

    let content = serde_yaml_ng::to_string(data)?;
    tokio::fs::write(&path, content)
        .await
        .context("Failed to write config file")?;

    Ok(())
}

pub async fn init_config() -> Result<()> {
    let data = load_config().await?;
    VERGE_CONFIG.store(Arc::new(data));
    Ok(())
}

pub static VERGE_CONFIG: Lazy<ArcSwap<VergeConfigData>> =
    Lazy::new(|| ArcSwap::new(Arc::new(VergeConfigData::default())));
