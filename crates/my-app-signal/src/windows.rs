use std::sync::atomic::{AtomicBool, Ordering};

use clash_verge_logging::{Type, logging};
use tokio::signal::windows;

use crate::RUNTIME;

/// 全局原子标志，用于防止在收到重复信号时多次执行清理逻辑。
/// 一旦收到首个信号并开始清理，后续信号将被忽略。
static IS_CLEANING_UP: AtomicBool = AtomicBool::new(false);

/// 注册 Windows 平台的系统信号监听器。
///
/// 监听以下四种 Windows 控制台/系统信号：
/// - `Ctrl+C`：用户按下 Ctrl+C 中断程序
/// - `Ctrl+Close`：控制台窗口被关闭
/// - `Ctrl+Shutdown`：系统正在关机
/// - `Ctrl+Logoff`：用户正在注销
///
/// 当任意信号触发时，调用传入的清理闭包 `f`。
/// 通过 `IS_CLEANING_UP` 原子标志确保清理逻辑只执行一次，
/// 避免在快速连续收到多个信号时重复触发。
///
/// # 类型参数
/// - `F`: 无参闭包，返回一个 Future，需满足 `Send + Sync + 'static`
/// - `Fut`: 闭包返回的 Future，需满足 `Send + 'static`
pub fn register<F, Fut>(f: F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future + Send + 'static,
{
    // 尝试获取 RUNTIME：OnceLock 存储的是 Option<Runtime>，
    // 所以 RUNTIME.get() 返回 Option<&Option<Runtime>>，
    // 双层 Some 匹配表示 Runtime 已成功创建且已初始化
    if let Some(Some(rt)) = RUNTIME.get() {
        // 在 tokio 运行时上派生一个异步任务来监听信号
        rt.spawn(async move {
            // 注册 Ctrl+C 信号接收器
            let mut ctrl_c = match windows::ctrl_c() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+C: {}", e);
                    return;
                }
            };

            // 注册 Ctrl+Close 信号接收器（控制台窗口关闭时触发）
            let mut ctrl_close = match windows::ctrl_close() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+Close: {}", e);
                    return;
                }
            };

            // 注册 Ctrl+Shutdown 信号接收器（系统关机时触发）
            let mut ctrl_shutdown = match windows::ctrl_shutdown() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+Shutdown: {}", e);
                    return;
                }
            };

            // 注册 Ctrl+Logoff 信号接收器（用户注销时触发）
            let mut ctrl_logoff = match windows::ctrl_logoff() {
                Ok(s) => s,
                Err(e) => {
                    logging!(error, Type::SystemSignal, "Failed to register Ctrl+Logoff: {}", e);
                    return;
                }
            };

            // 持续监听信号，任意一个信号到达即触发处理逻辑
            loop {
                let signal_name;
                // tokio::select! 同时等待所有信号，哪个先到达就执行哪个分支
                tokio::select! {
                    _ = ctrl_c.recv() => {
                        signal_name = "Ctrl+C";
                    }
                    _ = ctrl_close.recv() => {
                        signal_name = "Ctrl+Close";
                    }
                    _ = ctrl_shutdown.recv() => {
                        signal_name = "Ctrl+Shutdown";
                    }
                    _ = ctrl_logoff.recv() => {
                        signal_name = "Ctrl+Logoff";
                    }
                }

                // 使用 SeqCst 顺序保证多线程间状态一致性：
                // 如果已经在清理中，跳过本次信号，避免重复执行清理闭包
                if IS_CLEANING_UP.load(Ordering::SeqCst) {
                    logging!(
                        info,
                        Type::SystemSignal,
                        "Already shutting down, ignoring repeated signal: {}",
                        signal_name
                    );
                    continue;
                }
                // 标记为正在清理，防止后续信号重复触发
                IS_CLEANING_UP.store(true, Ordering::SeqCst);

                logging!(info, Type::SystemSignal, "Caught Windows signal: {}", signal_name);

                // 执行用户传入的清理闭包（如释放资源、停止服务 etc.）
                f().await;
            }
        });
    } else {
        // RUNTIME 未初始化（创建失败），无法注册信号监听
        logging!(
            error,
            Type::SystemSignal,
            "register shutdown signal failed, RUNTIME is not available"
        );
    }
}
