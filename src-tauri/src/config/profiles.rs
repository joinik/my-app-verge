use std::{collections::HashSet, path::PathBuf};

use anyhow::{Context, Result, bail};
use my_app_logging::{Type, logging};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Mapping;
use smartstring::alias::String;
use tokio::fs;

use crate::{
    config::{
        config::Config,
        prfitem::{PrfItem, PrfOption},
    },
    utils::dirs::{self, PathBufExec},
};

/// Profiles 配置文件名
const PROFILES_FILE: &str = "profiles.yaml";

/// 配置文件类型：本地文件(File) 或 远程订阅(Remote)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    #[default]
    File,
    Remote,
}

/// 流量统计与过期信息（用于远程订阅节点）
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProfileExtra {
    /// 上传流量（字节）
    pub upload: Option<u64>,
    /// 下载流量（字节）
    pub download: Option<u64>,
    /// 总流量（字节）
    pub total: Option<u64>,
    /// 过期时间戳
    pub expire: Option<u64>,
}

/// 单条本地配置文件的元数据结构（持久化到 disk）
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Profile {
    /// 唯一标识符
    pub id: String,
    /// 用户自定义名称
    pub name: String,
    /// 配置文件在磁盘上的路径
    pub file: PathBuf,
    /// 配置文件类型（File / Remote），序列化为 "type"
    #[serde(rename = "type")]
    pub type_: ProfileType,
    /// 远程订阅 URL（仅 Remote 类型）
    pub url: Option<String>,
    /// 最后更新时间戳（毫秒）
    pub updated_at: Option<u64>,
    /// 是否启用
    pub enabled: Option<bool>,
    /// 排序权重
    pub order: Option<i32>,
    /// 流量/过期等额外信息
    pub extra: ProfileExtra,
}

/// 运行时 Profiles 集合（使用 Draft 管理并发访问）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IProfiles {
    /// 当前激活的 profile 的 uid（对应 PrfConfig.current）
    pub current: Option<String>,
    /// Profile 条目列表
    pub items: Option<Vec<PrfItem>>,
}

/// Profile 预览摘要（用于 UI 列表展示，避免传递完整 PrfItem）
pub struct IProfilePreview<'a> {
    /// 唯一标识符
    pub uid: &'a String,
    /// 显示名称
    pub name: &'a String,
    /// 是否为当前激活的配置
    pub is_current: bool,
}

/// 清理孤立的 Profile 文件的结果
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// 扫描的文件总数
    pub total_files: usize,
    /// 成功删除的文件列表
    pub deleted_files: Vec<String>,
    /// 删除失败的文件列表
    pub failed_deletions: Vec<String>,
}

// 将右值的 Option 字段 patch 到左值对应字段上（仅当右值字段为 Some 时覆盖）
macro_rules! patch {
    ($lv: expr, $rv: expr, $key: tt) => {
        if ($rv.$key).is_some() {
            $lv.$key = $rv.$key.to_owned();
        }
    };
}

impl IProfiles {
    // Helper to find and remove an item by uid from the items vec, returning its file name (if any).
    fn take_item_file_by_uid(items: &mut Vec<PrfItem>, target_uid: Option<String>) -> Option<String> {
        for (i, _) in items.iter().enumerate() {
            if items[i].uid == target_uid {
                return items.remove(i).file;
            }
        }
        None
    }
    pub async fn new() -> Self {
        let path = match dirs::profiles_path() {
            Ok(p) => p,
            Err(err) => {
                logging!(error, Type::Config, "{err}");
                return Self::default();
            }
        };
        match help::read_yaml::<Self>(&path).await {
            Ok(mut profiles) => {
                let items = profiles.items.get_or_insert(vec![]);
                for item in items.iter_mut() {
                    if item.uid.is_none() {
                        item.uid = Some(help::get_uid("d").into());
                    }
                }
                profiles
            }
            Err(err) => {
                logging!(error, Type::Config, "{err}");
                Self::default()
            }
        }
    }

    /// 将当前 IProfiles 持久化到 YAML 文件
    pub async fn save_file(&self) -> Result<()> {
        help::save_yaml(&dirs::profiles_path()?, self, Some("# Profiles Config for Clash Verge")).await
    }

