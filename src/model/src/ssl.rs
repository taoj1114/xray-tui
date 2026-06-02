use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub domain: String,
    pub issued_at: NaiveDate,
    pub expires_at: NaiveDate,
    pub cert_path: String,
    pub key_path: String,
    pub issuer: String,
    pub auto_renew: bool,
    pub renew_command: Option<String>,
}
