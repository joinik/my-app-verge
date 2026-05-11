use crate::{core::handle, utils::resolve::window::build_new_window};
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use my_app_logging::{Type, logging};
use once_cell::sync::Lazy;
use std::time::Duration;
use std::{num::NonZeroU32, pin::Pin};
use tauri::{WebviewWindow, Wry};

/// 窗口操作结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowOperationResult {
    /// 窗口已显示并获得焦点
    Shown,
    /// 窗口已隐藏
    Hidden,
    /// 创建了新窗口
    Created,
    /// 摧毁了窗口
    Destroyed,
    /// 操作失败
    Failed,
    /// 无需操作
    NoAction,
}

/// 窗口状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowState {
    /// 窗口可见且有焦点
    VisibleFocused,
    /// 窗口可见但无焦点
    VisibleUnfocused,
    /// 窗口最小化
    Minimized,
    /// 窗口隐藏
    Hidden,
    /// 窗口不存在
    NotExist,
}

// 窗口操作防抖机制
const WINDOW_OPERATION_DEBOUNCE_MS: u64 = 1_275;
static WINDOW_OPERATION_LIMITER: Lazy<DefaultDirectRateLimiter> = Lazy::new(|| {
    #[allow(clippy::unwrap_used)]
    RateLimiter::direct(
        Quota::with_period(Duration::from_millis(WINDOW_OPERATION_DEBOUNCE_MS))
            .unwrap()
            .allow_burst(NonZeroU32::new(1).unwrap()),
    )
});

fn should_perform_window_operation() -> bool {
    let res = WINDOW_OPERATION_LIMITER.check().is_ok();
    if !res {
        logging!(debug, Type::Window, "Window operation rate limit exceeded")
    }
    res
}

pub struct WindowManager;
impl WindowManager {
    pub fn get_main_window_state() -> WindowState {
        match Self::get_main_window() {
            Some(window) => {
                let is_minimized = window.is_minimized().unwrap_or(false);
                let is_visible = window.is_visible().unwrap_or(false);
                let is_focused = window.is_focused().unwrap_or(false);

                if is_minimized {
                    return WindowState::Minimized;
                }

                if !is_visible {
                    return WindowState::Hidden;
                }

                if is_focused {
                    WindowState::VisibleFocused
                } else {
                    WindowState::VisibleUnfocused
                }
            }

            None => WindowState::NotExist,
        }
    }

    /// 获取主窗口实例
    pub fn get_main_window() -> Option<WebviewWindow<Wry>> {
        let app_handle = handle::Handle::app_handle();
        app_handle.get_webview_window("main")
    }

    pub async fn show_main_window() -> WindowOperationResult {
        // 防抖检查
        if !should_perform_window_operation() {
            return WindowOperationResult::NoAction;
        }
        logging!(info, Type::Window, "开始智能显示主窗口");
        logging!(debug, Type::Window, "{}", Self::get_window_status_info());

        let current_state = Self::get_main_window_state();

        match current_state {
            WindowState::VisibleFocused => {
                logging!(info, Type::Window, "主窗口已可见,且有焦点, 无需操作");
                WindowOperationResult::NoAction
            }
            WindowState::Minimized | WindowState::VisibleUnfocused | WindowState::Hidden => {
                if let Some(window) = Self::get_main_window() {
                    let state_after_check = Self::get_main_window_state();
                    if state_after_check == WindowState::VisibleUnfocused {
                        logging!(info, Type::Window, "窗口在检查期间已变为可见和有焦点状态");
                        return WindowOperationResult::NoAction;
                    }
                    Self::activate_window(&window)
                } else {
                    WindowOperationResult::Failed
                }
            }
            WindowState::NotExist => {
                logging!(info, Type::Window, "主窗口不存在,创建新窗口");
                if Self::create_window(true).await {
                    logging!(info, Type::Window, "新窗口创建成功");
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    WindowOperationResult::Created
                } else {
                    logging!(error, Type::Window, "新窗口创建失败");
                    WindowOperationResult::Failed
                }
            }
        }
    }

    /// 切换主窗口显示状态 (显示/隐藏)
    pub async fn toggle_main_window() -> WindowOperationResult {
        // 防抖检查
        if !should_perform_window_operation() {
            return WindowOperationResult::NoAction;
        }
        logging!(info, Type::Window, "开始切换主窗口显示状态");
        let current_state = Self::get_main_window_state();

        logging!(
            info,
            Type::Window,
            "当前主窗口状态: {:?}| 详细状态: {}",
            current_state,
            Self::get_window_status_info()
        );

        match current_state {
            WindowState::VisibleFocused | WindowState::VisibleUnfocused => Self::hide_main_window(),
            WindowState::Minimized | WindowState::Hidden => Self::activate_existing_main_window(),
            WindowState::NotExist => Self::handle_not_exist_toggle().await,
        }
    }

    // 窗口不存在时创建新窗口
    async fn handle_not_exist_toggle() -> WindowOperationResult {
        logging!(info, Type::Window, "主窗口不存在,将创建新窗口");
        // 由于已经有防抖保护，直接调用内部方法
        if Self::create_window(true).await {
            WindowOperationResult::Created
        } else {
            WindowOperationResult::Failed
        }
    }

