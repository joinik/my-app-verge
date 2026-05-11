use anyhow::{Context, Result};
use log::LevelFilter;
use my_app_logging::logging;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

use crate::config::{DEFAULT_PAC, config::Config};

const VERGE_CONFIG_FILE: &str = "verge.yaml";

/// Verge 应用配置数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IVerge {
    /// 主题模式：light, dark, system
    pub theme_mode: Option<String>,
    /// i18n language
    pub language: Option<String>,
    /// tray click event
    pub tray_event: Option<String>,
    /// copy env type
    pub env_type: Option<String>,

    /// start page
    pub start_page: Option<String>,
    /// startup script path
    pub startup_script: Option<String>,

    /// enable traffic graph default is true
    pub traffic_graph: Option<bool>,

    /// show memory info (only for Clash Meta)
    pub enable_memory_usage: Option<bool>,

    /// enable group icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_group_icon: Option<bool>,

    /// common tray icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_tray_icon: Option<bool>,

    /// tray icon
    #[cfg(target_os = "macos")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tray_icon: Option<String>,

    /// menu icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_icon: Option<String>,

    /// menu order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_order: Option<Vec<String>>,

    /// toast /notice positon on screen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toast_position: Option<String>,

    /// collapse navigation bar
    pub collapse_navbar: Option<bool>,

    /// sysproxy tray icon
    pub sysproxy_tray_icon: Option<bool>,

    /// tun tray icon
    pub tun_tray_icon: Option<bool>,

    /// clash tun mode
    /// 是否启用 TUN 模式
    pub enable_tun_mode: Option<bool>,

    /// 是否启用自动启动
    pub enable_auto_launch: Option<bool>,
    /// 是否启用系统代理
    pub enable_system_proxy: Option<bool>,

    /// not show the window on launch
    pub enable_silent_start: Option<bool>,

    /// enable proxy guard
    pub enable_proxy_guard: Option<bool>,

    /// enable dns settings - this controls whether dns_config.yaml is applied
    pub enable_dns_settings: Option<bool>,

    /// always use default bypass
    pub use_default_bypass: Option<bool>,

    /// set system proxy bypass
    /// 系统代理绕过列表
    pub system_proxy_bypass: Option<String>,

    /// proxy guard duration
    pub proxy_guard_duration: Option<u64>,

    /// use pac mode
    pub proxy_auto_config: Option<bool>,

    /// pac script content
    pub pac_file_content: Option<String>,

    /// proxy host address
    pub proxy_host: Option<String>,

    /// theme setting
    pub theme_setting: Option<IVergeTheme>,

    /// web ui list
    pub web_ui_list: Option<Vec<String>>,

    /// clash core path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clash_core: Option<String>,

    /// hotkey map
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotkeys: Option<Vec<String>>,

    /// enable global hotkey
    pub enable_global_hotkey: Option<bool>,

    /// home cards settings
    pub home_cards: Option<serde_json::Value>,

    /// auto close connection
    pub auto_close_connection: Option<bool>,

    /// 是否启用自动检查更新
    pub enable_auto_check_update: Option<bool>,

    /// 默认 latency test
    pub default_latency_test: Option<String>,

    /// 默认 latency timeout
    pub default_latency_timeout: Option<i16>,

    /// 是否自动检测当前节点延迟
    pub enable_auto_delay_detection: Option<bool>,

    /// 是否启用内部的脚本支持 默认为真
    pub enable_builtin_enhanced: Option<bool>,

    /// proxy layout column
    pub proxy_layout_column: Option<u8>,

    /// 测试站列表
    pub test_list: Option<Vec<IVergeTestItem>>,

    /// 自动日志清理间隔（天）
    /// 0: 不清理；1：1天；2：7天；3：30天；4：90天
    pub auto_log_clean: Option<i32>,

    /// Enable scheduled automatic backups
    pub enable_auto_backup_scheduled: Option<bool>,

    /// Automatic backup interval in hours
    pub auto_backup_interval_hours: Option<u64>,

    /// Create backups automatically when critical configs change
    pub auto_backup_on_change: Option<bool>,

    /// verge 的各种 port 用于覆盖 clash 的各种 port
    #[cfg(not(target_os = "windows"))]
    pub verge_redir_port: Option<u16>,

    #[cfg(not(target_os = "windows"))]
    pub verge_redir_enabled: Option<bool>,

    #[cfg(target_os = "linux")]
    pub verge_tproxy_port: Option<u16>,

    #[cfg(target_os = "linux")]
    pub verge_tproxy_enabled: Option<bool>,

    pub verge_mixed_port: Option<u16>,

    pub verge_socks_port: Option<u16>,

    pub verge_socks_enabled: Option<bool>,

    pub verge_port: Option<u16>,

    pub verge_http_enabled: Option<bool>,

    /// WebDAV 配置 (加密存储)
    #[serde(
        serialize_with = "serialize_encrypted",
        deserialize_with = "deserialize_encrypted",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub webdav_url: Option<String>,

    /// WebDAV 用户名 (加密存储)
    #[serde(
        serialize_with = "serialize_encrypted",
        deserialize_with = "deserialize_encrypted",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub webdav_username: Option<String>,

    /// WebDAV 密码 (加密存储)
    #[serde(
        serialize_with = "serialize_encrypted",
        deserialize_with = "deserialize_encrypted",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub webdav_password: Option<String>,

    #[serde(skip)]
    pub enable_tray_speed: Option<bool>,

    // pub enable_tray_icon: Option<bool>,
    /// show proxy groups directly on tray root menu
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tray_proxy_groups_display_mode: Option<String>,
    /// show outbound modes directly on tray root menu
    pub tray_inline_outbound_modes: Option<bool>,

    /// 自动进入轻量模式
    pub enable_auto_light_weight_mode: Option<bool>,

    /// 自动进入轻量模式的延迟（分钟）
    pub auto_light_weight_minutes: Option<u64>,

    /// 启用代理页面自动滚动
    pub enable_hover_jump_navigator: Option<bool>,

    /// 代理页面自动滚动延迟（毫秒）
    pub hover_jump_navigator_delay: Option<u64>,

    /// 启用外部控制器
    pub enable_external_controller: Option<bool>,

    /// 日志级别
    /// app log level
    /// silent | error | warn | info | debug | trace
    pub app_log_level: Option<String>,
    /// app log max size in KB
    pub app_log_max_size: Option<u64>,
    /// app log max count
    pub app_log_max_count: Option<usize>,
    enable_auto_backup_schedule: Option<bool>,
    notice_position: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IVergeTestItem {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IVergeTheme {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub primary_text: Option<String>,
    pub secondary_text: Option<String>,

    pub info_color: Option<String>,
    pub error_color: Option<String>,
    pub warning_color: Option<String>,
    pub success_color: Option<String>,

    pub font_family: Option<String>,
    pub css_injection: Option<String>,
}

impl IVerge {
    /// Valid clash cores
    pub const VALID_CLASH_CORES: &'static [&'static str] = &["verge-mihomo", "verge-mihomo-alpha"];

    pub async fn validate_and_fix_config() -> Result<()> {
        let config_path = dirs::verge_path()?;
        let mut config = match help::read_yaml::<Self>(&config_path).await {
            Ok(config) => config,
            Err(_) => Self::template(),
        };

        let mut needs_fix = false;
        if let Some(ref core) = config.clash_core {
            if !Self::VALID_CLASH_CORES.contains(&core.as_str()) || core.is_empty() {
                logging!(
                    warn,
                    Type::Config,
                    "启动时发现无效的clash_core配置:'{}'，将自动修正为 'verge-mihomo'",
                );
                config.clash_core = Some(
                    "verge-mihomo
            "
                    .into(),
                );
                needs_fix = true;
            }
        } else {
            logging!(info, Type::Config, "clash_core未设置，将使用默认值 'verge-mihomo'");
            config.clash_core = Some("verge-mihomo".into());
            needs_fix = true;
        }

        // Save the configuration after fixing
        if needs_fix {
            logging!(info, Type::Config, "保存修正后的配置文件");
            help::save_yaml(&config_path, &config, Some("# Clash Verge Config")).await?;
            logging!(info, Type::Config, "配置文件修正完成，需要重新加载配置");

            Self::reload_config_after_fix(config).await?;
        } else {
            logging!(info, Type::Config, "配置文件验证通过:{:?}", config.clash_core);
        }

        Ok(())
    }

    async fn reload_config_after_fix(updated_config: Self) -> Result<()> {
        logging!(
            info,
            Type::Config,
            "内存配置已强制更新，新的clash_core：{:?}",
            updated_config.clash_core
        );

        let config_draft = Config::verge().await;
        config_draft.edit_draft(|d| {
            *d = updated_config;
        });
        config_draft.apply();
        Ok(())
    }

    pub fn get_valid_clash_cores(&self) -> String {
        self.clash_core.clone().unwrap_or_else(|| "verge-mihomo".to_string())
    }

    /// 创建新的配置管理器
    pub async fn new() -> Self {
        match dirs::verge_path() {
            Ok(path) => match help::read_yaml::<Self>(&path).await {
                Ok(mut config) => {
                    // compatibility
                    if let Some(start_page) = config.start_page.clone()
                        && start_page == "/home"
                    {
                        config.start_page = Some(String::from("/"))
                    }
                    config
                }
                Err(err) => {
                    logging!(error, Type::Config, "{err}");
                    Self::template()
                }
            },
            Err(err) => {
                logging!(error, Type::Config, "{err}");
                Self::template()
            }
        }
    }

    pub fn template() -> Self {
        Self {
            app_log_max_size: Some(128),
            app_log_max_count: Some(8),
            clash_core: Some("verge-mihomo".to_string()),
            language: Some(my_app_i18n::system_language().into()),
            theme_mode: Some("system".to_string()),
            #[cfg(not(target_os = "windows"))]
            env_type: Some("bash".to_string()),
            #[cfg(target_os = "windows")]
            env_type: Some("powershell".to_string()),
            start_page: Some("/".into()),
            traffic_graph: Some(true),
            enable_memory_usage: Some(true),
            enable_group_icon: Some(true),
            #[cfg(target_os = "macos")]
            tray_icon: Some("monochrome".to_string()),
            menu_icon: Some("monochrome".to_string()),
            notice_position: Some("top-right".to_string()),
            collapse_navbar: Some(false),
            common_tray_icon: Some(false),
            sysproxy_tray_icon: Some(false),
            tun_tray_icon: Some(false),
            enable_auto_launch: Some(false),
            enable_silent_start: Some(false),
            enable_hover_jump_navigator: Some(true),
            hover_jump_navigator_delay: Some(280),
            enable_system_proxy: Some(false),
            proxy_auto_config: Some(false),
            pac_file_content: Some(DEFAULT_PAC.into()),
            proxy_host: Some("127.0.0.1".to_string()),
            #[cfg(not(target_os = "windows"))]
            verge_redir_port: Some(7895),
            #[cfg(target_os = "windows")]
            verge_redir_enabled: Some(false),
            #[cfg(target_os = "linux")]
            verge_tproxy_port: Some(7896),
            #[cfg(target_os = "linux")]
            verge_tproxy_enabled: Some(false),
            verge_mixed_port: Some(7897),
            verge_socks_port: Some(7898),
            verge_socks_enabled: Some(false),
            verge_port: Some(7899),
            verge_http_enabled: Some(false),
            enable_proxy_guard: Some(false),
            use_default_bypass: Some(true),
            proxy_guard_duration: Some(30),
            auto_close_connection: Some(true),
            enable_auto_check_update: Some(true),
            enable_builtin_enhanced: Some(true),
            auto_log_clean: Some(2), // 1: 1天, 2: 7天, 3: 30天, 4: 90天
            enable_auto_backup_schedule: Some(false),
            auto_backup_interval_hours: Some(24),
            auto_backup_on_change: Some(true),
            webdav_url: None,
            webdav_username: None,
            webdav_password: None,
            enable_tray_speed: Some(false),
            // enable_tray_icon: Some(true),
            tray_proxy_groups_display_mode: Some("default".into()),
            tray_inline_outbound_modes: Some(false),
            enable_global_hotkey: Some(true),
            enable_auto_light_weight_mode: Some(false),
            auto_light_weight_minutes: Some(10),
            enable_dns_settings: Some(false),
            home_cards: None,
            enable_external_controller: Some(false),
            ..Self::default()
        }
    }

    /// TODO 待删除
    /// 获取配置文件路径
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(VERGE_CONFIG_FILE);
        path
    }

    /// TODO 删除
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
    pub async fn save_file(&self) -> Result<()> {
        // 确保目录存在
        /*
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
        */
        help::save_yaml(&dirs::verge_path()?, &self, Some("# Clash Verge Config")).await
    }

    /// TODO 待删除
    /// 获取当前配置
    #[allow(dead_code)]
    pub fn get(&self) -> Arc<VergeConfigData> {
        self.data.load().clone()
    }

    /// 更新配置
    /// only save to file
    pub fn patch_config(&mut self, patch: &Self) {
        macro_rules! patch {
            ($key: tt) => {
                if patch.$key.is_some() {
                    self.$key = patch.$key.clone();
                }
            };
        }

        patch!(app_log_level);
        patch!(app_log_max_size);
        patch!(app_log_max_count);

        patch!(language);
        patch!(theme_mode);
        patch!(tray_event);
        patch!(env_type);
        patch!(start_page);
        patch!(startup_script);
        patch!(traffic_graph);
        patch!(enable_memory_usage);
        patch!(enable_group_icon);
        #[cfg(target_os = "macos")]
        patch!(tray_icon);
        patch!(menu_icon);
        patch!(menu_order);
        patch!(notice_position);
        patch!(collapse_navbar);
        patch!(common_tray_icon);
        patch!(sysproxy_tray_icon);
        patch!(tun_tray_icon);

        patch!(enable_tun_mode);
        patch!(enable_auto_launch);
        patch!(enable_silent_start);
        patch!(enable_hover_jump_navigator);
        patch!(hover_jump_navigator_delay);
        #[cfg(not(target_os = "windows"))]
        patch!(verge_redir_port);
        #[cfg(not(target_os = "windows"))]
        patch!(verge_redir_enabled);
        #[cfg(target_os = "linux")]
        patch!(verge_tproxy_port);
        #[cfg(target_os = "linux")]
        patch!(verge_tproxy_enabled);
        patch!(verge_mixed_port);
        patch!(verge_socks_port);
        patch!(verge_socks_enabled);
        patch!(verge_port);
        patch!(verge_http_enabled);
        patch!(enable_system_proxy);
        patch!(enable_proxy_guard);
        patch!(use_default_bypass);
        patch!(system_proxy_bypass);
        patch!(proxy_guard_duration);
        patch!(proxy_auto_config);
        patch!(pac_file_content);
        patch!(proxy_host);
        patch!(theme_setting);
        patch!(web_ui_list);
        patch!(clash_core);
        patch!(hotkeys);
        patch!(enable_global_hotkey);

        patch!(auto_close_connection);
        patch!(enable_auto_check_update);
        patch!(default_latency_test);
        patch!(default_latency_timeout);
        patch!(enable_auto_delay_detection);
        patch!(enable_builtin_enhanced);
        patch!(proxy_layout_column);
        patch!(test_list);
        patch!(auto_log_clean);
        patch!(enable_auto_backup_schedule);
        patch!(auto_backup_interval_hours);
        patch!(auto_backup_on_change);

        patch!(webdav_url);
        patch!(webdav_username);
        patch!(webdav_password);
        patch!(enable_tray_speed);
        // patch!(enable_tray_icon);
        patch!(tray_proxy_groups_display_mode);
        patch!(tray_inline_outbound_modes);
        patch!(enable_auto_light_weight_mode);
        patch!(auto_light_weight_minutes);
        patch!(enable_dns_settings);
        patch!(home_cards);
        patch!(enable_external_controller);
    }

    pub const fn get_singleton_port() -> u16 {
        crate::constants::network::ports::SINGLETON_SERVER
    }

    ///
    pub fn get_log_level(&self) -> LevelFilter {
        if let Some(level) = self.app_log_level.as_ref() {
            match level.to_lowercase().as_str() {
                "silent" => LevelFilter::Off,
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                _ => LevelFilter::Info,
            }
        } else {
            LevelFilter::Info
        }
    }

    // TODO 待删
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
