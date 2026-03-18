use anyhow::Result;
use smartstring::alias::String as SmartString;
pub type CmdResult<T = ()> = Result<T, SmartString>;

pub mod app;
pub mod clash;
pub mod profile;
pub mod proxy;
pub mod system;

pub use app::*;
pub use clash::*;
pub use profile::*;
pub use proxy::*;
pub use system::*;

pub trait StringifyErr<T> {
    fn stringify_err(self) -> CmdResult<T>;
    fn stringify_err_log<F>(self, log_fn: F) -> CmdResult<T>
    where
        F: Fn(&str);
}

impl<T, E: std::fmt::Display> StringifyErr<T> for Result<T, E> {
    fn stringify_err(self) -> CmdResult<T> {
        self.map_err(|e| SmartString::from(e.to_string()))
    }

    fn stringify_err_log<F>(self, log_fn: F) -> CmdResult<T>
    where
        F: Fn(&str),
    {
        self.map_err(|e| {
            let msg = SmartString::from(e.to_string());
            log_fn(&msg);
            msg
        })
    }
}
