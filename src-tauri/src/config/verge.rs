use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

const VERGE_CONFIG_FILE: &str = "verge.yaml";

/// Verge 应用配置数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct VergeConfigData {
    /// 主题模式：light, dark, system
    pub theme_mode: Option<String>,
    /// 语言设置
    pub language: Option<String>,
    /// 是否启用自动启动
    pub enable_auto_launch: Option<bool>,
    /// 是否启用系统代理
    pub enable_system_proxy: Option<bool>,
    /// 系统代理绕过列表
    pub system_proxy_bypass: Option<Vec<String>>,
    /// 是否启用 TUN 模式
    pub enable_tun_mode: Option<bool>,
    /// TUN 设备文件描述符
    pub tun_fd: Option<i32>,
    /// 是否启用自动检查更新
    pub enable_auto_check_update: Option<bool>,
    /// 自动检查更新间隔（秒）
    pub auto_check_update_interval: Option<u64>,
    /// 是否启用全局热键
    pub enable_global_hotkey: Option<bool>,
    /// 系统代理端口
    pub system_proxy_port: Option<u16>,
    /// Clash 核心路径
    pub clash_core_path: Option<String>,
    /// 是否启用服务模式
    pub enable_service_mode: Option<bool>,
    /// 是否启用 IPv6
    pub enable_ipv6: Option<bool>,
    /// 是否显示系统托盘图标
    pub show_tray_icon: Option<bool>,
    /// 启动时最小化到托盘
    pub start_minimized: Option<bool>,
    /// 是否在启动时连接
    pub connect_on_start: Option<bool>,
    /// 默认配置文件 ID
    pub default_profile_id: Option<String>,
    /// 是否启用智能路由
    pub enable_smart_routing: Option<bool>,
    /// 是否启用脚本模式
    pub enable_script_mode: Option<bool>,
    /// 日志级别
    pub log_level: Option<String>,
}

/// Verge 应用配置管理器
#[allow(dead_code)]
pub struct AppConfig {
    /// 配置数据
    data: ArcSwap<VergeConfigData>,
    /// 配置文件路径
    path: PathBuf,
    /// 最后修改时间
    pub last_update: Option<DateTime<Local>>,
}

impl Clone for AppConfig {
    fn clone(&self) -> Self {
        Self {
            data: ArcSwap::new(self.data.load().clone()),
            path: self.path.clone(),
            last_update: self.last_update.clone(),
        }
    }
}

impl AppConfig {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            data: ArcSwap::new(Arc::new(VergeConfigData::default())),
            path: Self::get_config_path(),
            last_update: None,
        }
    }

    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(VERGE_CONFIG_FILE);
        path
    }

    /// 从文件加载配置
    pub async fn load() -> Result<Self> {
        #[allow(unused_mut)]
        let mut config = Self::new();

        if config.path.exists() {
            let content = tokio::fs::read_to_string(&config.path)
                .await
                .context("Failed to read Verge config file")?;

            let data: VergeConfigData =
                serde_yaml_ng::from_str(&content).context("Failed to parse Verge config file")?;

            config.data.store(Arc::new(data));
        }

        Ok(config)
    }

    /// 保存配置到文件
    pub async fn save(&self) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }

        let data = self.data.load();
        let content = serde_yaml_ng::to_string(&**data).context("Failed to serialize Verge config")?;

        tokio::fs::write(&self.path, content)
            .await
            .context("Failed to write Verge config file")?;

        Ok(())
    }

    /// 获取当前配置
    #[allow(dead_code)]
    pub fn get(&self) -> Arc<VergeConfigData> {
        self.data.load().clone()
    }

    /// 更新配置
    pub fn patch(&self, new_config: VergeConfigData) {
        self.data.store(Arc::new(new_config));
    }

    /// 获取主题模式
    #[allow(dead_code)]
    pub fn get_theme_mode(&self) -> String {
        self.data
            .load()
            .theme_mode
            .clone()
            .unwrap_or_else(|| "system".to_string())
    }

    /// 设置主题模式
    #[allow(dead_code)]
    pub fn set_theme_mode(&self, mode: &str) {
        self.data.rcu(|old_data_arc| {
            let mut new_data = (**old_data_arc).clone();
            new_data.theme_mode = Some(mode.to_string());
            Arc::new(new_data)
        });
    }

    /// 是否启用系统代理
    #[allow(dead_code)]
    pub fn is_system_proxy_enabled(&self) -> bool {
        self.data.load().enable_system_proxy.unwrap_or(false)
    }

    /// 是否启用 TUN 模式
    #[allow(dead_code)]
    pub fn is_tun_mode_enabled(&self) -> bool {
        self.data.load().enable_tun_mode.unwrap_or(false)
    }

    /// 是否启用自动启动
    #[allow(dead_code)]
    pub fn is_auto_launch_enabled(&self) -> bool {
        self.data.load().enable_auto_launch.unwrap_or(false)
    }

    /// 是否启用全局热键
    #[allow(dead_code)]
    pub fn is_global_hotkey_enabled(&self) -> bool {
        self.data.load().enable_global_hotkey.unwrap_or(true)
    }

    /// 获取 Clash 核心路径
    #[allow(dead_code)]
    pub fn get_clash_core_path(&self) -> Option<String> {
        self.data.load().clash_core_path.clone()
    }

    /// 设置 Clash 核心路径
    #[allow(dead_code)]
    pub fn set_clash_core_path(&self, path: &str) {
        self.data.rcu(|old_data_arc| {
            let mut new_data = (**old_data_arc).clone();
            new_data.clash_core_path = Some(path.to_string());
            Arc::new(new_data)
        });
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_creation() {
        let config = AppConfig::new();
        assert!(config.get().theme_mode.is_none());
    }

    #[tokio::test]
    async fn test_theme_mode() {
        let config = AppConfig::new();
        config.set_theme_mode("dark");
        assert_eq!(config.get_theme_mode(), "dark");
    }

    #[tokio::test]
    async fn test_auto_launch() {
        let config = AppConfig::new();
        assert!(!config.is_auto_launch_enabled());
    }
}
