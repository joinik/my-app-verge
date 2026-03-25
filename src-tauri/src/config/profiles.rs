use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Ok, Result};
use arc_swap::ArcSwap;
use chrono::Local;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

const PROFILES_FILE: &str = "profiles.yaml";

/// 配置文件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum ProfileType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "merge")]
    Merge,
    #[serde(rename = "script")]
    Script,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct Profile {
    /// 唯一标识符
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 文件路径
    pub file: PathBuf,
    /// 配置类型
    #[serde(rename = "type")]
    pub type_: ProfileType,

    /// 远程URL http类型需要
    pub url: Option<String>,
    /// 最后更新时间
    pub updated_at: Option<u64>,
    /// 是否启用
    pub enabled: Option<bool>,
    /// 排序
    pub order: Option<i32>,
    /// 额外信息
    #[serde(default)]
    pub extra: ProfileExtra,
}

/// 额外信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[allow(dead_code)]
pub struct ProfileExtra {
    /// 上次流量
    pub upload: Option<u64>,
    /// 下载流量
    pub download: Option<u64>,
    /// 总流量
    pub total: Option<u64>,
    /// 更新时间
    pub update_time: Option<u64>,
    /// 订阅信息
    pub subscription_info: Option<SubscriptionInfo>,
}

/// 订阅信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[allow(dead_code)]
pub struct SubscriptionInfo {
    pub title: Option<String>,
    pub version: Option<String>,
    pub upload: Option<u64>,
    pub download: Option<u64>,
    pub total: Option<u64>,
    pub expire: Option<u64>,
}

/// 配置文件集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct ProfilesData {
    /// 配置文件列表
    pub items: Vec<Profile>,
    /// 当前选中配置 ID
    pub current: Option<String>,
    /// 是否自动更新
    pub auto_update: Option<bool>,
    /// 自动更新间隔
    pub update_interval: Option<u64>,
}

/// 配置文件管理器
#[derive(Debug)]
#[allow(dead_code)]
pub struct Profiles {
    /// 配置数据
    data: ArcSwap<ProfilesData>,
    /// 配置文件路径
    path: PathBuf,
}

impl Profiles {
    pub fn new() -> Self {
        Self {
            data: ArcSwap::new(Arc::new(ProfilesData::default())),
            path: Self::get_profiles_path(),
        }
    }

