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

        let (version, pid, cpu_pct, mem_bytes, uptime_sec) = if is_running {
            let vp = self.get_version_and_pid().unwrap_or((None, None));
            let res = vp.1.and_then(|p| self.get_process_stats(p)).unwrap_or((None, None, None));
            (vp.0, vp.1, res.0, res.1, res.2)
        } else {
            (None, None, None, None, None)
        };

        Ok(XrayStatus {
            is_installed: self.detect_installed(),
            is_running,
            version,
            pid,
            cpu_percent: cpu_pct,
            memory_bytes: mem_bytes,
            uptime_seconds: uptime_sec,
        })
    }

    fn get_process_stats(&self, pid: u32) -> Option<(Option<f64>, Option<u64>, Option<u64>)> {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
        let fields: Vec<&str> = stat.split_whitespace().collect();
        // field 13: utime, field 14: stime (jiffies), field 21: starttime
        let utime: u64 = fields.get(13)?.parse().ok()?;
        let stime: u64 = fields.get(14)?.parse().ok()?;
        let starttime: u64 = fields.get(21)?.parse().ok()?;
        let cpu_total = utime + stime;
        let clk_tck = 100u64; // sysconf(_SC_CLK_TCK) typically 100
        let uptime = std::fs::read_to_string("/proc/uptime").ok()?;
        let sys_uptime: f64 = uptime.split_whitespace().next()?.parse().ok()?;
        let elapsed = sys_uptime as u64 - starttime / clk_tck;
        let cpu_pct = if elapsed > 0 { Some((cpu_total as f64 * 100.0) / (elapsed as f64 * clk_tck as f64)) } else { None };

        // RSS from /proc/{pid}/statm field 1 (pages)
        let statm = std::fs::read_to_string(format!("/proc/{}/statm", pid)).ok()?;
        let rss_pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
        let mem = Some(rss_pages * 4096);

        let uptime_secs = if elapsed > 0 { Some(elapsed) } else { None };
        Some((cpu_pct, mem, uptime_secs))
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
        let unit_content = format!(
            "[Unit]\n\
             Description=Xray Service\n\
             After=network.target\n\n\
             [Service]\n\
             Type=simple\n\
             User=root\n\
             ExecStart={} run -config {}\n\
             Restart=on-failure\n\
             RestartSec=5\n\n\
             [Install]\n\
             WantedBy=multi-user.target\n",
            self.xray_binary, self.config_path
        );

        // Write unit file via sudo tee
        let mut child = std::process::Command::new("sudo")
            .args(["tee", &self.unit_file_path])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .map_err(|e| ServiceError::Systemd(format!("Failed to spawn sudo tee for unit file: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(unit_content.as_bytes()).map_err(|e| ServiceError::Io(e))?;
        }
        let status = child.wait().map_err(|e| ServiceError::Io(e))?;
        if !status.success() {
            return Err(ServiceError::Systemd(format!("Failed to write unit file via sudo, exit code {}", status)));
        }

        let _ = std::process::Command::new("sudo").args(["systemctl", "daemon-reload"]).output();
        let _ = std::process::Command::new("sudo").args(["systemctl", "enable", &self.unit_name]).output();

        Ok(())
    }
}
