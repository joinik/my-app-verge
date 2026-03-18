use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub mod clash;
pub mod config;
pub mod encrypt;
pub mod prfitem;
pub mod profiles;
pub mod verge;

pub use self::{
    clash::ClashConfig,
    config::VergeConfig,
    encrypt::{decrypt, encrypt},
    prfitem::{ProxyItem, RuleItem},
    profiles::{Profiles, ProfilesExt},
    verge::{AppConfig, VergeConfigData},
};

pub const DEFAULT_PAC: &str = r#"function FindProxyForURL(url, host) {
  return "PROXY 127.0.0.1:%mixed-port%; SOCKS5 127.0.0.1:%mixed-port%; DIRECT;";
}
"#;

/// 全局 Clash 配置实例
pub static CLASH_CONFIG: Lazy<ArcSwap<clash::ClashConfig>> =
    Lazy::new(|| ArcSwap::new(Arc::new(clash::ClashConfig::new())));

/// 全局 Verge 应用配置实例
pub static APP_CONFIG: Lazy<ArcSwap<AppConfig>> = Lazy::new(|| ArcSwap::new(Arc::new(AppConfig::new())));

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
