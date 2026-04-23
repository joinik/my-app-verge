use std::collections::{HashMap, HashSet};

use serde_yaml_ng::{Mapping, Value};

const PATCH_CONFIG_INNER: [&str; 4] = ["allow-lan", "ipv6", "unified-delay", "log-level"];

#[derive(Default, Clone)]
pub struct IRuntime {
    pub config: Option<Mapping>,
    // 记录在订阅中（包括merge和script生成的）出现过的keys
    // 这些keys不一定都生效
    pub exists_keys: HashSet<String>,
    // TODO 或许可以用 FixMap 来存储以提升效率
    pub chain_logs: HashMap<String, Vec<(String, String)>>,
}

impl IRuntime {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // 这里只更改 allow-lan | ipv6 | log-level | tun
    #[inline]
    pub fn patch_config(&mut self, patch: &Mapping) {
        let config = if let Some(config) = self.config.as_mut() {
            config
        } else {
            return;
        };

        for key in PATCH_CONFIG_INNER.iter() {
            if let Some(value) = patch.get(key) {
                config.insert((*key).into(), value.clone());
            }
        }

        let patch_tun = patch.get("tun");
        if let Some(patch_tun_value) = patch_tun {
            let mut tun = config
                .get("tun")
                .and_then(|val| val.as_mapping())
                .cloned()
                .unwrap_or_else(Mapping::new);

            if let Some(patch_tun_mapping) = patch_tun_value.as_mapping() {
                for key in use_keys(patch_tun_mapping) {
                    if let Some(value) = patch_tun_mapping.get(key.as_str()) {
                        tun.insert(Value::from(key.as_str()), value.clone());
                    }
                }
            }

            config.insert("tun".into(), tun.into());
            // config.insert("tun".into(), Value::from(tun));
        }
    }
}

// TODO 完整迁移 enhance 行为后移除
#[inline]
fn use_keys<'a>(config: &'a Mapping) -> impl Iterator<Item = String> + 'a {
    config.iter().filter_map(|(key, _)| key.as_str()).map(|s: &str| {
        let mut s: String = s.into();
        s.make_ascii_lowercase();
        s
    })
}