    pub fn get_profiles_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("my-new-app");
        path.push(PROFILES_FILE);
        path
    }

    #[allow(dead_code)]
    pub fn get_data(&self) -> Arc<ProfilesData> {
        self.data.load_full()
    }

    pub async fn load() -> Result<()> {
        let profiles = Self::new();

        if profiles.path.exists() {
            let content = tokio::fs::read_to_string(&profiles.path)
                .await
                .context("Failed to read profiles file")?;

            let data: ProfilesData = serde_yaml_ng::from_str(&content).context("Failed to parse profiles data")?;

            profiles.data.store(Arc::new(data));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create profiles directory")?;
        }

        let data = (*self.data.load_full()).clone();
        let content = serde_yaml_ng::to_string(&data).context("Failed to serialize profiles data")?;
        tokio::fs::write(&self.path, &content)
            .await
            .context("Failed to write profiles file")?;
        Ok(())
    }

    /// 获取当前配置
    #[allow(dead_code)]
    pub fn get(&self) -> Arc<ProfilesData> {
        self.data.load().clone()
    }

    #[allow(dead_code)]
    pub fn get_all(&self) -> Vec<Profile> {
        self.data.load().items.clone()
    }

    #[allow(dead_code)]
    pub fn get_current(&self) -> Option<Profile> {
        let data = self.data.load();
        data.current
            .as_ref()
            .and_then(|id| data.items.iter().find(|p| p.id == *id))
            .cloned()
    }

    #[allow(dead_code)]
    pub fn get_by_id(&self, id: &str) -> Option<Profile> {
        self.data.load().items.iter().find(|p| p.id == id).cloned()
    }

    #[allow(dead_code)]
    pub fn add(&self, profile: Profile) {
        let mut data = (*self.data.load_full()).clone();
        data.items.push(profile);
        self.data.store(Arc::new(data));
    }

    #[allow(dead_code)]
    pub fn update(&self, id: &str, new_profile: Profile) -> Result<()> {
        let mut data = (*self.data.load_full()).clone();
        if let Some(index) = data.items.iter().position(|p| p.id == id) {
            data.items[index] = new_profile;
            self.data.store(Arc::new(data));
            Ok(())
        } else {
            anyhow::bail!("Profile with id {} not found", id);
        }
    }

    #[allow(dead_code)]
    pub async fn delete(&self, id: &str) -> Result<()> {
        let mut data = (*self.data.load_full()).clone();
        let index = data
            .items
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("Profile with id {} not found", id))?;

        // 删除相关的文件
        if let Some(profile) = data.items.get(index) {
            if profile.file.exists() {
                tokio::fs::remove_file(&profile.file)
                    .await
                    .context("Failed to delete profile file")?;
            }
        }
        data.items.remove(index);

        if data.current.as_deref() == Some(id) {
            data.current = None;
        }
        self.data.store(Arc::new(data));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_current(&self, id: &str) -> Result<()> {
        let mut data = (*self.data.load_full()).clone();
        if !data.items.iter().any(|p| p.id == id) {
            anyhow::bail!("Profile with id {} not found", id);
        }
        data.current = Some(id.to_string());
        self.data.store(Arc::new(data));
        Ok(())
    }
    #[allow(dead_code)]
    fn get_profiles_dir(&self) -> PathBuf {
        self.path.parent().unwrap().to_path_buf()
        //  self.path.parent()
        //     .unwrap_or(&PathBuf::from("."))
        //     .to_path_buf()
    }

    #[allow(dead_code)]
    pub async fn create_profile(&self, name: &str, content: &str) -> Result<Profile> {
        let id = nanoid!();
        // 生成当前时间戳（毫秒级，u64 类型），用于记录配置文件的创建或更新时间
        let timestamp = Local::now().timestamp_millis() as u64;
        // 确保目录存在
        let profiles_dir = self.get_profiles_dir();
        tokio::fs::create_dir_all(&profiles_dir)
            .await
            .context("Failed to create profiles directory")?;

        // 生成文件路径
        let file_name = format!("{}_{}.yaml", timestamp, nanoid!(6));
        let file_path = profiles_dir.join(&file_name);

        // 写入文件
        tokio::fs::write(&file_path, content)
            .await
            .context("Failed to write profile file")?;

        let profile = Profile {
            id: id.clone(),
            name: name.to_string(),
            file: file_path,
            type_: ProfileType::File,
            url: None,
            updated_at: Some(timestamp),
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };
        self.add(profile.clone());
        Ok(profile)
    }
    /// 更新流量统计
    ///
    /// # Arguments
    ///
    /// * `id` - 配置文件ID
    /// * `upload` - 上传流量（字节）
    /// * `download` - 下载流量（字节）
    #[allow(dead_code)]
    pub fn update_traffic(&self, id: &str, upload: u64, download: u64) -> Result<()> {
        let mut data = (*self.data.load_full()).clone();
        if let Some(profile) = data.items.iter_mut().find(|p| p.id == id) {
            profile.extra.upload = Some(upload);
            profile.extra.download = Some(download);
            self.data.store(Arc::new(data));
            Ok(())
        } else {
            anyhow::bail!("Profile with id {} not found", id);
        }
    }

    /// 重新排序配置文件
    ///
    /// # Arguments
    ///
    /// * `ids` - 配置文件ID列表，按新顺序排列
    #[allow(dead_code)]
    pub fn reorder(&self, ids: Vec<String>) -> Result<()> {
        let mut data = (*self.data.load_full()).clone();
        for (index, id) in ids.iter().enumerate() {
            if let Some(profile) = data.items.iter_mut().find(|p| p.id == *id) {
                profile.order = Some(index as i32);
            }
        }
        self.data.store(Arc::new(data));
        Ok(())
    }
}

impl Default for Profiles {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置文件管理器扩展trait
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait ProfilesExt {
    async fn enhance(&self, profile_id: &str) -> Result<()>;
}

#[async_trait::async_trait]
impl ProfilesExt for Profiles {
    async fn enhance(&self, _profile_id: &str) -> Result<()> {
        // 实现配置增强逻辑
        // 这里可以根据需要实现规则增强等功能
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profile_creation() {
        let profiles = Profiles::new();
        assert!(profiles.get_all().is_empty());
    }

    #[tokio::test]
    async fn test_add_profile() {
        let profiles = Profiles::new();

        let profile = Profile {
            id: "test-1".to_string(),
            name: "Test Profile".to_string(),
            file: PathBuf::from("/test/profile.yaml"),
            type_: ProfileType::File,
            url: None,
            updated_at: None,
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };

        profiles.add(profile.clone());

        assert_eq!(profiles.get_all().len(), 1);
        assert_eq!(profiles.get_by_id("test-1"), Some(profile));
    }

    #[tokio::test]
    async fn test_delete_profile() {
        let profiles = Profiles::new();

        let profile = Profile {
            id: "test-2".to_string(),
            name: "Test Profile".to_string(),
            file: PathBuf::from("/test/profile.yaml"),
            type_: ProfileType::File,
            url: None,
            updated_at: None,
            enabled: Some(true),
            order: None,
            extra: ProfileExtra::default(),
        };

        profiles.add(profile.clone());
        profiles.delete("test-2").await.unwrap();

        assert!(profiles.get_all().is_empty());
    }
}
