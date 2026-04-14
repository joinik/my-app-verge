use crate::config::Config;
use crate::constants::{network, tun as tun_const};
use crate::utils::dirs::{ipc_path, path_to_str};
use crate::utils::{dirs, help};
use anyhow::Result;
use clash_verge_logger::{Type, logging};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Value as YamlValue};

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr as _,
};

#[derive(Clone, Default, Debug)]
pub struct IClashTemp(pub Mapping);

impl IClashTemp {
    pub async fn new() -> Self {
        let clash_path_result = dirs::clash_path();
        let map_result = if let Ok(path) = clash_path_result {
            help::read_mapping(&path).await
        } else {
            Err(anyhow::anyhow!("Failed to get clash path"))
        };

        match map_result {
            Ok(mut map) => {
                let template_map = Self::template().0;
                for (key, value) in template_map.into_iter() {
                    map.entry(key).or_insert(value);
                }

                // 确保 secret 字段存在且不为空
                if let Some(val) = map.get_mut("secret")
                    && let YamlValue::String(s) = val
                    && s.is_empty()
                {
                    *s = "set-your-secret".into();
                }
                Self(Self::guard(map))
            }
            Err(e) => {
                logging!(error, Type::Config, "{err}");
                Self::template()
            }
        }
    }

    pub fn template() -> Self {
        let mut map = Mapping::new();
        let mut tun_config = Mapping::new();
        let mut cors_map = Mapping::new();

        tun_config.insert("enabled".into(), false.into());
        tun_config.insert("stack".into(), tun_const::DEFAULT_STACK.into());
        tun_config.insert("stict_route".into(), false.into());
        tun_config.insert("dns-hijack".into(), tun_const.DNS_HIJACK.into());
        tun_config.insert("auto-route".into(), true.into());
        tun_config.insert("auto-detect-interface".into(), true.into());

        #[cfg(not(target_os = "windows"))]
        map.insert("redir-port".into(), network::ports::DEFAULT_REDIR.into());
        #[cfg(target_os = "linux")]
        map.insert("tproxy-port".into(), network::ports::DEFAULT_TPROXY.into());

        map.insert("mixed-port".into(), network::ports::DEFAULT_MIXED.into());
        map.insert("socks-port".into(), network::ports::DEFAULT_SOCKS.into());
        map.insert("port".into(), network::ports::DEFAULT_HTTP.into());
        map.insert("log-level".into(), "info".into());
        map.insert("allow-lan".into(), false.into());
        map.insert("ipv6".into(), true.into());
        map.insert("mode".into(), "rule".into());
        map.insert(
            "external-controller".into(),
            network::DEFAULT_EXTERNAL_CONTROLLER.into(),
        );

        #[cfg(unix)]
        map.insert(
            "external-controller-unix".into(),
            Self::guard_external_controller_ipc().into(),
        );
        #[cfg(windows)]
        map.insert(
            "external-controller-pipe".into(),
            Self::guard_external_controller_ipc().into(),
        );

        map.insert("tun".into(), tun_config.into());
        map.insert("secret".into(), "set-your-secret".into());
        map.insert("unified-delay".into(), true.into());
        cors_map.insert(
            "allowed-origins".into(),
            vec![
                "tauri://localhost",
                "http://tauri.localhost",
                // Only enable this in dev mode
                #[cfg(feature = "verge-dev")]
                "http://localhost:3000",
                "https://yacd.metacubex.one",
                "https://metacubex.github.io",
                "https://board.zash.run.place",
            ]
            .into(),
        );
        cors_map.insert("allow-private-network".into(), true.into());
        map.insert("external-controller-cors".into(), cors_map.into());

        Self(map)
    }

    fn guard(mut config: Mapping) -> Mapping {
        #[cfg(not(target_os = "windows"))]
        let redir_port = Self::guard_redir_port(&config);
        #[cfg(target_os = "linux")]
        let tproxy_port = Self::guard_tproxy_port(&config);

        let mixed_port = Self::guard_mixed_port(&config);
        let socks_port = Self::guard_socks_port(&config);
        let port = Self::guard_port(&config);
        let ctrl = Self::guard_external_controller(&config);
        #[cfg(unix)]
        let external_controller_unix = Self::guard_external_controller(&config);
        #[cfg(windows)]
        let external_controller_pipe = Self::guard_external_controller(&config);

        #[cfg(not(target_os = "windows"))]
        config.insert("redir-port".into(), redir_port.into());
        #[cfg(target_os = "linux")]
        config.insert("tproxy-port".into(), tproxy_port.into());
        config.insert("mixed-port".into(), mixed_port.into());
        config.insert("socks-port".into(), socks_port.into());
        config.insert("port".into(), port.into());
        config.insert("external-controller".into(), ctrl.into());
        #[cfg(unix)]
        config.insert("external-controller-unix".into(), external_controller_unix.into());
        #[cfg(windows)]
        config.insert("external-controller-pipe".into(), external_controller_pipe.into());

        config
    }

