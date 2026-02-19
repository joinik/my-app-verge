---
name: "clash-verge-rev-config"
description: "实现 clash-verge-rev 的核心配置系统，包括 Clash 配置、应用配置、配置文件管理等基础模块。从初学者角度出发，包含详细的代码实现、错误处理和测试验证。"
---

# Clash-Verge-Rev 核心配置系统实现

本技能专注于实现 clash-verge-rev 的核心配置系统，这是整个应用的基础架构。

## 学习目标

1. **理解配置管理** - 掌握 ArcSwap、serde 等核心库的使用
2. **文件操作** - 实现配置的序列化和持久化
3. **错误处理** - 使用 anyhow 进行优雅的错误处理
4. **异步编程** - 使用 tokio 处理 I/O 操作

## 实现步骤

### 步骤 1：更新 Cargo.toml

首先更新项目的依赖配置：

```toml
# Cargo.toml
[package]
name = "my-new-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.91"

[lib]
name = "my_new_app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[dependencies]
# 核心依赖
tauri = { version = "2", features = [
  "devtools",
  "tray-icon",
  "image-ico",
  "image-png",
] }

# 配置管理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml_ng = "0.1"
smartstring = { version = "1.0", features = ["serde"] }

# 异步支持
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# 并发控制
arc-swap = "1.8"
parking_lot = "0.12"

# 错误处理
anyhow = "1.0"
thiserror = "2.0"

# 日志系统
log = "0.4"
flexi_logger = "0.34"

# 工具库
dirs = "5.0"
tempfile = "3.0"
chrono = { version = "0.4", features = ["serde"] }
nanoid = "0.4"

# 加密
aes-gcm = "0.10"
base64 = "0.22"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dev-dependencies]
tempfile = "3.0"
```

### 步骤 2：创建配置目录结构

src-tauri/src/
├── config/
│ ├── mod.rs # 配置模块入口
│ ├── clash.rs # Clash 核心配置
│ ├── config.rs # 应用配置
│ ├── encrypt.rs # 加密工具
│ ├── prfitem.rs # 代理/规则项
│ └── profiles.rs # 配置文件管理
├── core/ # 核心功能
│ ├── mod.rs
│ ├── manager.rs # 进程管理
│ └── handle.rs # 全局句柄
├── cmd/ # Tauri 命令
│ ├── mod.rs
│ ├── app.rs # 应用命令
│ ├── system.rs # 系统命令
│ ├── clash.rs # Clash 命令
│ ├── proxy.rs # 代理命令
│ └── profile.rs # 配置命令
├── utils/ # 工具函数
│ ├── mod.rs
│ ├── dirs.rs # 目录操作
│ └── logging.rs # 日志系统
├── constants.rs # 常量定义
├── lib.rs # 库入口
└── main.rs # 主程序入口

### 步骤 3：实现配置模块

#### 3.1 config/mod.rs - 模块入口

```rust:src-tauri%2Fsrc%2Fconfig%2Fmod.rs
use std::sync::Arc;
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;

mod clash;
mod config;
mod encrypt;
mod prfitem;
pub mod profiles;
mod verge;

pub use self::{
    clash::{ClashConfig, CLASH_CONFIG},
    config::VergeConfig,
    encrypt::{decrypt, encrypt},
    prfitem::{ProxyItem, RuleItem},
    profiles::{Profiles, ProfilesExt},
    verge::{init_config, AppConfig, APP_CONFIG},
};

pub const DEFAULT_PAC: &str = r#"function FindProxyForURL(url, host) {
  return "PROXY 127.0.0.1:%mixed-port%; SOCKS5 127.0.0.1:%mixed-port%; DIRECT;";
}
"#;

/// 全局 Clash 配置实例
pub static CLASH_CONFIG: Lazy<ArcSwap<clash::ClashConfig>> = Lazy::new(|| {
    ArcSwap::new(Arc::new(clash::ClashConfig::new()))
});

/// 全局 Verge 应用配置实例
pub static APP_CONFIG: Lazy<ArcSwap<AppConfig>> = Lazy::new(|| {
    ArcSwap::new(Arc::new(AppConfig::new()))
});

/// 初始化所有配置
pub async fn init_config() -> anyhow::Result<()> {
    // 初始化 Clash 配置
    let clash_config = clash::ClashConfig::load().await?;
    CLASH_CONFIG.store(Arc::new(clash_config));

    // 初始化应用配置
    let app_config = AppConfig::load().await?;
    APP_CONFIG.store(Arc::new(app_config));

    // 初始化配置文件管理
    profiles::Profiles::load().await?;

    Ok(())
}
```

