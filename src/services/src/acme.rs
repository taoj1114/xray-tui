use crate::error::ServiceError;
use chrono::NaiveDate;
use std::process::Command;
use xray_model::CertInfo;

pub struct AcmeService;

impl AcmeService {
    pub fn is_installed() -> bool {
        Command::new("which").arg("acme.sh").output().is_ok()
    }

    fn run_acme(args: &[&str]) -> Result<String, ServiceError> {
        let output = Command::new("acme.sh")
            .args(args)
            .output()
            .map_err(|e| ServiceError::Acme(format!("Failed to run acme.sh: {}", e)))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}{}", stdout, stderr);
        if !output.status.success() {
            return Err(ServiceError::Acme(format!("acme.sh failed:\n{}", combined)));
        }
        Ok(combined)
    }

    pub fn issue_cert(
        domain: &str,
        method: &str,
        webroot: Option<&str>,
        cf_email: Option<&str>,
        cf_key: Option<&str>,
    ) -> Result<(), ServiceError> {
        let mut args = vec!["--issue", "-d", domain];

        match method {
            "webroot" => {
                args.push("--webroot");
                args.push(webroot.unwrap_or("/var/www/html"));
            }
            "alpn" => {
                args.push("--alpn");
            }
            "dns_cf" => {
                args.push("--dns");
                args.push("dns_cf");
                // Export CF credentials as env vars for acme.sh
                if let Some(email) = cf_email {
                    std::env::set_var("CF_Email", email);
                }
                if let Some(key) = cf_key {
                    std::env::set_var("CF_Key", key);
                }
            }
            _ => return Err(ServiceError::Acme(format!("Unknown method: {}", method))),
        }

        let output = Self::run_acme(&args)?;
        if output.contains("Certificate success") || output.contains("Cert success") || output.contains("Domains not changed") {
            Ok(())
        } else {
            Err(ServiceError::Acme(format!("Cert issue failed:\n{}", output)))
        }
    }

    pub fn renew_cert(domain: &str) -> Result<(), ServiceError> {
        let output = Self::run_acme(&["--renew", "-d", domain])?;
        if output.contains("success") {
            Ok(())
        } else {
            Err(ServiceError::Acme(format!("Cert renewal failed:\n{}", output)))
        }
    }

    pub fn list_certs() -> Result<Vec<CertInfo>, ServiceError> {
        let output = Self::run_acme(&["--list"])?;
        Self::parse_cert_list(&output)
    }

    fn parse_cert_list(output: &str) -> Result<Vec<CertInfo>, ServiceError> {
        let mut certs = Vec::new();
        let mut started = false;
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());

        for line in output.lines() {
            if line.contains("Main_Domain") { started = true; continue; }
            if !started { continue; }
            if line.trim().is_empty() { break; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }
            let domain = parts[0].trim_matches('*').trim_matches('.').to_string();
            let acme_dir = format!("{}/.acme.sh/{}", home, domain);
            let cert_path = format!("{}/fullchain.cer", acme_dir);
            let key_path = format!("{}/{}.key", acme_dir, domain);
            let issued_at = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
            let expires_at = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
            certs.push(CertInfo {
                domain, issued_at, expires_at, cert_path, key_path,
                issuer: "Let's Encrypt".into(), auto_renew: false, renew_command: None,
            });
        }
        Ok(certs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_cert_list() {
        let output = "Main_Domain  KeyLength  SAN_Domains  Created  RenewDate\n\
                      example.com  2048  *.example.com  /root/.acme.sh/example.com  2025-01-01  2025-04-01";
        let certs = AcmeService::parse_cert_list(output).unwrap();
        assert!(!certs.is_empty());
        assert_eq!(certs[0].domain, "example.com");
    }
}
