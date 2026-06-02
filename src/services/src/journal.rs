use crate::error::ServiceError;
use std::process::Command;

pub struct JournalService;

impl JournalService {
    pub fn fetch_logs(lines: u32, level: Option<&str>, filter: Option<&str>) -> Result<Vec<String>, ServiceError> {
        let mut cmd = Command::new("journalctl");
        cmd.args(["-u", "xray", "-n", &lines.to_string(), "--no-pager", "-o", "short-iso"]);

        if let Some(lvl) = level {
            if !lvl.is_empty() && lvl != "all" {
                cmd.args(["-p", &format!("{}..{}", lvl, "emerg")]);
            }
        }

        let output = cmd
            .output()
            .map_err(|e| ServiceError::Journal(format!("journalctl failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        let mut lines: Vec<String> = stdout.lines().map(String::from).collect();

        if let Some(f) = filter {
            if !f.is_empty() {
                lines.retain(|l| l.to_lowercase().contains(&f.to_lowercase()));
            }
        }

        Ok(lines)
    }
}
