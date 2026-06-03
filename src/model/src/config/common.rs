use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_loglevel")]
    pub loglevel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
fn default_loglevel() -> String { "warning".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub tag: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<HashMap<String, PolicyLevel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<PolicySystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLevel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "connIdle", alias = "conn_idle")]
    pub conn_idle: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uplinkOnly", alias = "uplink_only")]
    pub uplink_only: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "downlinkOnly", alias = "downlink_only")]
    pub downlink_only: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statsUserUplink", alias = "stats_user_uplink")]
    pub stats_user_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statsUserDownlink", alias = "stats_user_downlink")]
    pub stats_user_downlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bufferSize", alias = "buffer_size")]
    pub buffer_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySystem {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statsInboundUplink", alias = "stats_inbound_uplink")]
    pub stats_inbound_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statsInboundDownlink", alias = "stats_inbound_downlink")]
    pub stats_inbound_downlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statsOutboundUplink", alias = "stats_outbound_uplink")]
    pub stats_outbound_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statsOutboundDownlink", alias = "stats_outbound_downlink")]
    pub stats_outbound_downlink: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniffingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_sniff_dest_override")]
    #[serde(rename = "destOverride", alias = "dest_override")]
    pub dest_override: Vec<String>,
    #[serde(default)]
    #[serde(rename = "routeOnly", alias = "route_only")]
    pub route_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "metadataOnly", alias = "metadata_only")]
    pub metadata_only: Option<bool>,
}
fn default_true() -> bool { true }
fn default_sniff_dest_override() -> Vec<String> { vec!["http".into(), "tls".into()] }

impl Default for SniffingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dest_override: default_sniff_dest_override(),
            route_only: false,
            metadata_only: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}
