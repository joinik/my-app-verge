use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tauri::utils::config::AppConfig;

mod clash;
mod config;
mod encrypt;
mod prfitem;
pub mod profiles;
mod verge;

pub use self::{
    clash::{CLASH_CONFIG, ClashConfig},
    config::VergeConfig,
    encrypt::{decrypt, encrypt},
    prfitem::{ProxyItem, RuleItem},
    profiles::{Profiles, ProfilesExt},
    verge::{APP_CONFIG, AppConfig, init_config},
};

pub const DEFAULT_PAC: &str = r#"function FindProxyForURL(url, host) {
  return "PROXY 127.0.0.1:%mixed-port%; SOCKS5 127.0.0.1:%mixed-port%; DIRECT;";
}
"#;

/// 全局Clash 配置实例
pub static CLASH_CONFIG: Lazy<ArcSwap<clash::ClashConfig>> =
    Lazy::new(|| ArcSwap::new(Arc::new(clash::ClashConfig::new())));

/// 全局 Verge 应用配置实例
pub static APP_CONFIG: Lazy<ArcSwap<AppConfig>> = Lazy::new(|| ArcSwap::new(Arc::new(AppConfig::new())));

/// 初始化配置
pub async fn init_config() -> anyhow::Result<()> {
    // 初始化 Clash配置
    let clash_config = clash::ClashConfig::load().await?;
    CLASH_CONFIG.store(Arc::new(clash_config));

    // 初始化应用配置
    let app_config = AppConfig::load().await?;
    APP_CONFIG.store(Arc::new(app_config));

    // 初始化配置文件管理
    profiles::Profiles::load().await?;
    Ok(())
}