    // 隐藏主窗口
    fn hide_main_window() -> WindowOperationResult {
        logging!(info, Type::Window, "窗口可见，开始隐藏主窗口");
        if let Some(window) = Self::get_main_window() {
            match window.hide() {
                Ok(_) => {
                    logging!(info, Type::Window, "窗口已成功隐藏");
                    WindowOperationResult::Hidden
                }
                Err(e) => {
                    logging!(warn, Type::Window, "隐藏窗口失败: {}", e);
                    WindowOperationResult::Failed
                }
            }
        } else {
            logging!(error, Type::Window, "无法获取主窗口实例");
            WindowOperationResult::Failed
        }
    }

    /// 激活已存在主窗口
    fn activate_existing_main_window() -> WindowOperationResult {
        logging!(info, Type::Window, "窗口存在但被隐藏或最小化，开始激活主窗口");
        if let Some(window) = Self::get_main_window() {
            Self::activate_window(&window)
        } else {
            logging!(error, Type::Window, "无法获取主窗口实例");
            WindowOperationResult::Failed
        }
    }

    /// 激活窗口 (取消最小化，显示，设置焦点)
    fn activate_window(window: &WebviewWindow<Wry>) -> WindowOperationResult {
        logging!(info, Type::Window, "开始激活窗口");
        let mut operations_successful = true;

        // 1. 如果窗口最小化，先取消最小化
        if window.is_minimized().unwrap_or(false) {
            logging!(info, Type::Window, "窗口已最小化，开始取消最小化");
            if let Err(e) = window.unminimize() {
                logging!(warn, Type::Window, "取消最小化失败: {}", e);
                operations_successful = false;
            }
        }

        // 2. 显示窗口
        if let Err(e) = window.show() {
            logging!(warn, Type::Window, "显示窗口失败: {}", e);
            operations_successful = false;
        }

        // 3. 设置焦点
        if let Err(e) = window.set_focus() {
            logging!(warn, Type::Window, "设置焦点失败: {}", e);
            operations_successful = false;
        }

        // 4. 平台特定的激活策略
        #[cfg(target_os = "macos")]
        {
            logging!(info, Type::Window, "应用macOS 特定激活策略");
            handle::Handle::global().set_activation_policy_regular();
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 尝试额外的激活方法
            if let Err(e) = window.set_always_on_top(true) {
                logging!(warn, Type::Window, "设置始终在顶部失败: {}", e);
            }
            // 立即取消置顶
            if let Err(e) = window.set_always_on_top(false) {
                logging!(warn, Type::Window, "取消始终在顶部失败(非关键错误): {}", e);
            }
        }

        if operations_successful {
            logging!(info, Type::Window, "窗口已成功激活");
            WindowOperationResult::Shown
        } else {
            logging!(warn, Type::Window, "窗口激活失败");
            WindowOperationResult::Failed
        }
    }

    /// 检查窗口是否可见
    pub fn is_main_window_visible() -> bool {
        Self::get_main_window()
            .map(|windows| windows.is_focused().unwrap_or(false))
            .unwrap_or(false)
    }

    /// 检查窗口是否最小化
    pub fn is_main_window_minimized() -> bool {
        Self::get_main_window()
            .map(|windows| windows.is_minimized().unwrap_or(false))
            .unwrap_or(false)
    }

    /// 创建主窗口
    pub fn create_window(is_show: bool) -> Pin<Box<dyn Future<Output = bool> + Send>> {
        Box::pin(async move {
            logging!(info, Type::Window, "开始创建窗口 is_show={}", is_show);
            if !is_show {
                return false;
            }
            let window = match build_new_window().await {
                Ok(window) => {
                    logging!(info, Type::Window, "窗口创建成功");
                    window
                }
                Err(e) => {
                    logging!(error, Type::Window, "创建窗口失败: {}", e);
                    return false;
                }
            };

            // 直接激活刚才创建的窗口，避免因抖动导致首次显示
            if Self::activate_window(&window) == WindowOperationResult::Failed {
                return false;
            }

            handle::Handle::global().mark_startup_completed();
            true
        })
    }

    /// 销毁主窗口
    pub fn destroy_main_window() -> WindowOperationResult {
        if let Some(window) = Self::get_main_window() {
            let _ = window.destroy();
            logging!(info, Type::Window, "窗口已摧毁");
            #[cfg(target_os = "macos")]
            {
                logging!(info, Type::Window, "应用macOS 特定激活策略");
                handle::Handle::global().set_activation_policy_accessory();
            }
            return WindowOperationResult::Destroyed;
        }
        WindowOperationResult::Failed
    }

    /// 检查窗口是否被聚焦
    pub fn is_main_window_focused() -> bool {
        Self::get_main_window()
            .map(|windows| windows.is_focused().unwrap_or(false))
            .unwrap_or(false)
    }
    pub fn get_window_status_info() -> String {
        let state = Self::get_main_window_state();
        let is_visible = Self::is_main_window_visible();
        let is_focused = Self::is_main_window_focused();
        let is_minimized = Self::is_main_window_minimized();
        format!("窗口状态: {state:?} | 可见: {is_visible} | 有焦点: {is_focused} | 最小化: {is_minimized}")
    }
}
