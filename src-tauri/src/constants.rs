/// Application constants
pub mod files {
    use std::path::PathBuf;

    /// Config directory name
    #[allow(dead_code)]
    pub const APP_CONFIG_DIR: &str = "my-new-app";

    /// Verge config file name
    #[allow(dead_code)]
    pub const VERGE_CONFIG: &str = "verge.yaml";

    /// Clash config file name
    #[allow(dead_code)]
    pub const CLASH_CONFIG: &str = "config.yaml";

    /// Profiles directory
    #[allow(dead_code)]
    pub const PROFILES_DIR: &str = "profiles";

    /// Logs directory
    #[allow(dead_code)]
    pub const LOGS_DIR: &str = "logs";

    /// Core directory
    #[allow(dead_code)]
    pub const CORE_DIR: &str = "core";

    /// Icons directory
    #[allow(dead_code)]
    pub const ICONS_DIR: &str = "icons";

    /// Window state file
    #[allow(dead_code)]
    pub const WINDOW_STATE: &str = "window-state.json";

    /// Get app config directory
    #[allow(dead_code)]
    pub fn app_config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_CONFIG_DIR)
    }

    /// Get profiles directory
    #[allow(dead_code)]
    pub fn profiles_dir() -> PathBuf {
        app_config_dir().join(PROFILES_DIR)
    }

    /// Get logs directory
    #[allow(dead_code)]
    pub fn logs_dir() -> PathBuf {
        app_config_dir().join(LOGS_DIR)
    }

    /// Get core directory
    #[allow(dead_code)]
    pub fn core_dir() -> PathBuf {
        app_config_dir().join(CORE_DIR)
    }

    /// Get icons directory
    #[allow(dead_code)]
    pub fn icons_dir() -> PathBuf {
        app_config_dir().join(ICONS_DIR)
    }

    /// Get verge config path
    #[allow(dead_code)]
    pub fn verge_config() -> PathBuf {
        app_config_dir().join(VERGE_CONFIG)
    }

    /// Get clash config path
    #[allow(dead_code)]
    pub fn clash_config() -> PathBuf {
        app_config_dir().join(CLASH_CONFIG)
    }
}

/// App info
pub mod app {
    #[allow(dead_code)]
    pub const APP_NAME: &str = "My New App";
    #[allow(dead_code)]
    pub const APP_VERSION: &str = "0.1.0";
    #[allow(dead_code)]
    pub const APP_AUTHOR: &str = "You";
    #[allow(dead_code)]
    pub const APP_REPO: &str = "https://github.com/yourusername/my-new-app";
}

/// Default values
pub mod defaults {
    #[allow(dead_code)]
    pub const DEFAULT_PORT: u16 = 9090;
    #[allow(dead_code)]
    pub const DEFAULT_SOCKS_PORT: u16 = 7891;
    #[allow(dead_code)]
    pub const DEFAULT_HTTP_PORT: u16 = 7890;
    #[allow(dead_code)]
    pub const DEFAULT_MIXED_PORT: u16 = 7892;
}

/// Internal server
pub mod server {
    #[allow(dead_code)]
    pub const SERVER_PORT: u16 = 11235;
}

/// Import dirs crate
mod dirs {
    use std::path::PathBuf;

    #[allow(dead_code)]
    pub fn config_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            use std::env;
            env::var_os("APPDATA").map(PathBuf::from)
        }

        #[cfg(target_os = "macos")]
        {
            dirs_next::home_dir().map(|home| home.join("Library").join("Application Support"))
        }

        #[cfg(target_os = "linux")]
        {
            use std::env;
            env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| dirs_next::home_dir().map(|home| home.join(".config")))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }
}

/// Dependencies
#[cfg(target_os = "macos")]
extern crate dirs_next;

#[cfg(target_os = "linux")]
extern crate dirs_next;