#### 3.2 config/clash.rs - Clash 核心配置

```rust:src-tauri%2Fsrc%2Fconfig%2Fclash.rs
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use arc_swap::ArcSwap;
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde_yaml_ng::Value as YamlValue;

const CLASH_CONFIG_FILE: &str = "config.yaml";

/// Clash 核心配置结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClashConfigData {
    /// 混合端口
    pub mixed_port: Option<u16>,
    /// HTTP 端口
    pub http_port: Option<u16>,
    /// SOCKS5 端口
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
pub struct DnsConfig {
    pub enable: Option<bool>,
    pub nameserver: Option<Vec<String>>,
    pub nameserver_policy: Option<YamlValue>,
    pub fallback: Option<Vec<String>>,
    pub fallback_policy: Option<YamlValue>,
}

/// Clash 配置管理器
#[derive(Debug)]
pub struct ClashConfig {
    /// 配置数据
    data: ArcSwap<ClashConfigData>,
    /// 配置文件路径
    path: PathBuf,
    /// 最后修改时间
    pub last_update: Option<DateTime<Local>>,
}

impl ClashConfig {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            data: ArcSwap::new(Arc::new(ClashConfigData::default())),
            path: Self::get_config_path(),
            last_update: None,
        }
    }

    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(CLASH_CONFIG_FILE);
        path
    }

    /// 从文件加载配置
    pub async fn load() -> Result<Self> {
        let mut config = Self::new();

        if config.path.exists() {
            let content = tokio::fs::read_to_string(&config.path)
                .await
                .context("Failed to read Clash config file")?;

            let data: ClashConfigData = serde_yaml_ng::from_str(&content)
                .context("Failed to parse Clash config file")?;

            config.data.store(Arc::new(data));
            // 这里错误 ，todo fix
            config.last_update = Some(
                std::fs::metadata(&config.path)
                    .map(|meta| {
                        meta.modified()
                            .ok()
                            .map(|time| DateTime::from(time.with_timezone(&Local)))
                    })
                    .unwrap_or(None),
            );
        }

        Ok(config)
    }

    /// 保存配置到文件
    pub async fn save(&mut self) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }

        let data = self.data.load_full();
        let content = serde_yaml_ng::to_string(&data)
            .context("Failed to serialize Clash config")?;

        tokio::fs::write(&self.path, content)
            .await
            .context("Failed to write Clash config file")?;

        self.last_update = Some(Local::now());
        Ok(())
    }

    /// 获取当前配置
    pub fn get(&self) -> Arc<ClashConfigData> {
        self.data.load().clone()
    }

    /// 更新配置
    pub fn patch(&self, new_config: ClashConfigData) {
        self.data.store(Arc::new(new_config));
    }

    /// 获取混合端口
    pub fn get_mixed_port(&self) -> u16 {
        self.data.load().mixed_port.unwrap_or(7890)
    }

    /// 设置混合端口
    pub fn set_mixed_port(&self, port: u16) {
        let mut data = (*self.data.load()).clone();
        data.mixed_port = Some(port);
        self.data.store(Arc::new(data));
    }

    /// 获取外部控制器地址
    pub fn get_external_controller(&self) -> String {
        self.data.load()
            .external_controller
            .clone()
            .unwrap_or_else(|| "127.0.0.1:9090".to_string())
    }

    /// 检查是否允许局域网
    pub fn is_allow_lan(&self) -> bool {
        self.data.load().allow_lan.unwrap_or(false)
    }

    /// 获取日志级别
    pub fn get_log_level(&self) -> String {
        self.data.load()
            .log_level
            .clone()
            .unwrap_or_else(|| "info".to_string())
    }
}

impl Default for ClashConfig {
    fn default() -> Self {
        Self::new()
    }
}
```

#### 3.3 config/verge.rs - 应用配置

