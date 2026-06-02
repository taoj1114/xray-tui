use crate::error::ServiceError;
use xray_model::XrayStatus;

/// Placeholder systemd service that uses process commands for non-Linux / no-root scenarios.
/// Real D-Bus integration via zbus will replace this on Linux with root.
pub struct SystemdService {
    unit_name: String,
    xray_binary: String,
    config_path: String,
    unit_file_path: String,
}

impl SystemdService {
    pub fn new(xray_binary: String, config_path: String) -> Self {
        Self {
            unit_name: "xray".into(),
            xray_binary,
            config_path,
            unit_file_path: "/etc/systemd/system/xray.service".into(),
        }
    }

    pub fn get_status(&self) -> Result<XrayStatus, ServiceError> {
        let output = duct::cmd!("systemctl", "is-active", &self.unit_name)
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
            .map_err(|e| ServiceError::Systemd(format!("Failed to check status: {}", e)))?;

        let is_running = String::from_utf8_lossy(&output.stdout).trim() == "active";

        let (version, pid) = if is_running {
            self.get_version_and_pid().unwrap_or((None, None))
        } else {
            (None, None)
        };

        Ok(XrayStatus {
            is_installed: self.detect_installed(),
            is_running,
            version,
            pid,
            cpu_percent: None,
            memory_bytes: None,
            uptime_seconds: None,
        })
    }

    fn detect_installed(&self) -> bool {
        std::path::Path::new(&self.xray_binary).exists()
    }

    fn get_version_and_pid(&self) -> Result<(Option<String>, Option<u32>), ServiceError> {
        let version = self.get_version().ok();
        let pid = self.get_pid().ok();
        Ok((version, pid))
    }

    fn get_version(&self) -> Result<String, ServiceError> {
        let output = duct::cmd!(&self.xray_binary, "version")
            .stderr_to_stdout()
            .read()
            .map_err(|e| ServiceError::Systemd(format!("Failed to get version: {}", e)))?;
        Ok(output.lines().next().unwrap_or("unknown").to_string())
    }

    fn get_pid(&self) -> Result<u32, ServiceError> {
        let output = duct::cmd!("systemctl", "show", &self.unit_name, "--property=MainPID", "--value")
            .read()
            .map_err(|e| ServiceError::Systemd(format!("Failed to get PID: {}", e)))?;
        output.trim().parse::<u32>().map_err(|_| ServiceError::Systemd("Invalid PID".into()))
    }

    pub fn start(&self) -> Result<(), ServiceError> {
        duct::cmd!("systemctl", "start", &self.unit_name)
            .run()
            .map_err(|e| ServiceError::Systemd(format!("Failed to start: {}", e)))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), ServiceError> {
        duct::cmd!("systemctl", "stop", &self.unit_name)
            .run()
            .map_err(|e| ServiceError::Systemd(format!("Failed to stop: {}", e)))?;
        Ok(())
    }

    pub fn restart(&self) -> Result<(), ServiceError> {
        duct::cmd!("systemctl", "restart", &self.unit_name)
            .run()
            .map_err(|e| ServiceError::Systemd(format!("Failed to restart: {}", e)))?;
        Ok(())
    }

    pub fn install_unit_file(&self) -> Result<(), ServiceError> {
        if std::path::Path::new(&self.unit_file_path).exists() {
            return Ok(());
        }

        let unit_content = format!(
            "[Unit]\n\
             Description=Xray Service\n\
             After=network.target\n\n\
             [Service]\n\
             Type=simple\n\
             ExecStart={} run -config {}\n\
             Restart=on-failure\n\
             RestartSec=5\n\n\
             [Install]\n\
             WantedBy=multi-user.target\n",
            self.xray_binary, self.config_path
        );

        let dir = std::path::Path::new(&self.unit_file_path).parent().unwrap();
        std::fs::create_dir_all(dir)?;
        std::fs::write(&self.unit_file_path, unit_content)?;

        duct::cmd!("systemctl", "daemon-reload")
            .run()
            .map_err(|e| ServiceError::Systemd(format!("Failed to reload: {}", e)))?;

        duct::cmd!("systemctl", "enable", &self.unit_name)
            .run()
            .map_err(|e| ServiceError::Systemd(format!("Failed to enable: {}", e)))?;

        Ok(())
    }
}
