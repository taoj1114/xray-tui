use crate::error::ServiceError;
use chrono::NaiveDate;
use std::process::Command;
use xray_model::CertInfo;

pub struct AcmeService;

impl AcmeService {
    pub fn is_installed() -> bool {
        let path = Self::find_acme_sh();
        if path == "acme.sh" {
            // fallback check via which
            Command::new("which").arg("acme.sh").output().map(|o| o.status.success()).unwrap_or(false)
        } else {
            std::path::Path::new(&path).exists()
        }
    }

    /// Auto-install acme.sh via the official installer script, and register account if no account exists.
    pub fn install_acme(cf_email: Option<&str>) -> Result<(), ServiceError> {
        if Self::is_installed() { return Ok(()); }

        // Check for dependencies
        let has_curl = Command::new("which").arg("curl").output().map(|o| o.status.success()).unwrap_or(false);
        let has_wget = Command::new("which").arg("wget").output().map(|o| o.status.success()).unwrap_or(false);

        if !has_curl && !has_wget {
            return Err(ServiceError::Acme("Either 'curl' or 'wget' is required to install acme.sh. Please install one first.".into()));
        }

        // Run the official installer
        let cmd_str = if has_curl {
            "curl https://get.acme.sh | sh"
        } else {
            "wget -O - https://get.acme.sh | sh"
        };

        let output = Command::new("bash")
            .args(["-c", cmd_str])
            .output()
            .map_err(|e| ServiceError::Acme(format!("Failed to execute install command: {}", e)))?;

        if !output.status.success() {
            return Err(ServiceError::Acme(format!(
                "acme.sh install script failed (exit code {}):\nSTDOUT: {}\nSTDERR: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Register account
        let acme_sh = Self::find_acme_sh();
        if acme_sh == "acme.sh" && !Self::is_installed() {
             return Err(ServiceError::Acme("Installation seemed to succeed but acme.sh binary could not be located.".into()));
        }

        let mut register_args = vec!["--register-account"];
        if let Some(email) = cf_email {
            register_args.push("-m");
            register_args.push(email);
        } else {
            register_args.push("-m");
            register_args.push("my@example.com"); // acme.sh requires an email now
        }
        
        let reg_output = Command::new(&acme_sh).args(&register_args).output();
        if let Ok(out) = reg_output {
            if !out.status.success() {
                // Not a fatal error for the installation itself, but good to know
                eprintln!("Account registration warning: {}", String::from_utf8_lossy(&out.stderr));
            }
        }

        Ok(())
    }

    fn find_acme_sh() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        for path in &[
            "/root/.acme.sh/acme.sh",
            &format!("{}/.acme.sh/acme.sh", home),
            &format!("{}/.acme.sh/acme.sh", home.trim_end_matches('/')),
            "/usr/local/bin/acme.sh",
            "/usr/bin/acme.sh",
            "~/.acme.sh/acme.sh",
        ] {
            let p = if path.starts_with("~") {
                path.replace("~", &home)
            } else {
                path.to_string()
            };
            if std::path::Path::new(&p).exists() { return p; }
        }
        "acme.sh".to_string() // fallback
    }

    fn run_acme(args: &[&str]) -> Result<String, ServiceError> {
        let acme_sh = Self::find_acme_sh();
        let output = Command::new(&acme_sh)
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
    ) -> Result<CertInfo, ServiceError> {
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
                // acme.sh handles both Global API Key (CF_Key + CF_Email) 
                // and API Token (CF_Token)
                if let Some(token) = cf_key {
                    if token.starts_with("cfut_") || token.len() > 30 {
                        std::env::set_var("CF_Token", token);
                    } else {
                        std::env::set_var("CF_Key", token);
                        if let Some(email) = cf_email {
                            std::env::set_var("CF_Email", email);
                        }
                    }
                }
            }
            _ => return Err(ServiceError::Acme(format!("Unknown method: {}", method))),
        }

        let output = Self::run_acme(&args)?;
        if output.contains("Certificate success") || output.contains("Cert success") || output.contains("Domains not changed") {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            // Install to standard Xray cert dir
            let cert_dir = format!("/etc/xray/certs/{}", domain);
            let _ = std::fs::create_dir_all(&cert_dir);
            let cert_path = format!("{}/fullchain.pem", cert_dir);
            let key_path = format!("{}/privkey.pem", cert_dir);

            let install_output = Self::run_acme(&[
                "--install-cert", "-d", domain,
                "--cert-file", &cert_path,
                "--key-file", &key_path,
                "--reloadcmd", "systemctl reload xray 2>/dev/null || true",
            ]);
            // Not fatal if install-cert fails — cert is still issued, just not copied
            if install_output.is_err() {
                eprintln!("acme.sh install-cert warning for {}: {:?}", domain, install_output);
            }

            let now = chrono::Local::now().date_naive();
            Ok(CertInfo {
                domain: domain.to_string(),
                cert_path,
                key_path,
                issued_at: now,
                expires_at: now + chrono::Duration::days(90),
                issuer: "Let's Encrypt".into(),
                auto_renew: false,
                renew_command: Some(format!("{}/.acme.sh/acme.sh --renew -d {} --reloadcmd 'systemctl reload xray'", home, domain)),
            })
        } else {
            Err(ServiceError::Acme(format!("Cert issue failed:\n{}", output)))
        }
    }

    pub fn renew_cert(domain: &str) -> Result<(), ServiceError> {
        let output = Self::run_acme(&["--renew", "-d", domain])?;
        if output.contains("success") {
            let cert_dir = format!("/etc/xray/certs/{}", domain);
            let _ = std::fs::create_dir_all(&cert_dir);
            let _ = Self::run_acme(&[
                "--install-cert", "-d", domain,
                "--cert-file", &format!("{}/fullchain.pem", cert_dir),
                "--key-file", &format!("{}/privkey.pem", cert_dir),
                "--reloadcmd", "systemctl reload xray 2>/dev/null || true",
            ]);
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

            // Attempt to parse dates if they exist in parts 4 and 5
            let issued_at = parts.get(4).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
            let expires_at = parts.get(5).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());

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