```rust:src-tauri%2Fsrc%2Fconfig%2Fverge.rs
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use arc_swap::ArcSwap;
use anyhow::{Context, Result};
use chrono::{DateTime, Local};

const VERGE_CONFIG_FILE: &str = "verge.yaml";

/// Verge 应用配置数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug)]
pub struct AppConfig {
    /// 配置数据
    data: ArcSwap<VergeConfigData>,
    /// 配置文件路径
    path: PathBuf,
    /// 最后修改时间
    pub last_update: Option<DateTime<Local>>,
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
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(VERGE_CONFIG_FILE);
        path
    }

    /// 从文件加载配置
    pub async fn load() -> Result<Self> {
        let mut config = Self::new();

        if config.path.exists() {
            let content = tokio::fs::read_to_string(&config.path)
                .await
                .context("Failed to read Verge config file")?;

            let data: VergeConfigData = serde_yaml_ng::from_str(&content)
                .context("Failed to parse Verge config file")?;

            config.data.store(Arc::new(data));
        }

        config.last_update = Some(Local::now());
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
        let content = serde_yaml_ng::to_string(&data)
            .context("Failed to serialize Verge config")?;

        tokio::fs::write(&self.path, content)
            .await
            .context("Failed to write Verge config file")?;

        self.last_update = Some(Local::now());
        Ok(())
    }

    /// 获取当前配置
    pub fn get(&self) -> Arc<VergeConfigData> {
        self.data.load().clone()
    }

    /// 更新配置
    pub fn patch(&self, new_config: VergeConfigData) {
        self.data.store(Arc::new(new_config));
    }

    /// 获取主题模式
    pub fn get_theme_mode(&self) -> String {
        self.data.load()
            .theme_mode
            .clone()
            .unwrap_or_else(|| "system".to_string())
    }

    /// 设置主题模式
    pub fn set_theme_mode(&self, mode: &str) {
        let mut data = (*self.data.load()).clone();
        data.theme_mode = Some(mode.to_string());
        self.data.store(Arc::new(data));
    }

    /// 是否启用系统代理
    pub fn is_system_proxy_enabled(&self) -> bool {
        self.data.load().enable_system_proxy.unwrap_or(false)
    }

    /// 是否启用 TUN 模式
    pub fn is_tun_mode_enabled(&self) -> bool {
        self.data.load().enable_tun_mode.unwrap_or(false)
    }

    /// 是否启用自动启动
    pub fn is_auto_launch_enabled(&self) -> bool {
        self.data.load().enable_auto_launch.unwrap_or(false)
    }

    /// 是否启用全局热键
    pub fn is_global_hotkey_enabled(&self) -> bool {
        self.data.load().enable_global_hotkey.unwrap_or(true)
    }

    /// 获取 Clash 核心路径
    pub fn get_clash_core_path(&self) -> Option<String> {
        self.data.load().clash_core_path.clone()
    }

    /// 设置 Clash 核心路径
    pub fn set_clash_core_path(&self, path: &str) {
        let mut data = (*self.data.load()).clone();
        data.clash_core_path = Some(path.to_string());
        self.data.store(Arc::new(data));
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 初始化应用配置
pub async fn init_config() -> Result<()> {
    let config = AppConfig::load().await?;
    APP_CONFIG.store(Arc::new(config));
    Ok(())
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
```

#### 3.4 config/profiles.rs - 配置文件管理

