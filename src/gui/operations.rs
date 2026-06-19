use crate::vpn::VpnManager;
use crate::vpn::models::{VpnConnection, VpnProposal, ConnectionStatus};
use crate::config;
use crate::utils::ZipExtractor;
use crate::errors::{WIKEv2ConnectError, Result};
use log::{info, error};
use std::fs;
use std::path::Path;

// ── Config file parsing ───────────────────────────────────────────────────────

pub fn parse_config_file(file_path: &str) -> Result<VpnConnection> {
    info!("Parsing config file: {}", file_path);

    let (config_content, cert_path) = if file_path.ends_with(".zip") {
        let zip_file = fs::File::open(file_path).map_err(WIKEv2ConnectError::FileError)?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(WIKEv2ConnectError::ZipError)?;
        let temp_dir = tempfile::TempDir::new().map_err(WIKEv2ConnectError::FileError)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(WIKEv2ConnectError::ZipError)?;
            let outpath = temp_dir.path().join(file.mangled_name());
            if file.is_dir() {
                fs::create_dir_all(&outpath).map_err(WIKEv2ConnectError::FileError)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(WIKEv2ConnectError::FileError)?;
                    }
                }
                let mut out = fs::File::create(&outpath).map_err(WIKEv2ConnectError::FileError)?;
                std::io::copy(&mut file, &mut out).map_err(WIKEv2ConnectError::FileError)?;
            }
        }

        let ps1_files = ZipExtractor::find_file(temp_dir.path(), "ps1")?;
        let pem_files = ZipExtractor::find_file(temp_dir.path(), "pem")?;
        let crt_files = ZipExtractor::find_file(temp_dir.path(), "crt")?;

        let ps1_path = ps1_files.into_iter().next()
            .ok_or_else(|| WIKEv2ConnectError::ConfigError("No .ps1 file found in ZIP".to_string()))?;

        let content = read_text_file(&ps1_path).map_err(WIKEv2ConnectError::FileError)?;

        let cert = pem_files.into_iter().next()
            .or_else(|| crt_files.into_iter().next())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let _ = temp_dir.keep();
        (content, cert)
    } else if file_path.ends_with(".ps1") {
        let content = read_text_file(Path::new(file_path)).map_err(WIKEv2ConnectError::FileError)?;
        (content, String::new())
    } else {
        return Err(WIKEv2ConnectError::ConfigError("Formato non supportato. Usa .zip o .ps1".to_string()));
    };

    let vpn_config = config::parse_powershell_config(&config_content)?;
    let proposal = vpn_config.to_proposal();

    let name = if !vpn_config.name.is_empty() {
        vpn_config.name.clone()
    } else {
        Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("VPN")
            .to_string()
    };

    Ok(VpnConnection {
        name,
        server_address: vpn_config.server_address,
        username: String::new(),
        certificate_path: cert_path,
        ike_proposal: proposal.ike,
        esp_proposal: proposal.esp,
        status: ConnectionStatus::Disconnected,
        dns_suffix: None,
        encap: false,
        ipcomp: false,
        password: String::new(),
    })
}

// ── VPN lifecycle ─────────────────────────────────────────────────────────────

pub async fn list_connections() -> Result<Vec<VpnConnection>> {
    VpnManager::list_connections().await
}

pub async fn create_connection(conn: VpnConnection) -> Result<()> {
    info!("Creating VPN connection: {}", conn.name);
    let proposal = VpnProposal { ike: conn.ike_proposal.clone(), esp: conn.esp_proposal.clone() };

    // Install certificate if not already in system path
    let cert_path = if !conn.certificate_path.is_empty()
        && !conn.certificate_path.starts_with("/etc/pki")
    {
        match crate::cert::install_certificate(&conn.certificate_path, &conn.name).await {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => { error!("Certificate install failed: {}", e); conn.certificate_path.clone() }
        }
    } else {
        conn.certificate_path.clone()
    };

    VpnManager::create_connection(&conn.name, &conn.server_address, &proposal, &cert_path, &conn.username, &conn.password).await.map(|_| ())
}

pub async fn update_connection(original_name: &str, conn: VpnConnection) -> Result<()> {
    info!("Updating VPN connection: {}", original_name);
    let mut updates = std::collections::HashMap::new();
    if !conn.username.is_empty()         { updates.insert("user".to_string(),        conn.username.clone()); }
    if !conn.server_address.is_empty()   { updates.insert("address".to_string(),     conn.server_address.clone()); }
    if !conn.certificate_path.is_empty() { updates.insert("certificate".to_string(), conn.certificate_path.clone()); }
    if !conn.ike_proposal.is_empty()     { updates.insert("ike".to_string(),         conn.ike_proposal.clone()); }
    if !conn.esp_proposal.is_empty()     { updates.insert("esp".to_string(),         conn.esp_proposal.clone()); }
    if !updates.is_empty() {
        VpnManager::modify_connection(original_name, updates).await?;
    }
    if !conn.password.is_empty() {
        let _ = async_process::Command::new("nmcli")
            .args(["connection", "modify", original_name, "vpn.secrets",
                   &format!("password={}", conn.password)])
            .output().await;
    }
    Ok(())
}

pub async fn delete_connection(name: &str) -> Result<()> {
    VpnManager::delete_connection(name).await
}

pub async fn connect_vpn(name: &str) -> Result<()> {
    VpnManager::connect(name).await
}

pub async fn disconnect_vpn(name: &str) -> Result<()> {
    VpnManager::disconnect(name).await
}

// ── Encoding-aware file reader ────────────────────────────────────────────────

fn read_text_file(path: &Path) -> std::io::Result<String> {
    let bytes = fs::read(path)?;

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let words: Vec<u16> = bytes[2..].chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        return String::from_utf16(&words)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let words: Vec<u16> = bytes[2..].chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        return String::from_utf16(&words)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(bytes[3..].to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }
    if let Ok(s) = String::from_utf8(bytes.clone()) { return Ok(s); }
    // Latin-1 / Windows-1252 fallback
    Ok(bytes.iter().map(|&b| b as char).collect())
}
