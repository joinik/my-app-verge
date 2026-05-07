//! 嵌入式 HTTP 服务器模块
//!
//! 本模块实现了一个轻量级的嵌入式 HTTP 服务器，基于 `warp` 框架，
//! 主要用于实现单例进程模式（singleton）和 PAC 代理自动配置脚本服务。
//! 服务器绑定到 `127.0.0.1:{singleton_port}`，仅监听本地回环地址。
//!
//! ## 提供的 API 端点
//! - `/commands/visible` — 通知已有实例显示应用窗口
//! - `/commands/pac` — 返回 PAC 代理自动配置脚本内容
//! - `/commands/scheme` — 处理 `clash://` 协议链接
//!
//! ## 设计说明
//! 服务器通过 `oneshot` channel 实现优雅关闭（graceful shutdown），
//! 配合 warp 的 `bind_with_graceful_shutdown` 机制，确保服务器
//! 在处理完当前请求后再退出。

use super::resolve;
use crate::{
    cmd::is_port_in_use,
    config::{Config, DEFAULT_PAC, IVerge},
    module::lightweight,
    process::AsyncHandler,
    utils::window_manager::WindowManager,
};
use anyhow::{Result, bail};
use my_app_logging::{Type, logging, logging_error};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use reqwest::ClientBuilder;
use smartstring::alias::String;
use std::time::Duration;
use tokio::sync::oneshot;
use warp::Filter as _;

/// 嵌入式 HTTP 服务器的 URL 查询参数。
///
/// 用于解析 `clash://` 协议链接中的查询字符串，
/// 例如 `/commands/scheme?param=clash://xxxx`。
#[derive(serde::Deserialize, Debug)]
struct QueryParam {
    /// URL 查询参数的值，包含完整的 `clash://` 协议链接内容
    param: String,
}

/// 用于优雅关闭嵌入式服务器的 oneshot 信号发送端。
///
/// 当调用 [`shutdown_embedded_server()`] 时，通过此 Sender 发送关闭信号，
/// 触发 warp 服务器的 graceful shutdown 流程。
/// 采用 `OnceCell<Mutex<Option<>>>` 确保线程安全且只能发送一次关闭信号。
static SHUTDOWN_SENDER: OnceCell<Mutex<Option<oneshot::Sender<()>>>> = OnceCell::new();

/// 检查单例模式：检测端口是否已被占用（即已有实例运行）。
///
/// ## 工作流程
/// 1. 获取配置中的单例端口号
/// 2. 尝试连接该端口判断是否已被占用
/// 3. 如果端口被占用（已有实例在运行）：
///    - 若有命令行参数携带 `clash://` 协议链接，发送 scheme 指令给已有实例处理
///    - 若无命令行参数，发送 visible 指令让已有实例显示窗口
///    - 记录错误日志并 bail 退出，确保只运行一个实例
/// 4. 如果端口未被占用，返回 `Ok(())` 继续正常启动流程
pub async fn check_singleton() -> Result<()> {
    let port = IVerge::get_singleton_port();
    // 检测端口占用情况，若端口已被占用则说明已有实例在运行
    if is_port_in_use(port) {
        // 创建 500ms 超时的 HTTP 客户端，避免阻塞过久
        let client = ClientBuilder::new().timeout(Duration::from_millis(500)).build()?;

        // 收集命令行参数，用于检测是否有 clash:// 协议链接需要处理
        // needless_collect 允许：需要 Send + 'static 的生命周期要求
        #[allow(clippy::needless_collect)]
        let argvs: Vec<std::string::String> = std::env::args().collect();
        if argvs.len() > 1 {
            // macOS 下不允许通过命令行发送 scheme，强制在 app delegate 中处理
            #[cfg(not(target_os = "macos"))]
            {
                let param = argvs[1].as_str();
                if param.starts_with("clash:") {
                    // 将 clash:// 协议链接转发给已有实例处理
                    client
                        .get(format!("http://127.0.0.1:{port}/commands/scheme?param={param}"))
                        .send()
                        .await?;
                }
            }
        } else {
            // 无命令行参数，直接通知已有实例显示主窗口
            client
                .get(format!("http://127.0.0.1:{port}/commands/visible"))
                .send()
                .await?;
        }
        logging!(error, Type::Window, "failed to setup singleton listen server");
        bail!("app exists");
    }
    Ok(())
}