```rust:src-tauri%2Fsrc%2Fconfig%2Fprofiles.rs
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{PathBuf, Path},
    sync::Arc,
};
use arc_swap::ArcSwap;
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use nanoid::nanoid;
use serde_yaml_ng::Value as YamlValue;

const PROFILES_FILE: &str = "profiles.yaml";

/// 配置文件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "merge")]
    Merge,
    #[serde(rename = "script")]
    Script,
}

/// 单个配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// 唯一标识符
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 文件路径
    pub file: PathBuf,
    /// 配置类型
    #[serde(rename = "type")]
    pub type_: ProfileType,
    /// 远程 URL（HTTP 类型需要）
    pub url: Option<String>,
    /// 最后更新时间
    pub updated_at: Option<u64>,
    /// 是否启用
    pub enabled: Option<bool>,
    /// 排序权重
    pub order: Option<i32>,
    /// 额外信息
    #[serde(default)]
    pub extra: ProfileExtra,
}

/// 额外信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileExtra {
    /// 上传流量
    pub upload: Option<u64>,
    /// 下载流量
    pub download: Option<u64>,
    /// 总流量
    pub total: Option<u64>,
    /// 更新时间
    pub update_time: Option<u64>,
    /// 订阅信息
    pub subscription_info: Option<SubscriptionInfo>,
}

/// 订阅信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub title: Option<String>,
    pub version: Option<String>,
    pub upload: Option<u64>,
    pub download: Option<u64>,
    pub total: Option<u64>,
    pub expire: Option<u64>,
}

/// 配置文件集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesData {
    /// 配置文件列表
    pub items: Vec<Profile>,
    /// 当前选中的配置 ID
    pub current: Option<String>,
    /// 是否自动更新
    pub auto_update: Option<bool>,
    /// 自动更新间隔（小时）
    pub update_interval: Option<u64>,
}

/// 配置文件管理器
#[derive(Debug)]
pub struct Profiles {
    /// 配置数据
    data: ArcSwap<ProfilesData>,
    /// 配置文件路径
    path: PathBuf,
}

impl Profiles {
    /// 创建新的配置文件管理器
    pub fn new() -> Self {
        Self {
            data: ArcSwap::new(Arc::new(ProfilesData::default())),
            path: Self::get_profiles_path(),
        }
    }

    /// 获取配置文件路径
    fn get_profiles_path() -> PathBuf {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(PROFILES_FILE);
        path
    }

    /// 从文件加载配置
    pub async fn load() -> Result<Self> {
        let mut profiles = Self::new();

        if profiles.path.exists() {
            let content = tokio::fs::read_to_string(&profiles.path)
                .await
                .context("Failed to read profiles file")?;

            let data: ProfilesData = serde_yaml_ng::from_str(&content)
                .context("Failed to parse profiles file")?;

            profiles.data.store(Arc::new(data));
        }

        Ok(profiles)
    }

    /// 保存配置到文件
    pub async fn save(&self) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create profiles directory")?;
        }

        let data = self.data.load();
        let content = serde_yaml_ng::to_string(&data)
            .context("Failed to serialize profiles")?;

        tokio::fs::write(&self.path, content)
            .await
            .context("Failed to write profiles file")?;

        Ok(())
    }

    /// 获取当前配置
    pub fn get(&self) -> Arc<ProfilesData> {
        self.data.load().clone()
    }

    /// 获取所有配置文件
    pub fn get_all(&self) -> Vec<Profile> {
        self.data.load().items.clone()
    }

    /// 获取当前选中的配置
    pub fn get_current(&self) -> Option<Profile> {
        let data = self.data.load();
        data.current
            .as_ref()
            .and_then(|id| data.items.iter().find(|p| p.id == *id))
            .cloned()
    }

    /// 根据 ID 获取配置
    pub fn get_by_id(&self, id: &str) -> Option<Profile> {
        self.data.load()
            .items
            .iter()
            .find(|p| p.id == id)
            .cloned()
    }

    /// 添加新配置
    pub fn add(&self, profile: Profile) {
        let mut data = (*self.data.load()).clone();
        data.items.push(profile);
        self.data.store(Arc::new(data));
    }

    /// 更新配置
    pub fn update(&self, id: &str, new_profile: Profile) -> Result<()> {
        let mut data = (*self.data.load()).clone();

        if let Some(index) = data.items.iter().position(|p| p.id == id) {
            data.items[index] = new_profile;
            self.data.store(Arc::new(data));
            Ok(())
        } else {
            anyhow::bail!("Profile not found: {}", id);
        }
    }

    /// 删除配置
    pub fn delete(&self, id: &str) -> Result<()> {
        let mut data = (*self.data.load()).clone();

        let index = data.items.iter()
            .position(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("Profile not found: {}", id))?;

        // 删除关联的文件
        if let Some(profile) = data.items.get(index) {
            if profile.file.exists() {
                tokio::fs::remove_file(&profile.file)
                    .await
                    .context("Failed to delete profile file")?;
            }
        }

        data.items.remove(index);

        // 如果删除的是当前配置，清除当前配置选择
        if data.current.as_ref() == Some(&id.to_string()) {
            data.current = None;
        }

        self.data.store(Arc::new(data));
        Ok(())
    }

    /// 设置当前配置
    pub fn set_current(&self, id: &str) -> Result<()> {
        let mut data = (*self.data.load()).clone();

        if !data.items.iter().any(|p| p.id == id) {
            anyhow::bail!("Profile not found: {}", id);
        }

        data.current = Some(id.to_string());
        self.data.store(Arc::new(data));
        Ok(())
    }

    /// 创建新配置文件
    pub async fn create_profile(&self, name: &str, content: &str) -> Result<Profile> {
        let id = nanoid!();
        let timestamp = Local::now().timestamp_millis() as u64;

        // 确保目录存在
        let profiles_dir = self.get_profiles_dir();
        tokio::fs::create_dir_all(&profiles_dir)
            .await
            .context("Failed to create profiles directory")?;

        // 生成文件路径
        let file_name = format!("{}_{}.yaml", timestamp, nanoid!(6));
        let file_path = profiles_dir.join(&file_name);

        // 写入文件内容
        tokio::fs::write(&file_path, content)
            .await
            .context("Failed to write profile file")?;

        let profile = Profile {
            id: id.clone(),
            name: name.to_string(),
            file: file_path,
            type_: ProfileType::File,
            url: None,
            updated_at: Some(timestamp),
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };

        self.add(profile.clone());
        Ok(profile)
    }

    /// 获取配置目录
    fn get_profiles_dir(&self) -> PathBuf {
        self.path.parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf()
    }

    /// 更新配置流量信息
    pub fn update_traffic(&self, id: &str, upload: u64, download: u64) -> Result<()> {
        let mut data = (*self.data.load()).clone();

        if let Some(profile) = data.items.iter_mut().find(|p| p.id == id) {
            profile.extra.upload = Some(upload);
            profile.extra.download = Some(download);
            self.data.store(Arc::new(data));
            Ok(())
        } else {
            anyhow::bail!("Profile not found: {}", id);
        }
    }

    /// 重新排序配置
    pub fn reorder(&self, ids: &[String]) -> Result<()> {
        let mut data = (*self.data.load()).clone();

        let mut new_items = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(profile) = data.items.iter().find(|p| p.id == *id) {
                new_items.push(profile.clone());
            } else {
                anyhow::bail!("Profile not found: {}", id);
            }
        }

        data.items = new_items;
        self.data.store(Arc::new(data));
        Ok(())
    }
}

impl Default for Profiles {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置文件管理器扩展trait
#[async_trait::async_trait]
pub trait ProfilesExt {
    async fn enhance(&self, profile_id: &str) -> Result<()>;
}

#[async_trait::async_trait]
impl ProfilesExt for Profiles {
    async fn enhance(&self, _profile_id: &str) -> Result<()> {
        // 实现配置增强逻辑
        // 这里可以根据需要实现规则增强等功能
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profile_creation() {
        let profiles = Profiles::new();
        assert!(profiles.get_all().is_empty());
    }

    #[tokio::test]
    async fn test_add_profile() {
        let profiles = Profiles::new();

        let profile = Profile {
            id: "test-1".to_string(),
            name: "Test Profile".to_string(),
            file: PathBuf::from("/test/profile.yaml"),
            type_: ProfileType::File,
            url: None,
            updated_at: None,
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };

        profiles.add(profile.clone());

        assert_eq!(profiles.get_all().len(), 1);
        assert_eq!(profiles.get_by_id("test-1"), Some(profile));
    }

    #[tokio::test]
    async fn test_delete_profile() {
        let profiles = Profiles::new();

        let profile = Profile {
            id: "test-2".to_string(),
            name: "Test Profile".to_string(),
            file: PathBuf::from("/test/profile.yaml"),
            type_: ProfileType::File,
            url: None,
            updated_at: None,
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };

        profiles.add(profile.clone());
        profiles.delete("test-2").unwrap();

        assert!(profiles.get_all().is_empty());
    }
}
```