    /// 用补丁更新 current 字段，仅当 patch.current 对应的 uid 在 items 中存在时才更新
    pub fn patch_config(&mut self, patch: &Self) {
        if self.items.is_none() {
            self.items = Some(vec![]);
        }

        if let Some(current) = &patch.current
            && let Some(items) = self.items.as_ref()
        {
            let some_uid = Some(current);
            if items.iter().any(|e| e.uid.as_ref() == some_uid) {
                self.current = some_uid.cloned();
            }
        }
    }

    /// 获取当前激活的 profile uid
    pub const fn get_current(&self) -> Option<&String> {
        self.current.as_ref()
    }

    /// 按 uid 查找 PrfItem，未找到时返回错误
    pub fn get_item(&self, uid: impl AsRef<str>) -> Result<&PrfItem> {
        let uid_str = uid.as_ref();

        if let Some(items) = self.items.as_ref() {
            for item in items.iter() {
                if let Some(uid_val) = &item.uid
                    && uid_val.as_str() == uid_str
                {
                    return Ok(item);
                }
            }
        }
        bail!("failed to get the profile item \"uid:{}\"", uid_str);
    }

    /// 追加一个新 item 到列表
    /// 如果 item 包含 file_data，先写入磁盘再保存元数据
    /// 如果当前没有激活配置，自动将新 item 设为 current
    pub async fn append_item(&mut self, item: &mut PrfItem) -> Result<()> {
        let uid = &item.uid;
        if uid.is_none() {
            bail!("the uid should not be null");
        }

        // save the file data
        // move the field value after save
        if let Some(file_data) = item.file_data.take() {
            // take() 是 Rust 中从 Option 取出值的标准惯用写法，省克隆、省内存、语义清晰
            if item.file.is_none() {
                bail!("the file should not be null");
            }

            let file = item
                .file
                .clone()
                .ok_or_else(|| anyhow::anyhow!("file field is required when file_data is provided"))?;
            let path = dirs::app_profiles_dir()?.join(file.as_str());

            tokio::fs::write(&path, file_data.as_bytes())
                .await
                .with_context(|| format!("failed to write to file \"{file}\""))?;
        }

        if self.current.is_none() && (item.itype == Some("remote".into()) || item.itype == Some("local".into())) {
            self.current = uid.to_owned();
        }

        if self.items.is_none() {
            self.items = Some(vec![]);
        }

        if let Some(items) = self.items.as_mut() {
            items.push(item.to_owned());
        }
        Ok(())
    }

    /// 拖拽排序：将 active_id 对应的 item 移动到 over_id 对应的位置
    pub async fn reorder(&mut self, active_id: &String, over_id: &String) -> Result<()> {
        let mut items = self.items.take().unwrap_or_default();

        let old_index = items.iter().position(|item| item.uid.as_ref() == Some(active_id));
        let new_index = items.iter().position(|item| item.uid.as_ref() == Some(over_id));

        let (old_idx, new_idx) = match (old_index, new_index) {
            (Some(old), Some(new)) => (old, new),
            _ => return Ok(()),
        };
        let item = items.remove(old_idx);
        items.insert(new_idx, item);
        self.items = Some(items);
        self.save_file().await
    }

    /// 部分更新：使用 patch 中的非空字段覆盖目标 item 的同名字段
    pub async fn patch_item(&mut self, uid: &String, patch: &PrfItem) -> Result<()> {
        let mut items = self.items.take().unwrap_or_default();

        for each in items.iter_mut() {
            if each.uid.as_ref() == Some(uid) {
                patch!(each, patch, name);
                patch!(each, patch, file);
                patch!(each, patch, itype);
                patch!(each, patch, url);
                patch!(each, patch, desc);
                patch!(each, patch, extra);
                patch!(each, patch, selected);
                patch!(each, patch, updated);
                patch!(each, patch, option);

                self.items = Some(items);
                return self.save_file().await;
            }
        }
        self.items = Some(items);
        bail!("failed to find the profile item \"uid:{uid}\"")
    }

