use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    #[serde(default = "default_domain_strategy")]
    pub domain_strategy: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}
fn default_domain_strategy() -> String { "AsIs".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<Vec<String>>,
    pub outbound_tag: String,
    #[serde(default)]
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Vec<String>>,
}

impl RoutingRule {
    pub fn new_direct_domain(domains: Vec<String>) -> Self {
        Self {
            domain: Some(domains),
            outbound_tag: "direct".into(),
            type_: "field".into(),
            ..Default::default()
        }
    }

    pub fn new_block_domain(domains: Vec<String>) -> Self {
        Self {
            domain: Some(domains),
            outbound_tag: "block".into(),
            type_: "field".into(),
            ..Default::default()
        }
    }

    pub fn new_direct_ip(ips: Vec<String>) -> Self {
        Self {
            ip: Some(ips),
            outbound_tag: "direct".into(),
            type_: "field".into(),
            ..Default::default()
        }
    }

    pub fn preset_cn_direct() -> Self {
        Self::new_direct_domain(vec!["geosite:cn".into()])
    }

    pub fn preset_ads_block() -> Self {
        Self::new_block_domain(vec!["geosite:category-ads-all".into()])
    }

    pub fn preset_cn_ip_direct() -> Self {
        Self::new_direct_ip(vec!["geoip:cn".into()])
    }

    pub fn preset_private_direct() -> Self {
        Self::new_direct_ip(vec!["geoip:private".into()])
    }

    pub fn all_presets() -> Vec<Self> {
        vec![
            Self::preset_cn_direct(),
            Self::preset_ads_block(),
            Self::preset_cn_ip_direct(),
            Self::preset_private_direct(),
        ]
    }
}

impl Default for RoutingRule {
    fn default() -> Self {
        Self {
            domain: None,
            ip: None,
            port: None,
            source: None,
            source_port: None,
            network: None,
            protocol: None,
            inbound_tag: None,
            outbound_tag: "direct".into(),
            type_: "field".into(),
            user: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_rule_serialization() {
        let rule = RoutingRule::preset_cn_direct();
        let json = serde_json::to_string(&rule).unwrap();
        eprintln!("JSON: {}", json);
        assert!(json.contains("outboundTag"));
        assert!(json.contains("geosite:cn"));
    }

    #[test]
    fn test_routing_config_serialization() {
        let config = RoutingConfig {
            domain_strategy: "IPIfNonMatch".into(),
            rules: RoutingRule::all_presets(),
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"domainStrategy\": \"IPIfNonMatch\""));
        assert!(json.contains("geosite:category-ads-all"));
    }
}
