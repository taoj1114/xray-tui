use crate::error::ServiceError;
use xray_model::InboundConfig;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    config_dir: PathBuf,
}

use std::time::SystemTime;

pub struct ConfigEntry {
    pub config: InboundConfig,
    pub filename: String,
    pub enabled: bool,
    pub modified: SystemTime,
}

impl ConfigManager {
    fn make_filename(inbound: &InboundConfig) -> String {
        let proto = inbound.protocol.to_string().to_lowercase();
        let net = match inbound.stream_settings.network {
            xray_model::TransportNetwork::Tcp => "tcp",
            xray_model::TransportNetwork::Ws => "ws",
            xray_model::TransportNetwork::Grpc => "grpc",
            xray_model::TransportNetwork::HttpUpgrade => "hup",
            xray_model::TransportNetwork::Quic => "quic",
            xray_model::TransportNetwork::Kcp => "kcp",
            xray_model::TransportNetwork::DomainSocket => "ds",
        };
        let sec = match inbound.stream_settings.security {
            xray_model::StreamSecurity::None => "none",
            xray_model::StreamSecurity::Tls => "tls",
            xray_model::StreamSecurity::Reality => "reality",
        };
        format!("{}+{}+{}+{}.json", proto, net, sec, inbound.port)
    }
    pub fn new(config_dir: &str) -> Self {
        let path = PathBuf::from(config_dir);
        let _ = fs::create_dir_all(&path);
        Self { config_dir: path }
    }

    /// Load all configs, tracking enabled status via extension, sorted by modification time.
    pub fn load_configs(&self) -> Result<Vec<ConfigEntry>, ServiceError> {
        let mut entries = Vec::new();
        if !self.config_dir.exists() { return Ok(entries); }

        for entry in fs::read_dir(&self.config_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() { continue; }
            
            let metadata = entry.metadata()?;
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            let is_json = name.ends_with(".json");
            let is_disabled = name.ends_with(".json.disabled");

            if is_json || is_disabled {
                let content = fs::read_to_string(&path)?;
                if let Ok(config) = serde_json::from_str::<InboundConfig>(&content) {
                    entries.push(ConfigEntry {
                        config,
                        filename: name.to_string(),
                        enabled: is_json,
                        modified,
                    });
                }
            }
        }
        
        // Sort by modification time (newest last, or oldest last?)
        // Usually, users want newest entries at the bottom or top. 
        // Let's go with ascending order (oldest first, newest at the end) so they appear at the bottom of the list.
        entries.sort_by_key(|e| e.modified);
        
        Ok(entries)
    }

    /// Save inbound with an auto-generated descriptive filename.
    /// Format: `{protocol}+{network}+{security}+{port}.json`
    pub fn save_inbound(&self, inbound: &InboundConfig, enabled: bool) -> Result<(), ServiceError> {
        let base_name = Self::make_filename(inbound);
        let filename = if enabled { base_name.clone() } else { format!("{}.disabled", base_name) };
        let path = self.config_dir.join(&filename);
        
        // Remove potentially conflicting file
        let alt_name = if enabled { format!("{}.disabled", filename) } else { filename.strip_suffix(".disabled").unwrap().to_string() };
        let _ = std::process::Command::new("sudo").args(["rm", "-f", self.config_dir.join(alt_name).to_str().unwrap()]).output();

        let json = serde_json::to_string_pretty(inbound)?;
        
        // Try writing directly
        if let Err(_) = fs::write(&path, &json) {
            // Fallback to sudo tee
            let mut child = std::process::Command::new("sudo")
                .args(["tee", path.to_str().unwrap()])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .spawn()
                .map_err(|e| ServiceError::Storage(format!("Failed to spawn sudo tee: {}", e)))?;

            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(json.as_bytes()).map_err(ServiceError::from)?;
            }
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn toggle_enabled(&self, filename: &String) -> Result<String, ServiceError> {
        let old_path = self.config_dir.join(filename);
        let (new_name, _enabled) = if filename.ends_with(".disabled") {
            (filename.strip_suffix(".disabled").unwrap().to_string(), true)
        } else {
            (format!("{}.disabled", filename), false)
        };
        let new_path = self.config_dir.join(&new_name);
        
        if let Err(_) = fs::rename(&old_path, &new_path) {
            // Fallback to sudo mv
            let status = std::process::Command::new("sudo")
                .args(["mv", old_path.to_str().unwrap(), new_path.to_str().unwrap()])
                .status()
                .map_err(ServiceError::from)?;
            if !status.success() {
                return Err(ServiceError::Storage(format!("sudo mv failed with {}", status)));
            }
        }
        Ok(new_name)
    }

    pub fn delete_config(&self, filename: &str) -> Result<(), ServiceError> {
        let path = self.config_dir.join(filename);
        if let Err(_) = fs::remove_file(&path) {
            if path.exists() {
                let _ = std::process::Command::new("sudo").args(["rm", path.to_str().unwrap()]).output();
            }
        }
        Ok(())
    }
}