    /// 更新远程订阅 item（仅修改 `updated`、`extra`、`file_data` 字段）
    /// 参数 `item` 的 file_data 在写入磁盘后被 take() 消费，避免重复写入
    pub async fn update_item(&mut self, uid: &String, item: &mut PrfItem) -> Result<()> {
        if self.items.is_none() {
            self.items = Some(vec![]);
        }

        // 先验证目标 uid 存在
        let _ = self.get_item(uid)?;

        if let Some(items) = self.items.as_mut() {
            let some_uid = Some(uid.clone());
            for each in items.iter_mut() {
                if each.uid == some_uid {
                    // 更新基本字段（clone，不消费 item）
                    each.extra = item.extra.clone();
                    each.updated = item.updated;
                    each.home = item.home.to_owned();
                    // 合并 option（以 each 为基础，item 的字段覆盖）
                    each.option = PrfOption::merge(each.option.as_ref(), item.option.as_ref());

                    // 若有新的 file_data，写入磁盘后更新 each.file
                    if let Some(file_data) = item.file_data.take() {
                        let file = each.file.take();
                        let file =
                            file.unwrap_or_else(|| item.file.take().unwrap_or_else(|| format!("{}.yaml", &uid).into()));

                        each.file = Some(file.clone());

                        let path = dirs::app_profiles_dir()?.join(file.as_str());

                        fs::write(&path, file_data.as_bytes())
                            .await
                            .with_context(|| format!("failed to write to file \"{file}\""))?;
                    }

                    break;
                }
            }
        }

        self.save_file().await
    }

    /// 删除指定 uid 的 item（及其所有关联扩展 item），并删除对应的磁盘文件
    /// 若删除的是当前激活项，自动选择下一个 remote/local 项作为 current
    pub async fn delete_item(&mut self, uid: &String) -> Result<bool> {
        let current = self.current.as_ref().unwrap_or(uid);
        let current = current.clone();
        let item = self.get_item(uid)?;
        let merge_uid = item.option.as_ref().and_then(|e| e.merge.clone());
        let script_uid = item.option.as_ref().and_then(|e| e.script.clone());
        let rules_uid = item.option.as_ref().and_then(|e| e.rules.clone());
        let proxies_uid = item.option.as_ref().and_then(|e| e.proxies.clone());
        let groups_uid = item.option.as_ref().and_then(|e| e.groups.clone());
        let mut items = self.items.take().unwrap_or_default();

        // remove the main item (if exists) and delete its file
        if let Some(file) = Self::take_item_file_by_uid(&mut items, Some(uid.clone())) {
            let _ = dirs::app_profiles_dir()?.join(file.as_str()).remove_if_exists().await;
        }

        // remove related extension items (merge, script, rules, proxies, groups)
        if let Some(file) = Self::take_item_file_by_uid(&mut items, merge_uid.clone()) {
            let _ = dirs::app_profiles_dir()?.join(file.as_str()).remove_if_exists().await;
        }
        if let Some(file) = Self::take_item_file_by_uid(&mut items, script_uid.clone()) {
            let _ = dirs::app_profiles_dir()?.join(file.as_str()).remove_if_exists().await;
        }
        if let Some(file) = Self::take_item_file_by_uid(&mut items, rules_uid.clone()) {
            let _ = dirs::app_profiles_dir()?.join(file.as_str()).remove_if_exists().await;
        }
        if let Some(file) = Self::take_item_file_by_uid(&mut items, proxies_uid.clone()) {
            let _ = dirs::app_profiles_dir()?.join(file.as_str()).remove_if_exists().await;
        }
        if let Some(file) = Self::take_item_file_by_uid(&mut items, groups_uid.clone()) {
            let _ = dirs::app_profiles_dir()?.join(file.as_str()).remove_if_exists().await;
        }
        // 如果删除的是当前激活项，自动切换到下一个可用项
        if current == *uid {
            self.current = None;
            for item in items.iter() {
                if item.itype == Some("remote".into()) || item.itype == Some("local".into()) {
                    self.current = item.uid.clone();
                    break;
                }
            }
        }

        self.items = Some(items);
        self.save_file().await?;
        Ok(current == *uid)
    }
    /// 获取current指向的订阅内容
    pub async fn current_mapping(&self) -> Result<Mapping> {
        match (self.current.as_ref(), self.items.as_ref()) {
            (Some(current), Some(items)) => {
                if let Some(item) = items.iter().find(|e| e.uid.as_ref() == Some(current)) {
                    let file_path = match item.file.as_ref() {
                        Some(file) => dirs::app_profiles_dir()?.join(file.as_str()),
                        None => bail!("failed to get the file field"),
                    };
                    return help::read_mapping(&file_path).await;
                }
                bail!("failed to find the current profile \"uid:{current}\"");
            }
            _ => Ok(Mapping::new()),
        }
    }

