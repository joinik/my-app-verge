use compact_str::CompactString;
use flexi_logger::DeferredNow;
#[cfg(not(feature = "tauri-dev"))]
use flexi_logger::filter::LogLineFilter;
use flexi_logger::writers::FileLogWriter;
use flexi_logger::writers::LogWriter as _;
use log::Level;
use log::Record;
use std::{fmt, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

pub type SharedWriter = Arc<Mutex<FileLogWriter>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Cmd,
    Core,
    Config,
    Setup,
    System,
    SystemSignal,
    Service,
    Hotkey,
    Window,
    Tray,
    Timer,
    Frontend,
    Backup,
    File,
    Lightweight,
    Network,
    ProxyMode,
    Validate,
    ClashVergeRev,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cmd => write!(f, "[Cmd]"),
            Self::Core => write!(f, "[Core]"),
            Self::Config => write!(f, "[Config]"),
            Self::Setup => write!(f, "[Setup]"),
            Self::System => write!(f, "[System]"),
            Self::SystemSignal => write!(f, "[SystemSignal]"),
            Self::Service => write!(f, "[Service]"),
            Self::Hotkey => write!(f, "[Hotkey]"),
            Self::Window => write!(f, "[Window]"),
            Self::Tray => write!(f, "[Tray]"),
            Self::Timer => write!(f, "[Timer]"),
            Self::Frontend => write!(f, "[Frontend]"),
            Self::Backup => write!(f, "[Backup]"),
            Self::File => write!(f, "[File]"),
            Self::Lightweight => write!(f, "[Lightweight]"),
            Self::Network => write!(f, "[Network]"),
            Self::ProxyMode => write!(f, "[ProxyMode]"),
            Self::Validate => write!(f, "[Validate]"),
            Self::ClashVergeRev => write!(f, "[ClashVergeRev]"),
        }
    }
}

#[macro_export]
macro_rules! logging {
    // 不带 print 参数的版本 （默认不打印）
    ($level:ident, $type:expr, $($arg:tt)*) =>{
        log::$level!(target:"app", "{} {}", $type, format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! logging_error {
    // Handle Result<T, E>
    ($type:expr, $expr:expr) => {
        if let Err(err) = $expr {
            log::error!(target: "app", "[{}] {}", $type, err);
        }
    };

    // Handle formatted message: always print to stdout and log as error
    ($type:expr, $fmt:literal $(, $arg:tt)*) => {
        log::error!(target: "app", "[{}] {}", $type, format_args!($fmt, $($arg)*));
    };
}

/// 将 sidecar 日志写入文件日志写入器
///
/// # 参数
/// - `writer`: 文件日志写入器的互斥锁守卫，确保线程安全
/// - `now`: 延迟时间戳，用于记录日志时间
/// - `level`: 日志级别（如 Info、Warn、Error 等）
/// - `message`: 要写入的日志消息内容（原参数名 `logger` 语义不清，实际是消息字符串）
///
/// # 说明
/// 1. 通过 `format_args!` 构造格式化参数
/// 2. 使用 `Record::builder()` 构建日志记录，target 固定为 "sidecar"
/// 3. 调用 writer 的 write 方法将日志写入文件，忽略写入错误
#[inline]
pub fn write_sidecar_log(
    writer: MutexGuard<'_, FileLogWriter>,
    now: &mut DeferredNow,
    level: Level,
    message: &CompactString, // 日志消息内容
) {
    // 构造格式化参数，将 message 包装为 fmt::Arguments
    let args = format_args!("{}", message);

    // 使用建造者模式构建日志 Record：
    // - args: 日志内容
    // - level: 日志级别
    // - target: 固定为 "sidecar"，用于标识日志来源
    let record = Record::builder().args(args).level(level).target("sidecar").build();

    // 调用 flexi_logger 的 writer 方法写入日志，忽略可能的 IO 错误
    let _ = writer.write(now, &record);
}

#[cfg(not(feature = "tauri-dev"))]
pub struct NoModuleFilter<'a>(pub &'a [&'a str]);

#[cfg(not(feature = "tauri-dev"))]
impl<'a> NoModuleFilter<'a> {
    #[inline]
    pub fn filter(&self, record: &Record) -> bool {
        if let Some(module) = record.module_path() {
            for blocked in self.0 {
                if module.len() >= blocked.len() && module.as_bytes()[..blocked.len()] == blocked.as_bytes()[..] {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(not(feature = "tauri-dev"))]
impl<'a> LogLineFilter for NoModuleFilter<'a> {
    #[inline]
    fn write(
        &self,
        now: &mut DeferredNow,
        record: &Record,
        writer: &dyn flexi_logger::filter::LogLineWriter,
    ) -> std::io::Result<()> {
        if !self.filter(record) {
            return Ok(());
        }
        writer.write(now, record)
    }
}
