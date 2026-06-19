use crate::errors::{WIKEv2ConnectError, Result};
use crate::vpn::models::VpnConfig;
use regex::Regex;
use std::fs;
use std::path::Path;
use log::{debug, error};

pub fn parse_powershell_config(content: &str) -> Result<VpnConfig> {
    debug!("Parsing PowerShell configuration");

    let mut config = VpnConfig {
        name: String::new(),
        server_address: String::new(),
        dh_group: String::new(),
        encryption_method: String::new(),
        integrity_check: String::new(),
        cipher_transform: String::new(),
        auth_transform: String::new(),
    };

    if let Some(name) = extract_param(content, "Name") {
        config.name = name;
    }
    if let Some(addr) = extract_param(content, "ServerAddress") {
        config.server_address = addr;
    }
    if let Some(dh) = extract_param(content, "DHGroup") {
        config.dh_group = dh;
    }
    if let Some(enc) = extract_param(content, "EncryptionMethod") {
        config.encryption_method = enc;
    }
    if let Some(integrity) = extract_param(content, "IntegrityCheckMethod") {
        config.integrity_check = integrity;
    }
    if let Some(cipher) = extract_param(content, "CipherTransformConstants") {
        config.cipher_transform = cipher;
    }
    if let Some(auth) = extract_param(content, "AuthenticationTransformConstants") {
        config.auth_transform = auth;
    }

    if config.server_address.is_empty() {
        error!("PowerShell config parsing failed: missing server address");
        return Err(WIKEv2ConnectError::ConfigError(
            "Missing server address in configuration".to_string(),
        ));
    }

    debug!("PowerShell config parsed: {}", config.name);
    Ok(config)
}

fn extract_param(content: &str, param: &str) -> Option<String> {
    let patterns = vec![
        format!("-{}\\s+'([^']+)'", param),
        format!("-{}\\s+\"([^\"]+)\"", param),
        format!("-{}\\s+([^\\s,\\)]+)", param),
        format!("{}\\s*=\\s*'([^']+)'", param),
        format!("{}\\s*=\\s*\"([^\"]+)\"", param),
        format!("{}\\s*=\\s*([^\\s,\\)]+)", param),
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(caps) = re.captures(content) {
                if let Some(value) = caps.get(1) {
                    return Some(value.as_str().to_string());
                }
            }
        }
    }

    None
}

pub fn parse_config_from_file<P: AsRef<Path>>(path: P) -> Result<VpnConfig> {
    let content = fs::read_to_string(path)
        .map_err(WIKEv2ConnectError::FileError)?;

    if content.contains("param") || content.contains("-ServerAddress") {
        parse_powershell_config(&content)
    } else {
        Err(WIKEv2ConnectError::ConfigError(
            "Unsupported configuration format".to_string(),
        ))
    }
}

pub fn extract_ip_from_config(config: &VpnConfig) -> String {
    config.server_address.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_param() {
        let ps_script = r#"
        param(
            [string]$ServerAddress = 'vpn.example.com',
            [string]$Name = 'MyVPN'
        )
        "#;

        assert_eq!(extract_param(ps_script, "ServerAddress"), Some("vpn.example.com".to_string()));
        assert_eq!(extract_param(ps_script, "Name"), Some("MyVPN".to_string()));
    }
}