#### 3.5 config/prfitem.rs - 代理和规则项

```rust:src-tauri%2Fsrc%2Fconfig%2Fprfitem.rs
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// 代理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
    Socks5T,
    Trojan,
    Vless,
    Vmess,
    Snell,
    Obfs,
}

/// 单个代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyItem {
    /// 代理名称
    pub name: String,
    /// 代理类型
    #[serde(rename = "type")]
    pub type_: ProxyType,
    /// 服务器地址
    pub server: String,
    /// 端口
    pub port: u16,
    /// 密码/密钥
    pub password: Option<String>,
    /// 用户名
    pub username: Option<String>,
    /// 加密方式
    pub cipher: Option<String>,
    /// SNI
    pub sni: Option<String>,
    /// 跳过证书验证
    pub skip_cert_verify: Option<bool>,
    /// UDP 转发
    pub udp: Option<bool>,
    /// 插件选项
    pub plugin: Option<String>,
    /// 插件选项
    pub plugin_opts: Option<String>,
}

/// 规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    IpCidr,
    GeoIP,
    Classical,
    IPSuffix,
}

/// 单个规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleItem {
    /// 规则类型
    #[serde(rename = "type")]
    pub type_: RuleType,
    /// 匹配内容
    pub content: String,
    /// 代理名称
    pub proxy: Option<String>,
    /// 策略组名称
    pub group: Option<String>,
    /// 是否排除
    pub excluded: Option<bool>,
}

/// 规则集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,
    pub type_: String,
    pub format: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
}

/// 代理组类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyGroupType {
    Select,
    UrlTest,
    Fallback,
    LoadBalance,
    Relay,
}

/// 代理组配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: ProxyGroupType,
    /// 包含的代理列表
    pub proxies: Vec<String>,
    /// URL 测试地址
    pub url: Option<String>,
    /// 间隔（秒）
    pub interval: Option<u64>,
    /// 容忍度
    pub tolerance: Option<u64>,
    /// 是否包含原始代理
    pub include_original: Option<bool>,
}
```

