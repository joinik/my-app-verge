use flexi_logger::{DeferredNow, LogSpec, Logger};


pub fn init_logger()->Result<(), flexi_logger::FlexiLoggerError>{
    let log_path = get_log_dir();
    let log_spec = LogSpec::default().default_filter_level(log::LevelFilter::Info);
    Logger::with(log_spec)
        .log_to_file()
        .directory(&log_path)
        .append()
        .format(|w, now: &mut DeferredNow, record, msg| {
            write!(
                w,
                "[{}] [{}] {} - {}:{}:{}",
                now.now().format("%Y-%m-%d %H:%M:%S%.6f"),
                record.level(),
                record.target(),
                record.file().unwrap_or("?"),
                record.line().unwrap_or(0),
                msg
            )
        })
        .start()
}

fn get_log_dir() -> PathBuf{
    let mut path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("my-new-app");
    path.push("logs");
    path
}


