use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use arc_swap::ArcSwap;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value as YamlValue;

#[allow(dead_code)]
const CLASH_CONFIG_FILE: &str = "config.yaml";

/// Clash 核心配置结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ClashConfigData {
    /// 混合端口
    pub mixed_port: Option<u16>,
    /// HTTP端口
    pub http_port: Option<u16>,
    /// SOCKS5端口
    pub socks_port: Option<u16>,
    /// 允许局域网
    pub allow_lan: Option<bool>,
    /// 日志级别
    pub log_level: Option<String>,
    /// 外部控制器
    pub external_controller: Option<String>,
    /// IPv6 支持
    pub ipv6: Option<bool>,
    /// DNS 设置
    pub dns: Option<DnsConfig>,
    /// 代理配置
    pub proxies: Option<YamlValue>,
    /// 代理组
    pub proxy_groups: Option<YamlValue>,
    /// 规则
    pub rules: Option<Vec<YamlValue>>,
}

/// DNS 配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DnsConfig {
    pub enable: Option<bool>,
    pub nameserver: Option<Vec<String>>,
    pub nameserver_policy: Option<YamlValue>,
    pub fallback: Option<Vec<String>>,
    pub fallback_policy: Option<YamlValue>,
}

/// Clash 配置管理器
#[derive(Debug)]
#[allow(dead_code)]
pub struct ClashConfig {
    /// 配置数据
    data: ArcSwap<ClashConfigData>,
    /// 配置文件路径
    path: PathBuf,
    /// 最后修改时间
    pub last_update: Option<DateTime<Local>>,
}

impl ClashConfig {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            data: ArcSwap::new(Arc::new(ClashConfigData::default())),
            path: Self::get_config_path(),
            last_update: None,
        }
    }

    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(CLASH_CONFIG_FILE);
        path
    }

    pub async fn load() -> anyhow::Result<Self> {
        let mut config = Self::new();
        if config.path.exists() {
            let content = tokio::fs::read_to_string(&config.path)
                .await
                .context("Failed to read Clash config file")?;

            let data = serde_yaml_ng::from_str(&content).context("Failed to parse Clash config file")?;

            config.data.store(Arc::new(data));
            config.last_update = std::fs::metadata(&config.path)
                .ok()
                .and_then(|meta| meta.modified().ok().map(|time| DateTime::<Local>::from(time)));
        }
        Ok(config)
    }

    #[allow(dead_code)]
    pub async fn save(&mut self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }
        let data = self.data.load_full();
        let content = serde_yaml_ng::to_string(&data).context("Failed to serialize Clash config")?;

        tokio::fs::write(&self.path, content)
            .await
            .context("Failed to write Clash config file")?;

        self.last_update = Some(Local::now());
        Ok(())
    }

    /// 获取当前配置
    #[allow(dead_code)]
    pub fn get(&self) -> Arc<ClashConfigData> {
        self.data.load().clone()
    }

    /// 更新配置
    #[allow(dead_code)]
    pub fn patch(&self, new_config: ClashConfigData) {
        self.data.store(Arc::new(new_config));
    }

    /// 获取混合端口
    #[allow(dead_code)]
    pub fn get_mixed_port(&self) -> u16 {
        self.data.load().mixed_port.unwrap_or(7890)
    }

    /// 设置混合端口
    #[allow(dead_code)]
    pub fn set_mixed_port(&self, port: u16) {
        let mut data = (*self.data.load_full()).clone();
        data.mixed_port = Some(port);
        self.data.store(Arc::new(data));
    }

    /// 获取外部控制器地址
    #[allow(dead_code)]
    pub fn get_external_controller(&self) -> String {
        self.data
            .load()
            .external_controller
            .clone()
            .unwrap_or_else(|| "127.0.0.1:9090".to_string())
    }

    /// 检查是否允许局域网
    #[allow(dead_code)]
    pub fn is_allow_lan(&self) -> bool {
        self.data.load().allow_lan.unwrap_or(false)
    }

    /// 获取日志级别
    #[allow(dead_code)]
    pub fn get_log_level(&self) -> String {
        self.data.load().log_level.clone().unwrap_or_else(|| "info".to_string())
    }
}

impl Default for ClashConfig {
    fn default() -> Self {
        Self::new()
    }
}