#### 3.6 config/encrypt.rs - 加密工具

```rust:src-tauri%2Fsrc%2Fconfig%2Fencrypt.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::Rng;

/// 使用 AES-256-GCM 加密数据
pub fn encrypt(data: &str, key: &str) -> anyhow::Result<String> {
    // 生成随机密钥
    let key_bytes = Key::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(key_bytes);

    // 生成随机 nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密数据
    let ciphertext = cipher.encrypt(nonce, data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // 编码为 Base64
    let mut result = STANDARD.encode(&nonce_bytes);
    result.push(':');
    result.push_str(&STANDARD.encode(&ciphertext));

    Ok(result)
}

/// 使用 AES-256-GCM 解密数据
pub fn decrypt(encrypted: &str, key: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid encrypted format");
    }

    let nonce_bytes = STANDARD.decode(parts[0])
        .map_err(|e| anyhow::anyhow!("Failed to decode nonce: {}", e))?;
    let ciphertext = STANDARD.decode(parts[1])
        .map_err(|e| anyhow::anyhow!("Failed to decode ciphertext: {}", e))?;

    if nonce_bytes.len() != 12 {
        anyhow::bail!("Invalid nonce length");
    }

    let key_bytes = Key::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(key_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in decrypted data: {}", e))
}

/// 生成随机密钥
pub fn generate_key(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = "Hello, World!";
        let key = "test-key-32-characters-long!!";

        let encrypted = encrypt(data, key).unwrap();
        let decrypted = decrypt(&encrypted, key).unwrap();

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_generate_key() {
        let key = generate_key(32);
        assert_eq!(key.len(), 32);

        let key2 = generate_key(32);
        assert_ne!(key, key2);
    }
}
```

### 步骤 4：实现核心管理模块

#### 4.1 core/manager.rs - 进程管理器