/// 启动嵌入式 HTTP 服务器，提供三个本地 API 端点用于进程间通信。
///
/// 服务器绑定到 `127.0.0.1:{singleton_port}`（仅限本地回环地址），
/// 用于单例模式下已有实例与新建实例之间的通信。
///
/// ## 路由说明
/// - `GET /commands/visible` — 通知已有实例退出轻量模式并显示窗口
/// - `GET /commands/pac` — 返回 PAC 代理自动配置脚本（替换 `%mixed-port%` 占位符）
/// - `GET /commands/scheme?param=...` — 处理 `clash://` 协议链接
///
/// ## 生命周期
/// 服务器通过 `async_runtime::spawn` 在后台异步任务中运行，
/// 配合 `oneshot::channel` 实现外部触发的优雅关闭。
pub fn embed_server() {
    // 创建 oneshot channel，用于触发服务器优雅关闭
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    #[allow(clippy::expect_used)]
    SHUTDOWN_SENDER
        .set(Mutex::new(Some(shutdown_tx)))
        .expect("failed to set shutdown signal for embedded server");
    let port = IVerge::get_singleton_port();

    // --- 路由：/commands/visible ---
    // 用于单例模式：新实例通知已有实例退出轻量模式并显示主窗口
    let visible = warp::path!("commands" / "visible").and_then(|| async {
        logging!(info, Type::Window, "检测到从单例模式恢复应用窗口");
        if !lightweight::exit_lightweight_mode().await {
            WindowManager::show_main_window().await;
        } else {
            logging!(error, Type::Window, "轻量模式退出失败，无法恢复应用窗口")
        }
        Ok::<_, warp::Rejection>(warp::reply::with_status::<std::string::String>(
            "ok".to_string(),
            warp::http::StatusCode::OK,
        ))
    });

    // --- 路由：/commands/pac ---
    // 返回 PAC（Proxy Auto-Config）代理自动配置脚本内容
    // 将脚本中的 %mixed-port% 占位符替换为实际的混合代理端口号
    let pac = warp::path!("commands" / "pac").and_then(|| async move {
        let verge_config = Config::verge().await;
        let clash_config = Config::clash().await;

        // 获取 PAC 文件内容，若无自定义配置则使用默认 PAC 脚本
        let pac_content = verge_config
            .data_arc()
            .pac_file_content
            .clone()
            .unwrap_or_else(|| DEFAULT_PAC.into());

        // 获取混合代理端口号，优先使用 verge 配置，否则从 clash 配置获取
        let pac_port = verge_config
            .data_arc()
            .verge_mixed_port
            .unwrap_or_else(|| clash_config.data_arc().get_mixed_port());

        // 将 PAC 脚本中的 %mixed-port% 占位符替换为实际端口号
        let processed_content = pac_content.replace("%mixed-port%", &format!("{pac_port}"));
        Ok::<_, warp::Rejection>(
            warp::http::Response::builder()
                .header("Content-Type", "application/x-ns-proxy-autoconfig")
                .body(processed_content)
                .unwrap_or_default(),
        )
    });

    // --- 路由：/commands/scheme ---
    // 处理 clash:// 协议链接的异步解析
    // 注意：使用 map 而非 and_then 可以避免异步闭包导致的 Send 问题，
    // 此处仍使用 and_then，但通过 AsyncHandler::spawn 将耗时操作移到独立任务中执行
    let scheme = warp::path!("commands" / "scheme")
        .and(warp::query::<QueryParam>())
        .and_then(|query: QueryParam| async move {
            // 在独立异步任务中执行 scheme 解析，避免阻塞 warp 工作线程
            AsyncHandler::spawn(|| async move {
                logging_error!(Type::Setup, resolve::resolve_scheme(&query.param).await);
            });
            Ok::<_, warp::Rejection>(warp::reply::with_status::<std::string::String>(
                "ok".to_string(),
                warp::http::StatusCode::OK,
            ))
        });

    // 组合所有路由为一个统一的 Filter，按优先级排列：visible > scheme > pac
    let commands = visible.or(scheme).or(pac);

    // 在后台异步任务中启动 warp 服务器，支持 graceful shutdown
    // 使用 AsyncHandler::spawn 确保任务在应用的主 tokio runtime 上运行
    AsyncHandler::spawn(move || async move {
        warp::serve(commands)
            .bind(([127, 0, 0, 1], port)) // 仅监听本地回环地址，保证安全性
            .await
            .graceful(async {
                shutdown_rx.await.ok(); // 等待关闭信号
            })
            .run()
            .await;
    });
}

/// 优雅关闭嵌入式 HTTP 服务器。
///
/// 通过全局 `SHUTDOWN_SENDER` 发送 oneshot 信号，触发 warp 服务器的 graceful shutdown。
/// 服务器会在处理完当前正在进行的请求后安全退出。
///
/// ## 安全性
/// - 使用链式 `if let`（Rust edition 2024 语法）安全地取出 Sender
/// - `Mutex::take()` 确保 Sender 只能被消费一次，防止重复发送关闭信号
/// - `sender.send(())` 返回 `Result`，使用 `.ok()` 忽略接收端已丢弃的错误
pub fn shutdown_embedded_server() {
    logging!(info, Type::Window, "shutting down embedded server");
    // 链式 if let：先检查 OnceCell 是否已初始化，再取出 Mutex 中的 Sender
    if let Some(sender) = SHUTDOWN_SENDER.get()
        && let Some(sender) = sender.lock().take()
    {
        // 发送关闭信号，触发 warp graceful shutdown
        sender.send(()).ok();
    }
}