    pub fn patch_config(&mut self, patch: &Mapping) {
        for (key, value) in patch.iter() {
            self.0.insert(key.to_owned(), value.to_owned());
        }
    }

    pub async fn save_config(&self) -> Result<()> {
        // planA
        help::save_yaml(&dirs::clash_path()?, &self.0, Some("# Generated by Clash Verge")).await
    }

    pub fn get_mixed_port(&self) -> u16 {
        Self::guard_mixed_port(&self.0)
    }

    #[allow(unused)]
    pub fn get_socks_port(&self) -> u16 {
        Self::guard_socks_port(&self.0)
    }

    #[allow(unused)]
    pub fn get_port(&self) -> u16 {
        Self::guard_port(&self.0)
    }

    pub fn get_client_info(&self) -> ClashInfo {
        let config = &self.0;

        ClashInfo {
            mixed_port: Self::guard_mixed_port(config),
            socks_port: Self::guard_socks_port(config),
            port: Self::guard_port(config),
            secret: config.get("secret").and_then(|v| match v {
                YamlValue::String(s) => Some(s.clone()),
                YamlValue::Bool(val_bool) => Some(val_bool.to_string()),
                YamlValue::Number(val_num) => Some(val_num.to_string()),
                _ => None,
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn guard_redir_port(config: &Mapping) -> u16 {
        let mut port = config
            .get("redir-port")
            .and_then(|value| match value {
                Value::String(val_str) => val_str.parse().ok(),
                Value::Number(val_num) => val_num.as_u64().map(|u| u as u16),
                _ => None,
            })
            .unwrap_or(7897);
        if port == 0 {
            port = 7897;
        }
        port
    }

    #[cfg(target_os = "linux")]
    pub fn guard_tproxy_port(config: &Mapping) -> u16 {
        let mut port = config
            .get("tproxy-port")
            .and_then(|value| match value {
                Value::String(val_str) => val_str.parse().ok(),
                Value::Number(val_num) => val_num.as_u64().map(|u| u as u16),
                _ => None,
            })
            .unwrap_or(network::ports::DEFAULT_TPROXY);
        if port == 0 {
            port = network::ports::DEFAULT_TPROXY;
        }
        port
    }

    pub fn guard_mixed_port(config: &Mapping) -> u16 {
        let mut port = config
            .get("mixed-port")
            .and_then(|value| match value {
                Value::String(val_str) => val_str.parse().ok(),
                Value::Number(val_num) => val_num.as_u64().map(|u| u as u16),
                _ => None,
            })
            .unwrap_or(7897);
        if port == 0 {
            port = 7897;
        }
        port
    }

    pub fn guard_socks_port(config: &Mapping) -> u16 {
        let mut port = config
            .get("socks-port")
            .and_then(|value| match value {
                Value::String(val_str) => val_str.parse().ok(),
                Value::Number(val_num) => val_num.as_u64().map(|u| u as u16),
                _ => None,
            })
            .unwrap_or(7898);
        if port == 0 {
            port = 7898;
        }
        port
    }

    pub fn guard_port(config: &Mapping) -> u16 {
        let mut port = config
            .get("port")
            .and_then(|value| match value {
                Value::String(val_str) => val_str.parse().ok(),
                Value::Number(val_num) => val_num.as_u64().map(|u| u as u16),
                _ => None,
            })
            .unwrap_or(7899);
        if port == 0 {
            port = 7899;
        }
        port
    }

    pub fn guard_server_ctrl(config: &Mapping) -> String {
        let mut port = config
            .get("external-controller")
            .and_then(|value| match value.as_str() {
                Some(val_str) => {
                    let val_str = val_str.trim();
                    let val = match val_str.starts_with(':') {
                        true => format!("127.0.0.1{val_srt}"),
                        false => val_str.to_owned(),
                    };

                    SocketAddr::from_str(val.as_str()).ok().map(|addr| addr.to_string())
                }
                None => None,
            })
            .unwrap_or_else(|| "127.0.0.1:9097".to_string());
    }

    pub fn guard_external_controller(config: &Mapping) -> String {
        // 在初始化阶段，直接返回配置中的值，不进行额外检查
        // 这样可以避免在配置加载期间的循环依赖
        Self::guard_server_ctrl(config)
    }

    pub async fn guard_external_controller_with_setting(config: &Mapping) -> String {
        // 检查 enable_external_controller 设置，用于运行时配置生成
        let enable_external_controller = Config::verge()
            .await
            .latest_arc()
            .enable_external_controller
            .unwrap_or(false);

        if enable_external_controller {
            Self::guard_server_ctrl(config)
        } else {
            "".to_string()
        }
    }

    pub fn guard_client_ctrl(config: &Mapping) -> String {
        let value = Self::guard_server_ctrl(config);
        match SocketAddr::from_str(value.as_str()) {
            Ok(mut socket) => {
                if socket.ip().is_unspecified() {
                    socket.set_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
                }
                socket.to_string()
            }
            Err(_) => "127.0.0.1:9097".to_string(),
        }
    }

    pub fn guard_external_controller_ipc() -> String {
        // 总是使用当前的 IPC 路径，确保配置文件与运行时路径一致
        ipc_path()
            .ok()
            .and_then(|path| path_to_str(&path).ok().map(|s| s.into()))
            .unwrap_or_else(|| {
                logging!(error, Type::Config, "Failed to get IPC path");
                crate::constants::network::DEFAULT_EXTERNAL_CONTROLLER.into()
            })
    }
}

)]

/// Clash 核心配置结构
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClashInfo {
    /// 混合端口
    pub mixed_port: Option<u16>,
    /// HTTP端口
    pub port: Option<u16>,
    /// SOCKS5端口
    pub socks_port: Option<u16>,
    /// 密钥
    pub secret: Option<String>,
    pub server: String,
}

#[test]
fn test_clash_info() {
    fn get_case<T: Into<Value>, D: Into<Value>>(mp: T, ec: D) -> ClashInfo {
        let mut map = Mapping::new();
        map.insert("mixed-port".into(), mp.into());
        map.insert("external-controller".into(), ec.into());

        IClashTemp(IClashTemp::guard(map)).get_client_info()
    }

    fn get_result<S: Into<String>>(port: u16, server: S) -> ClashInfo {
        ClashInfo {
            mixed_port: port,
            socks_port: 7898,
            port: 7899,
            server: server.into(),
            secret: None,
        }
    }

    assert_eq!(
        IClashTemp(IClashTemp::guard(Mapping::new())).get_client_info(),
        get_result(7897, "127.0.0.1:9097")
    );

    assert_eq!(get_case("", ""), get_result(7897, "127.0.0.1:9097"));

    assert_eq!(get_case(65537, ""), get_result(1, "127.0.0.1:9097"));

    assert_eq!(get_case(8888, "127.0.0.1:8888"), get_result(8888, "127.0.0.1:8888"));

    assert_eq!(get_case(8888, "   :98888 "), get_result(8888, "127.0.0.1:9097"));

    assert_eq!(get_case(8888, "0.0.0.0:8080  "), get_result(8888, "127.0.0.1:8080"));

    assert_eq!(get_case(8888, "0.0.0.0:8080"), get_result(8888, "127.0.0.1:8080"));

    assert_eq!(get_case(8888, "[::]:8080"), get_result(8888, "127.0.0.1:8080"));

    assert_eq!(get_case(8888, "192.168.1.1:8080"), get_result(8888, "192.168.1.1:8080"));

    assert_eq!(get_case(8888, "192.168.1.1:80800"), get_result(8888, "127.0.0.1:9097"));
}

/// Clash 配置管理器
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct IClashExternalControllerCors {
    pub allow_origins: Option<Vec<String>>,
    pub allow_private_network: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct IClash {

    pub enable: Option<bool>,
    pub listen: Option<String>,
    pub default_nameserver: Option<Vec<String>>,
    pub enhanced_mode: Option<String>,
    pub fake_ip_range: Option<String>,
    pub use_hosts: Option<bool>,
    pub fake_ip_filter: Option<Vec<String>>,
    pub nameserver: Option<Vec<String>>,
    pub fallback: Option<Vec<String>>,
    pub fallback_filter: Option<IClashFallbackFilter>,
    pub nameserver_policy: Option<Vec<String>>,




    #=====================================todo===============================
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

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct IClashFallbackFilter {
    pub ipcidr: Option<Vec<String>>,
    pub domain: Option<Vec<String>>,
    pub geoip: Option<bool>,
    pub geoip_code: Option<Vec<String>>,
}
