use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

use arc_swap::ArcSwap;
use futures::stream::StreamExt;
use tauri_plugin_shell::process::{CommandEvent, PtyEvent};
use tokio::process::Command as TokioCommand;
use tokio::sync::RwLock;

/// 运行模式
#[derive(Debug, Clone, PartialEq)]
pub enum RunningMode {
    /// 服务模式
    Service,
    /// 伴随进程模式
    Sidecar,
    /// 未运行
    NotRunning,
}

pub struct CoreManangerState {
    /// 运行模式
    running_mode: ArcSwap<RunningMode>,
    /// 子进程
    child: ArcSwapOption<Child>,
    /// 最后一次更新时间
    last_update: ArcSwapOption<std::time::Instant>,
}

impl Default for CoreManangerState {
    fn default() -> Self {
        Self {
            running_mode: ArcSwap::new(Arc::new(RunningMode::NotRunning)),
            child: ArcSwapOption::new(None),
            last_update: ArcSwapOption::new(None),
        }
    }
}

/// 核心管理器
#[derive(Debug, Clone)]
pub struct CoreManager {
    state: Arc<CoreManangerState>,
    /// 核心路径
    core_path: PathBuf,
    /// 核心参数
    config_path: PathBuf,
}

impl CoreManager {
    pub fn new(core_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            state: Arc::new(CoreManangerState::default()),
            core_path,
            config_path,
        }
    }

    /// 启动 Clash 核心
    pub async fn start(&self) -> Result<()> {
        if self.is_running() {
            anyhow::bail!("Core is already running");
        }

        // 检查配置文件是否存在
        if !self.config_path.exists() {
            anyhow::bail!("Config file not found: {}", self.config_path.display());
        }

        // 检查核心文件是否存在
        if !self.core_path.exists() {
            anyhow::bail!("Core file not found: {}", self.core_path.display());
        }

        // 启动进程

        let mut cmd = TokioCommand::new(&self.core_path)
            .args(&[
                "-d",
                self.config_path
                    .parent()
                    .unwrap_or(&PathBuf::from("."))
                    .to_str()
                    .unwrap_or("."),
            ])
            .args(&self.core_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn core process")?;

        // 等待进程启动完成
        tokio::time::sleep(Duration::from_secs(2)).await;

        // 检查进程是否正常运行
        if let Some(status) = cmd.try_wait()? {
            if status.code().is_some() {
                anyhow::bail!("Clash Core exited immediately with code: {:?}", status.code().unwrap());
            }
        }
        if !exit_status.success() {
            anyhow::bail!("Core process failed with exit status: {:?}", exit_status);
        }

        // 更新状态
        self.state.child.store(Some(Arc::new(child)));
        self.state.running_mode.store(Arc::new(RunningMode::Service));
        self.state.last_update.store(Some(std::time::Instant::now()));
        Ok(())
    }

    /// 停止 Clash 核心
    pub async fn stop(&self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        let child = self.state.child.swap(None);
        if let Some(child_arc) = child {
            let mut child =
                Arc::try_unwrap(child_arc).map_err(|_| anyhow::anyhow!("Failed to unwrap child process"))?;
            child.kill().await.context("Failed to kill Clash core")?;
            child.wait().await.context("Failed to wait for Clash core")?;
        }

        self.state.running_mode.store(Arc::new(RunningMode::NotRunning));

        Ok(())
    }

    /// 重启 Clash 核心
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(*self.state.running_mode.load(),
            RunningMode::Service | RunningMode::Sidecar)
    }


    /// 获取进程输出
    pub async fn get_output(&self) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        let child = self.state.child.load();
        if let Some(child_arc) = child {
            let child = Arc::clone(child_arc);
            // 获取stdout和stderr
            // 这里需要根据实际实现调整
            Ok(None)
        } else {
            Ok(None)
        }
    }
}
