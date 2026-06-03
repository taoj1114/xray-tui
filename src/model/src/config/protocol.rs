use serde::{Deserialize, Serialize};
use super::FallbackConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtocolSettings {
    VMess(VMessSettings),
    VLess(VLessSettings),
    Trojan(TrojanSettings),
    Shadowsocks(ShadowsocksSettings),
    Http(HttpSettings),
    Socks(SocksSettings),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VMessSettings {
    pub clients: Vec<VMessClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMessClient {
    pub id: String,
    #[serde(default = "default_vmess_security")]
    pub security: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
fn default_vmess_security() -> String { "auto".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VLessSettings {
    pub clients: Vec<VLessClient>,
    #[serde(default = "default_decryption")]
    pub decryption: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLessClient {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
fn default_decryption() -> String { "none".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrojanSettings {
    pub clients: Vec<TrojanClient>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub fallbacks: Vec<FallbackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrojanClient {
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadowsocksSettings {
    pub method: String,
    pub password: String,
    #[serde(default = "default_ss_network")]
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
fn default_ss_network() -> String { "tcp,udp".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpSettings {
    #[serde(default)]
    pub accounts: Vec<HttpAccount>,
    #[serde(default = "default_http_timeout")]
    pub timeout: u32,
    #[serde(default)]
    pub allow_transparent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAccount {
    pub user: String,
    pub pass: String,
}
fn default_http_timeout() -> u32 { 300 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SocksSettings {
    pub auth: SocksAuth,
    #[serde(default)]
    pub udp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(default)]
    pub user_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SocksAuth {
    NoAuth {},
    Password { accounts: Vec<SocksAccount> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocksAccount {
    pub user: String,
    pub pass: String,
}