```rust:src-tauri%2Fsrc%2Fcore%2Fmanager.rs
use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    process::{Command, Stdio, Child},
    sync::Arc,
    time::Duration,
};
use arc_swap::ArcSwap;
use tokio::process::Command as AsyncCommand;
use tokio::sync::RwLock;
use futures::stream::StreamExt;
use tauri_plugin_shell::process::{CommandEvent, PtyEvent};

/// 运行模式
#[derive(Debug, Clone, PartialEq)]
pub enum RunningMode {
    /// 服务模式
    Service,
    /// 伴随进程模式
    Sidecar,
    /// 未运行
    NotRunning,
}

/// 核心管理器状态
#[derive(Debug)]
pub struct CoreManagerState {
    /// 当前运行模式
    running_mode: ArcSwap<RunningMode>,
    /// 子进程句柄
    child: ArcSwapOption<Child>,
    /// 最后更新时间
    last_update: ArcSwapOption<std::time::Instant>,
}

impl Default for CoreManagerState {
    fn default() -> Self {
        Self {
            running_mode: ArcSwap::new(Arc::new(RunningMode::NotRunning)),
            child: ArcSwapOption::new(None),
            last_update: ArcSwapOption::new(None),
        }
    }
}

/// 核心管理器
#[derive(Debug)]
pub struct CoreManager {
    /// 内部状态
    state: Arc<CoreManagerState>,
    /// 核心可执行文件路径
    core_path: PathBuf,
    /// 核心配置路径
    config_path: PathBuf,
}

impl CoreManager {
    /// 创建新的核心管理器
    pub fn new(core_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            state: Arc::new(CoreManagerState::default()),
            core_path,
            config_path,
        }
    }

    /// 启动 Clash 核心
    pub async fn start(&self) -> Result<()> {
        if self.is_running() {
            anyhow::bail!("Clash core is already running");
        }

        // 检查核心文件是否存在
        if !self.core_path.exists() {
            anyhow::bail!("Clash core not found: {}", self.core_path.display());
        }

        if !self.config_path.exists() {
            anyhow::bail!("Clash config not found: {}", self.config_path.display());
        }

        // 启动进程
        let mut child = AsyncCommand::new(&self.core_path)
            .args(&["-d", self.config_path.parent()
                .unwrap_or(&PathBuf::from("."))
                .to_str()
                .unwrap_or(".")])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start Clash core")?;

        // 等待进程启动
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 检查进程是否正常运行
        if let Some(status) = child.try_wait()? {
            if status.code().is_some() {
                anyhow::bail!("Clash core exited immediately with code: {:?}", status.code());
            }
        }

        // 更新状态
        self.state.running_mode.store(Arc::new(RunningMode::Sidecar));
        self.state.child.store(Some(Arc::new(child)));

        Ok(())
    }

    /// 停止 Clash 核心
    pub async fn stop(&self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        let child = self.state.child.swap(None);
        if let Some(child_arc) = child {
            let mut child = Arc::try_unwrap(child_arc)
                .map_err(|_| anyhow::anyhow!("Failed to unwrap child process"))?;

            child.kill().await.context("Failed to kill Clash core")?;
            child.wait().await.context("Failed to wait for Clash core")?;
        }

        self.state.running_mode.store(Arc::new(RunningMode::NotRunning));
        Ok(())
    }

    /// 重启 Clash 核心
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(*self.state.running_mode.load(),
            RunningMode::Service | RunningMode::Sidecar)
    }

    /// 获取当前运行模式
    pub fn get_running_mode(&self) -> RunningMode {
        *self.state.running_mode.load()
    }

    /// 获取进程输出
    pub async fn get_output(&self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let child = self.state.child.load();
        if let Some(child_arc) = child {
            let child = Arc::clone(child_arc);
            // 获取stdout和stderr
            // 这里需要根据实际实现调整
            Ok(None)
        } else {
            Ok(None)
        }
    }
}
```

### 步骤 5：实现 Tauri 命令模块

#### 5.1 cmd/mod.rs - 命令入口

```rust:src-tauri%2Fsrc%2Fcmd%2Fmod.rs
use anyhow::Result;
use smartstring::alias::String as SmartString;

pub type CmdResult<T = ()> = Result<T, SmartString>;

pub mod app;
pub mod clash;
pub mod profile;
pub mod proxy;
pub mod system;

pub use app::*;
pub use clash::*;
pub use profile::*;
pub use proxy::*;
pub use system::*;

/// 字符串错误转换 trait
pub trait StringifyErr<T> {
    fn stringify_err(self) -> CmdResult<T>;
    fn stringify_err_log<F>(self, log_fn: F) -> CmdResult<T>
    where
        F: Fn(&str);
}

impl<T, E: std::fmt::Display> StringifyErr<T> for Result<T, E> {
    fn stringify_err(self) -> CmdResult<T> {
        self.map_err(|e| SmartString::from(e.to_string()))
    }

    fn stringify_err_log<F>(self, log_fn: F) -> CmdResult<T>
    where
        F: Fn(&str),
    {
        self.map_err(|e| {
            let msg = SmartString::from(e.to_string());
            log_fn(&msg);
            msg
        })
    }
}
```

#### 5.2 cmd/app.rs - 应用命令

