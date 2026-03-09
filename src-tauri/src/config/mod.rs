use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use std::sync::Arc;

mod clash;
mod verge;
mod profiles;

pub use clash::ClashConfig;
pub use verge::VERGE_CONFIG;
pub use verge::load_config;

pub const DEFAULT_PAC: &str = r#"function FindProxyForURL(url, host) {
  return "PROXY 127.0.0.1:%mixed-port%; SOCKS5 127.0.0.1:%mixed-port%; DIRECT;";
}
"#;

pub static CLASH_DATA: Lazy<ArcSwap<clash::ClashConfigData>> =
    Lazy::new(|| ArcSwap::new(Arc::new(clash::ClashConfigData::default())));

pub async fn init_config() -> anyhow::Result<()> {
    let clash_cfg = ClashConfig::load().await?;
    CLASH_DATA.store(clash_cfg.get());

    let config_data = load_config().await?;
    VERGE_CONFIG.store(Arc::new(config_data));

    Ok(())
}