    /// 判断profile是否是current指向的
    pub fn is_current_profile_index(&self, index: &String) -> bool {
        self.current.as_ref() == Some(index)
    }

    /// 获取所有的profiles(uid，名称, 是否为 current)
    pub fn profiles_preview(&self) -> Option<Vec<IProfilePreview<'_>>> {
        self.items.as_ref().map(|items| {
            items
                .iter()
                .filter_map(|e| {
                    if let (Some(uid), Some(name)) = (e.uid.as_ref(), e.name.as_ref()) {
                        let is_current = self.is_current_profile_index(uid);
                        let preview = IProfilePreview { uid, name, is_current };
                        Some(preview)
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// 通过 uid 获取名称
    pub fn get_name_by_uid(&self, uid: &String) -> Option<&String> {
        if let Some(items) = &self.items {
            for item in items {
                if item.uid.as_ref() == Some(uid) {
                    return item.name.as_ref();
                }
            }
        }
        None
    }

    /// 以 app 中的 profile 列表为准，删除不再需要的文件
    pub async fn cleanup_orphaned_files(&self) -> Result<CleanupResult> {
        let profiles_dir = dirs::app_profiles_dir()?;

        if !profiles_dir.exists() {
            return Ok(CleanupResult {
                total_files: 0,
                deleted_files: vec![],
                failed_deletions: vec![],
            });
        }

        // 获取所有 active profile 的文件名集合
        let active_files = self.get_all_active_files();

        // 添加全局扩展配置文件到保护列表
        let protected_files = self.get_protected_global_files();

        // 扫描 profiles 目录下的所有文件
        let mut total_files = 0;
        let mut deleted_files = vec![];
        let mut failed_deletions = vec![];

        for entry in std::fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            total_files += 1;

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                && Self::is_profile_file(file_name)
            {
                // 检查是否为全局扩展文件
                if protected_files.contains(file_name) {
                    logging!(debug, Type::Config, "保护全局扩展配置文件: {file_name}");
                    continue;
                }

                // 检查是否为活跃文件
                if !active_files.contains(file_name) {
                    match path.to_path_buf().remove_if_exists().await {
                        Ok(_) => {
                            deleted_files.push(file_name.into());
                            logging!(debug, Type::Config, "已清理冗余文件: {file_name}");
                        }
                        Err(e) => {
                            failed_deletions.push(format!("{file_name}: {e}").into());
                            logging!(warn, Type::Config, "Warning: 清理文件失败: {file_name} - {e}");
                        }
                    }
                }
            }
        }

        let result = CleanupResult {
            total_files,
            deleted_files,
            failed_deletions,
        };

        logging!(
            info,
            Type::Config,
            "Profile 文件清理完成: 总文件数={}, 删除文件数={}, 失败数={}",
            result.total_files,
            result.deleted_files.len(),
            result.failed_deletions.len()
        );

        Ok(result)
    }

    /// 不删除全局扩展配置
    fn get_protected_global_files(&self) -> HashSet<String> {
        let mut protected_files = HashSet::new();

        protected_files.insert("Merge.yaml".into());
        protected_files.insert("Script.js".into());

        protected_files
    }

    /// 获取所有 active profile 关联的文件名
    fn get_all_active_files(&self) -> HashSet<&str> {
        let mut active_files: HashSet<&str> = HashSet::new();

        if let Some(items) = &self.items {
            for item in items {
                // 收集所有类型 profile 的文件
                if let Some(file) = &item.file {
                    active_files.insert(file);
                }

                // 对于主 profile 类型（remote/local），还需要收集其关联的扩展文件
                if let Some(itype) = &item.itype
                    && (itype == "remote" || itype == "local")
                    && let Some(option) = &item.option
                {
                    // 收集关联的扩展文件
                    if let Some(merge_uid) = &option.merge
                        && let Ok(merge_item) = self.get_item(merge_uid)
                        && let Some(file) = &merge_item.file
                    {
                        active_files.insert(file);
                    }

                    if let Some(script_uid) = &option.script
                        && let Ok(script_item) = self.get_item(script_uid)
                        && let Some(file) = &script_item.file
                    {
                        active_files.insert(file);
                    }

                    if let Some(rules_uid) = &option.rules
                        && let Ok(rules_item) = self.get_item(rules_uid)
                        && let Some(file) = &rules_item.file
                    {
                        active_files.insert(file);
                    }

                    if let Some(proxies_uid) = &option.proxies
                        && let Ok(proxies_item) = self.get_item(proxies_uid)
                        && let Some(file) = &proxies_item.file
                    {
                        active_files.insert(file);
                    }

                    if let Some(groups_uid) = &option.groups
                        && let Ok(groups_item) = self.get_item(groups_uid)
                        && let Some(file) = &groups_item.file
                    {
                        active_files.insert(file);
                    }
                }
            }
        }

        active_files
    }

    /// 检查文件名是否符合 profile 文件的命名规则
    fn is_profile_file(filename: &str) -> bool {
        // 匹配各种 profile 文件格式
        // R12345678.yaml (remote)
        // L12345678.yaml (local)
        // m12345678.yaml (merge)
        // s12345678.js (script)
        // r12345678.yaml (rules)
        // p12345678.yaml (proxies)
        // g12345678.yaml (groups)

        let patterns = [
            r"^[RL][a-zA-Z0-9]+\.yaml$",  // Remote/Local profiles
            r"^m[a-zA-Z0-9]+\.yaml$",     // Merge files
            r"^s[a-zA-Z0-9]+\.js$",       // Script files
            r"^[rpg][a-zA-Z0-9]+\.yaml$", // Rules/Proxies/Groups files
        ];

        patterns.iter().any(|pattern| {
            regex::Regex::new(pattern)
                .map(|re| re.is_match(filename))
                .unwrap_or(false)
        })
    }
}

// ====================================================================
// 线程安全的 Draft 包装函数
// 这些函数通过 Draft::with_data_modify 间接操作 IProfiles，
// 避免调用方持有内部锁跨越 .await 点（违反 Send 约束）。
// ====================================================================

/// 从已有 item 克隆并携带 file_data 创建新 item，然后追加到 profiles
pub async fn profiles_append_item_with_filedata_safe(item: &PrfItem, file_data: Option<String>) -> Result<()> {
    let item = &mut PrfItem::from(item, file_data).await?;
    profiles_append_item_safe(item).await
}

/// 通过 Draft 安全地追加 item，避免在 async 中持有内部锁
pub async fn profiles_append_item_safe(item: &mut PrfItem) -> Result<()> {
    Config::profiles()
        .await
        .with_data_modify(|mut profiles| async move {
            profiles.append_item(item).await?;
            Ok((profiles, ()))
        })
        .await
}

/// 通过 Draft 安全地部分更新 item
pub async fn profiles_patch_item_safe(index: &String, item: &PrfItem) -> Result<()> {
    Config::profiles()
        .await
        .with_data_modify(|mut profiles| async move {
            profiles.patch_item(index, item).await?;
            Ok((profiles, ()))
        })
        .await
}

/// 通过 Draft 安全地删除 item，返回是否同时删除了当前激活项
pub async fn profiles_delete_item_safe(index: &String) -> Result<bool> {
    Config::profiles()
        .await
        .with_data_modify(|mut profiles| async move {
            let deleted = profiles.delete_item(index).await?;
            Ok((profiles, deleted))
        })
        .await
}

/// 通过 Draft 安全地执行拖拽排序
pub async fn profiles_reorder_safe(active_id: &String, over_id: &String) -> Result<()> {
    Config::profiles()
        .await
        .with_data_modify(|mut profiles| async move {
            profiles.reorder(active_id, over_id).await?;
            Ok((profiles, ()))
        })
        .await
}

/// 通过 Draft 安全地持久化 profiles 到磁盘
pub async fn profiles_save_file_safe() -> Result<()> {
    Config::profiles()
        .await
        .with_data_modify(|profiles| async move {
            profiles.save_file().await?;
            Ok((profiles, ()))
        })
        .await
}

/// 通过 Draft 安全地更新远程订阅 item（更新 extra/updated/file_data）
pub async fn profiles_draft_update_item_safe(index: &String, item: &mut PrfItem) -> Result<()> {
    Config::profiles()
        .await
        .with_data_modify(|mut profiles| async move {
            profiles.update_item(index, item).await?;
            Ok((profiles, ()))
        })
        .await
}
