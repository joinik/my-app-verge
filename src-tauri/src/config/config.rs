use std::{path::PathBuf, thread::sleep};

use anyhow::{Result, anyhow};
use my_app_draft::Draft;
use my_app_logging::{Type, logging, logging_error};
use once_cell::sync::OnceCell;

use crate::{config::{clash::{IClash, IClashTemp}, prfitem::PrfItem, profiles::{IProfiles, profiles_append_item_safe}, verge::IVerge}, utils::dirs};

pub struct Config {
    clash_config: Draft<IClashTemp>,
    verge_config: Draft<IVerge>,
    profiles_config: Draft<IProfiles>,
    runtime_config: Draft<IRuntime>,
}

impl Config {
    pub async fn global() -> &'static Self {
        static CONFIG: OnceCell<Config> = OnceCell::const_new();
        CONFIG
            .get_or_init(|| async {
                Self {
                    clash_config: Draft::new(IClashTemp::new().await),
                    verge_config: Draft::new(IVerge::new().await),
                    profiles_config: Draft::new(IProfiles::new().await),
                    runtime_config: Draft::new(IRuntime::new().await),
                }
            })
            .await
    }

    pub async fn clash() -> Draft<IClashTemp> {
        Self::global().await.clash_config.clone()
    }
    pub async fn verge() -> Draft<IVerge> {
        Self::global().await.verge_config.clone()
    }
    pub async fn profiles() -> Draft<IProfiles> {
        Self::global().await.profiles_config.clone()
    }
    pub async fn runtime() -> Draft<IRuntime> {
        Self::global().await.runtime_config.clone()
    }

    /// 初始化订阅
    pub async fn init_config() -> Result<()> {
        Self::ensure_default_profile_items().await?;

        let verge = Self::verge().await.latest_arc();
        my_app_i18n::set_locale(verge.language.as_deref());

        // init Tun mode
        let handle = Handle::app_handle();
        let is_admin = is_current_process_admin(handle);
        let is_service_available = service::is_service_available().await.is_ok();
        if !is_admin && !is_service_available {
            let verge = Self::verge().await;
            verge.edit_draft(|d| {
                d.enable_tun_mode = Some(false)
            })
            verge.apply();
            let _ = tray::Tray::global().update_menu().await;

            // 分离数据获取和异步调用避免Send问题
            let verge_data = Self::verge().await.latest_arc();
            logging_error!(Type::Core, verge_data.save_file().await);
        }

        let validation_result = Self::generate_and_validate_config().await?;

        if let Some((msg_type, msg_content)) = validation_result {
            sleep(timing::STARTUP_ERROR_DELAY).await;
            handle::Handle::notice_message(msg_type, msg_content);

        }

        {
            let profiles = Self::profiles().await.latest_arc();
            let _ = profiles.cleanup_orphaned_files().await;
        }

        Ok(())
    }


    // Ensure "Merge" and "Script" profile items exist, adding them if missing.
    async fn ensure_default_profile_items() -> Result<()> {
        let profiles = Self::profiles().await;

        if profiles.latest_arc().get_item("Merge").is_err() {
            let merge_item = &mut PrfItem::from_merge(Some("Merge".into()))?;
            profiles_append_item_safe(merge_item).await?;
        }
        if profiles.latest_arc().get_item("Script").is_err() {
            let script_item = &mut PrfItem::from_script(Some("Script".into()))?;
            profiles_append_item_safe(script_item).await?;
        }
        Ok(())
    }

    async fn generate_and_validate() -> Result<Option<(&'static str, String)>> {
        // Generate runtime config
        if let Err(err) = Self::generate().await {
            logging!(error, Type::Config, "Failed to generate runtime config: {}", err);
        } else {
            logging!(info, Type::Config, "Runtime config generated successfully");
        }

        // Generate and validate runtime config
        let config_result = Self::generate_file(ConfigType::Run).await;

        if config_result.is_ok(){
            // Validate config
            logging!(info, Type::Config, "Config validation started");
            
            match CoreConfigValidator::global().validate_config().await {
                Ok((is_valid, error_msg)) => {
                    if !is_valid {
                        logging!(
                            warn,
                            Type::Config,
                            "[首次启动] 配置验证失败，使用默认最小配置启动: {}",
                            error_msg
                        );
                        CoreManager::global()
                            .use_default_config("config_validate::boot_error", &error_msg)
                            .await?;
                        Ok(Some(("config_validate::boot_error", error_msg)))
                    } else {
                        logging!(info, Type::Config, "Config validation succeeded");
                        Ok(None)
                    }
                }
                Err(err) => {
                    logging!(warn, Type::Config, "验证过程执行失败: {}", err);
                    CoreManager::global()
                        .use_default_config("config_validate::process_terminated", "")
                        .await?;
                    Ok(Some(("config_validate::process_terminated", String::new())))
                }
            }
        } else {
            logging!(warn, Type::Config, "生成配置文件失败，使用默认配置");
            CoreManager::global()
                .use_default_config("config_validate::error", "")
                .await?;
            Ok(Some(("config_validate::error", String::new())))
        }
    }

    pub async fn generate_file(typ: ConfigType) -> Result<PathBuf> {
        let path = match typ{
            ConfigType::Run => dirs::app_home_dir().unwrap().join(files::RUNTIME_CONFIG),
            ConfigType::Check => dirs::app_home_dir().unwrap().join(files::CHECK_CONFIG),
            
        };
        let runtime = Self::runtime().await;
        let runtime_arc = runtime.latest_arc();
        let config = runtime_arc.config.as_ref().ok_or_else(|| anyhow!("failed to get runtime config"))?;

        help::save_yaml(&path, config, Some("# Generated by Clash Verge")).await?;
        Ok(path)
    }
}


#[derive(Debug)]
pub enum ConfigType{
    Run,
    Check,
}