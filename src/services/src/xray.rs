use xray_model::{XrayConfig, InboundConfig, RoutingConfig, GlobalSettings};
use crate::error::ServiceError;

pub struct XrayService {
    settings: GlobalSettings,
}

impl XrayService {
    pub fn new(settings: GlobalSettings) -> Self {
        Self { settings }
    }

    pub fn generate_config(
        &self,
        inbounds: &[InboundConfig],
        routing: &RoutingConfig,
    ) -> XrayConfig {
        XrayConfig::from_inbounds(inbounds, routing)
    }

    pub fn write_config(&self, config: &XrayConfig) -> Result<(), ServiceError> {
        let json = serde_json::to_string_pretty(config)?;

        let config_path = &self.settings.config_path;
        if std::path::Path::new(config_path).exists() {
            let _ = std::fs::copy(config_path, format!("{}.bak", config_path));
        }

        if let Some(parent) = std::path::Path::new(config_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(config_path, json.as_bytes())?;
        Ok(())
    }

    pub fn validate_config(&self, config: &XrayConfig) -> Result<(), ServiceError> {
        for inbound in &config.inbounds {
            if inbound.listen.is_empty() {
                return Err(ServiceError::Command("listen address is empty".into()));
            }
            if inbound.port == 0 {
                return Err(ServiceError::Command("port cannot be 0".into()));
            }
        }
        Ok(())
    }

    pub fn generate_reality_keys(&self) -> Result<(String, String), ServiceError> {
        let output = duct::cmd!(&self.settings.xray_binary_path, "x25519")
            .stderr_to_stdout()
            .read()
            .map_err(|e| ServiceError::Command(format!("Failed to run xray x25519: {}", e)))?;

        let mut private_key = String::new();
        let mut public_key = String::new();

        for line in output.lines() {
            if let Some(key) = line.strip_prefix("Private key: ") {
                private_key = key.trim().to_string();
            }
            if let Some(key) = line.strip_prefix("Public key: ") {
                public_key = key.trim().to_string();
            }
        }

        if private_key.is_empty() || public_key.is_empty() {
            return Err(ServiceError::Command(
                format!("Failed to parse x25519 output:\n{}", output)
            ));
        }

        Ok((private_key, public_key))
    }

    pub fn detect_xray(&self) -> (bool, Option<String>) {
        match duct::cmd!(&self.settings.xray_binary_path, "version")
            .stderr_to_stdout()
            .read()
        {
            Ok(output) => {
                let version = output.lines().next().map(|s| s.to_string());
                (true, version)
            }
            Err(_) => (false, None),
        }
    }

    /// 使用官方安装脚本安装 Xray（需要 root，非 root 自动 sudo）
    pub fn install_xray(&self) -> Result<(), ServiceError> {
        let is_root = std::env::var("USER").map(|u| u == "root").unwrap_or(false);
        let mut cmd = if is_root {
            let mut c = std::process::Command::new("bash");
            c.args(["-c", "bash <(curl -Ls https://github.com/XTLS/Xray-install/raw/main/install-release.sh)"]);
            c
        } else {
            let mut c = std::process::Command::new("sudo");
            c.args(["bash", "-c", "bash <(curl -Ls https://github.com/XTLS/Xray-install/raw/main/install-release.sh)"]);
            c
        };
        let output = cmd.output()
            .map_err(|e| ServiceError::Command(format!("Xray install failed: {}", e)))?;
        if !output.status.success() {
            return Err(ServiceError::Command(format!(
                "Xray install script failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Check if binary now exists
        if !std::path::Path::new(&self.settings.xray_binary_path).exists() {
            return Err(ServiceError::Command(format!(
                "Install completed but binary not found at {}. Check output:\n{}",
                self.settings.xray_binary_path,
                String::from_utf8_lossy(&output.stdout)
            )));
        }
        Ok(())
    }

    /// Uninstall Xray — stop service, remove binary, config, and systemd unit
    pub fn uninstall_xray(&self) -> Result<(), ServiceError> {
        let binary = &self.settings.xray_binary_path;
        let config = &self.settings.config_path;
        // Stop & disable systemd unit first
        let _ = std::process::Command::new("sudo").args(["systemctl", "stop", "xray"]).output();
        let _ = std::process::Command::new("sudo").args(["systemctl", "disable", "xray"]).output();
        // Remove unit file
        let _ = std::process::Command::new("sudo").args(["rm", "-f", "/etc/systemd/system/xray.service"]).output();
        let _ = std::process::Command::new("sudo").args(["systemctl", "daemon-reload"]).output();
        // Remove binary
        let _ = std::process::Command::new("sudo").args(["rm", "-f", binary]).output();
        // Remove config + backup
        let _ = std::process::Command::new("sudo").args(["rm", "-f", config]).output();
        let _ = std::process::Command::new("sudo").args(["rm", "-f", &format!("{}.bak", config)]).output();
        Ok(())
    }

    pub fn settings(&self) -> &GlobalSettings {
        &self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xray_model::{InboundProtocol, VLessSettings, VLessClient, TransportNetwork, StreamSecurity, StreamSettings, WsSettings, SniffingConfig, ProtocolSettings};

    #[test]
    fn test_generate_and_validate() {
        let settings = GlobalSettings::default();
        let svc = XrayService::new(settings);

        let inbound = InboundConfig {
            tag: Some("test".into()),
            port: 443,
            listen: "0.0.0.0".into(),
            protocol: InboundProtocol::VLess,
            settings: ProtocolSettings::VLess(VLessSettings {
                clients: vec![VLessClient {
                    id: uuid::Uuid::new_v4().to_string(),
                    flow: None,
                    email: None,
                    level: None,
                }],
                decryption: "none".into(),
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Tcp,
                security: StreamSecurity::None,
                tls_settings: None,
                reality_settings: None,
                ws_settings: None,
                grpc_settings: None,
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig::default(),
        };

        let routing = RoutingConfig::default();
        let config = svc.generate_config(&[inbound], &routing);
        assert_eq!(config.inbounds.len(), 1);
        assert!(svc.validate_config(&config).is_ok());

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("test"));
    }

    #[test]
    fn test_validate_rejects_empty_listen() {
        let settings = GlobalSettings::default();
        let svc = XrayService::new(settings);
        let mut config = XrayConfig::from_inbounds(&[], &RoutingConfig::default());
        let inbound = InboundConfig {
            tag: None,
            port: 8080,
            listen: String::new(),
            protocol: InboundProtocol::Socks,
            settings: ProtocolSettings::Socks(xray_model::SocksSettings {
                auth: xray_model::SocksAuth::NoAuth {},
                udp: true,
                ip: None,
                user_level: 0,
            }),
            stream_settings: StreamSettings {
                network: TransportNetwork::Tcp,
                security: StreamSecurity::None,
                tls_settings: None,
                reality_settings: None,
                ws_settings: None,
                grpc_settings: None,
                httpupgrade_settings: None,
                tcp_settings: None,
                kcp_settings: None,
                quic_settings: None,
            },
            sniffing: SniffingConfig::default(),
        };
        config.inbounds.push(inbound);
        assert!(svc.validate_config(&config).is_err());
    }
}
