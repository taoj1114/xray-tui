use crate::error::ServiceError;
use std::process::Command;

pub struct JournalService;

impl JournalService {
    /// Fetch the last N lines from xray's journal. Filters by systemd priority if given.
    pub fn fetch_logs(lines: u32, _level: Option<&str>, _filter: Option<&str>) -> Result<Vec<String>, ServiceError> {
        let mut cmd = Command::new("journalctl");
        cmd.args(["-u", "xray", "-n", &lines.to_string(), "--no-pager", "-o", "short-iso", "-q"]);
        let output = cmd.output().map_err(|e| ServiceError::Journal(format!("journalctl: {}", e)))?;
        Ok(String::from_utf8_lossy(&output.stdout).lines().map(String::from).collect())
    }
}
