use serde::{Deserialize, Serialize};

/// 代理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
    Socks5T,
    Trojan,
    Vless,
    Vmess,
    Snell,
    Obfs,
}
/// 单个代理配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct ProxyItem {
    /// 代理名称
    pub name: String,
    /// 代理类型
    #[serde(rename = "type")]
    pub type_: ProxyType,
    /// 服务器地址
    pub server: String,
    /// 服务器端口
    pub port: u16,
    /// 密码/密钥
    pub password: String,
    /// 用户名
    pub username: Option<String>,
    /// 加密方式
    pub cipher: Option<String>,
    /// SNI 主机名
    pub sni: Option<String>,
    /// 是否跳过证书验证
    pub skip_cert_verify: Option<bool>,
    /// 是否开启 UDP 转发
    pub udp: Option<bool>,
    /// 插件配置
    pub plugin: Option<String>,
    /// 插件参数
    pub plugin_opts: Option<String>,
}

/// 路由规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    IpCidr,
    GeoIP,
    Classical,
    IPSuffix,
}

/// 单个路由规则配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct RuleItem {
    /// 路由规则类型
    #[serde(rename = "type")]
    pub type_: RuleType,
    /// 路由规则值
    pub content: String,
    /// 代理名称
    pub proxy: String,
    /// 路由规则分组
    pub group: Option<String>,
    /// 是否排除路由规则
    pub excluded: Option<bool>,
}

/// 规则集
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct RuleSet {
    pub name: String,
    pub type_: String,
    pub format: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
}

/// 代理分组类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ProxyGroupType {
    Select,
    UrlTest,
    Fallback,
    LoadBalance,
    Relay,
}

/// 代理分组配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct ProxyGroup {
    pub name: String,
    /// 代理分组类型
    #[serde(rename = "type")]
    pub type_: ProxyGroupType,
    /// 代理分组下的代理列表
    pub proxies: Vec<String>,
    /// 测试 URL
    pub url: Option<String>,
    /// 测试间隔（秒）
    pub interval: Option<u64>,
    /// 测试超时（秒）
    pub tolerance: Option<u64>,
    /// 是否包含原始代理
    pub include_original: Option<bool>,
}