```rust:src-tauri%2Fsrc%2Fcmd%2Fapp.rs
use crate::config::{APP_CONFIG, CLASH_CONFIG};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewWindow};

/// 获取应用配置
#[tauri::command]
pub async fn get_verge_config() -> Result<Arc<verge_config::VergeConfigData>, String> {
    Ok(APP_CONFIG.load().get().clone())
}

/// 更新应用配置
#[tauri::command]
pub async fn patch_verge_config(
    config: verge_config::VergeConfigData,
) -> Result<(), String> {
    APP_CONFIG.load().patch(config);
    APP_CONFIG.load().save().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 重启应用
#[tauri::command]
pub async fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
    Ok(())
}

/// 退出应用
#[tauri::command]
pub async fn exit_app(app: AppHandle) -> Result<(), String> {
    // 清理资源
    // 停止 Clash 核心
    // 保存配置
    std::process::exit(0);
}

/// 获取应用目录
#[tauri::command]
pub fn get_app_dir() -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-new-app");
    Ok(path.to_str()
        .ok_or("Invalid path")?
        .to_string())
}

/// 获取日志目录
#[tauri::command]
pub fn get_logs_dir() -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("my-new-app")
        .join("logs");
    Ok(path.to_str()
        .ok_or("Invalid path")?
        .to_string())
}

/// 打开目录
#[tauri::command]
pub async fn open_dir(path: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use winapi::um::winbase::CREATE_NO_WINDOW;

        Command::new("explorer.exe")
            .args(&[path.to_str().unwrap_or("")])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
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
            os_name: sysinfo::System::name().unwrap_or_else(|| "Unknown".into()),
            os_version: sysinfo::System::os_version().unwrap_or_else(|| "Unknown".into()),
            kernel_version: sysinfo::System::kernel_version().unwrap_or_else(|| "Unknown".into()),
            cpu_count: sysinfo::System::cpu_count(),
            total_memory: sysinfo::System::total_memory(),
            used_memory: sysinfo::System::used_memory(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_count: u32,
    pub total_memory: u64,
    pub used_memory: u64,
}
```

### 步骤 6：实现日志系统

```rust:src-tauri%2Fsrc%2Futils%2Flogging.rs
use flexi_logger::{DeferredNow, LogSpec, Logger};
use std::path::PathBuf;

pub fn init_logger() -> Result<(), flexi_logger::FlexiLoggerError> {
    let log_path = get_log_dir();

    let log_spec = LogSpec::default()
        .default_filter_level(log::LevelFilter::Info);

    Logger::with(log_spec)
        .log_to_file()
        .directory(&log_path)
        .append()
        .format(|w, now: &DeferredNow, record, msg| {
            write!(
                w,
                "[{}] [{}] {} - {}:{}:{}",
                now.now().format("%Y-%m-%d %H:%M:%S%.6f"),
                record.level(),
                record.target(),
                record.file().unwrap_or("?"),
                record.line().unwrap_or(0),
                msg
            )
        })
        .start()
}

fn get_log_dir() -> PathBuf {
    let mut path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("my-new-app");
    path.push("logs");
    path
}
```

## 测试指南

### 单元测试

```rust
// tests/config_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clash_config_load_save() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let mut config = ClashConfig::new();
        config.set_mixed_port(8080);

        // 保存配置
        config.save().await.unwrap();

        // 重新加载
        let loaded = ClashConfig::load().await.unwrap();
        assert_eq!(loaded.get_mixed_port(), 8080);
    }

    #[tokio::test]
    async fn test_profile_operations() {
        let profiles = Profiles::new();

        let profile = Profile {
            id: "test".to_string(),
            name: "Test".to_string(),
            file: PathBuf::from("/test.yaml"),
            type_: ProfileType::File,
            url: None,
            updated_at: None,
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };

        profiles.add(profile.clone());
        assert_eq!(profiles.get_all().len(), 1);

        profiles.delete("test").unwrap();
        assert!(profiles.get_all().is_empty());
    }
}
```

## 性能优化

### 1. 配置加载优化

- 使用 ArcSwap 避免不必要的锁竞争
- 延迟加载非必要配置
- 批量保存减少 I/O 操作

### 2. 内存使用优化

- 使用 Arc 共享配置数据
- 避免不必要的克隆
- 使用智能指针管理生命周期

### 3. 异步处理

- 使用 tokio 进行异步 I/O
- 实现后台任务处理
- 使用 channel 进行进程间通信

## 常见问题

### Q1: 配置加载失败怎么办？

A1: 检查配置文件路径和格式，使用默认配置作为回退。

### Q2: 如何处理配置文件冲突？

A2: 实现文件锁机制，避免并发写入。

### Q3: 内存占用过高如何优化？

A3: 使用 ArcSwap 减少内存复制，及时释放不需要的资源。

## 下一步计划

完成当前阶段后，我们将继续实现：

1. **阶段 2**: Clash 核心管理（进程控制、生命周期）
2. **阶段 3**: 代理和规则管理
3. **阶段 4**: 系统集成（系统代理、TUN）
4. **阶段 5**: 媒体解锁功能
5. **阶段 6**: 备份和恢复

## 质量检查清单

- [ ] 配置正确保存和加载
- [ ] 错误处理完善
- [ ] 单元测试覆盖核心逻辑
- [ ] 性能满足要求
- [ ] 文档完整
- [ ] 代码风格一致
