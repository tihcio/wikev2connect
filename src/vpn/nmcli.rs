use crate::errors::{WIKEv2ConnectError, Result};
use crate::vpn::models::{VpnConnection, VpnProposal, ConnectionStatus};
use async_process::Command;
use log::{debug, error, info};
use std::collections::HashMap;

pub async fn list_vpn_connections() -> Result<Vec<VpnConnection>> {
    // List ALL connections (active and inactive) with name, type and active status
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "NAME,TYPE,ACTIVE", "connection", "show"])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli failed: {}", e)))?;

    if !output.status.success() {
        debug!("nmcli error: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut connections = Vec::new();

    for line in stdout.lines() {
        // Format: NAME:TYPE:ACTIVE  (but NAME may contain colons — split from right)
        let parts: Vec<&str> = line.rsplitn(3, ':').collect();
        if parts.len() < 2 {
            continue;
        }
        // rsplitn gives parts in reverse order: [ACTIVE, TYPE, NAME]
        let active = parts[0] == "yes";
        let conn_type = parts[1];
        let name = if parts.len() == 3 { parts[2] } else { continue };

        if conn_type != "vpn" && !conn_type.contains("strongswan") {
            continue;
        }

        let mut conn = get_connection_details(name).await.unwrap_or_else(|_| VpnConnection {
            name: name.to_string(),
            server_address: String::new(),
            username: String::new(),
            certificate_path: String::new(),
            ike_proposal: String::new(),
            esp_proposal: String::new(),
            status: ConnectionStatus::Disconnected,
            dns_suffix: None,
            encap: false,
            ipcomp: false,
            password: String::new(),
        });

        conn.status = if active { ConnectionStatus::Connected } else { ConnectionStatus::Disconnected };
        connections.push(conn);
    }

    Ok(connections)
}

pub async fn create_vpn_connection(
    name: &str,
    server: &str,
    proposal: &VpnProposal,
    cert_path: &str,
    username: &str,
    password: &str,
) -> Result<VpnConnection> {
    let vpn_data = format!(
        "address={}, method=eap, user={}, certificate={}, virtual=yes, encap=no, ipcomp=no, proposal=yes, ike={}, esp={}, password-flags=1",
        server, username, cert_path, proposal.ike, proposal.esp
    );

    let output = Command::new("nmcli")
        .args(&[
            "connection",
            "add",
            "type",
            "vpn",
            "con-name",
            name,
            "vpn-type",
            "org.freedesktop.NetworkManager.strongswan",
            "vpn.data",
            &vpn_data,
            "vpn.secrets",
            &format!("password={}", password),
        ])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli add connection failed: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        error!("Failed to create VPN connection: {}", err_msg);
        return Err(WIKEv2ConnectError::VpnError(format!(
            "Failed to create connection: {}",
            err_msg
        )));
    }

    info!("VPN connection '{}' created successfully", name);
    get_connection_details(name).await
}

pub async fn modify_vpn_connection(
    name: &str,
    updates: HashMap<String, String>,
) -> Result<()> {
    for (key, value) in updates {
        let output = Command::new("nmcli")
            .args(&["connection", "modify", name, &format!("vpn.data.{}", key), &value])
            .output()
            .await
            .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli modify failed: {}", e)))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to modify connection: {}", err_msg);
            return Err(WIKEv2ConnectError::VpnError(format!(
                "Failed to modify {}: {}",
                key, err_msg
            )));
        }
    }

    info!("VPN connection '{}' modified successfully", name);
    Ok(())
}

pub async fn delete_vpn_connection(name: &str) -> Result<()> {
    let output = Command::new("nmcli")
        .args(&["connection", "delete", name])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli delete failed: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        error!("Failed to delete connection: {}", err_msg);
        return Err(WIKEv2ConnectError::VpnError(format!(
            "Failed to delete connection: {}",
            err_msg
        )));
    }

    info!("VPN connection '{}' deleted successfully", name);
    Ok(())
}

pub async fn connect_vpn(name: &str) -> Result<()> {
    let output = Command::new("nmcli")
        .args(&["connection", "up", name])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli connect failed: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        error!("Failed to connect VPN: {}", err_msg);
        return Err(WIKEv2ConnectError::VpnError(format!(
            "Failed to connect: {}",
            err_msg
        )));
    }

    info!("Connected to VPN '{}'", name);
    Ok(())
}

pub async fn disconnect_vpn(name: &str) -> Result<()> {
    let output = Command::new("nmcli")
        .args(&["connection", "down", name])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli disconnect failed: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        error!("Failed to disconnect VPN: {}", err_msg);
        return Err(WIKEv2ConnectError::VpnError(format!(
            "Failed to disconnect: {}",
            err_msg
        )));
    }

    info!("Disconnected from VPN '{}'", name);
    Ok(())
}

pub async fn get_connection_status(name: &str) -> Result<String> {
    let output = Command::new("nmcli")
        .args(&["-t", "-f", "ACTIVE", "connection", "show", name])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli status check failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout == "yes" {
        Ok("Connected".to_string())
    } else {
        Ok("Disconnected".to_string())
    }
}

async fn get_connection_details(name: &str) -> Result<VpnConnection> {
    let output = Command::new("nmcli")
        .args(&["connection", "show", name])
        .output()
        .await
        .map_err(|e| WIKEv2ConnectError::CommandError(format!("nmcli show failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut key_values: HashMap<String, String> = HashMap::new();

    // Parse multi-line nmcli output: "KEY:   VALUE"
    // Some values span multiple lines (continuation lines start with spaces)
    let mut current_key = String::new();
    let mut current_value = String::new();

    for line in stdout.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation of previous value
            current_value.push(' ');
            current_value.push_str(line.trim());
        } else if let Some(colon_pos) = line.find(':') {
            // Save previous key-value
            if !current_key.is_empty() {
                key_values.insert(current_key.clone(), current_value.trim().to_string());
            }
            current_key = line[..colon_pos].trim().to_string();
            current_value = line[colon_pos + 1..].trim().to_string();
        }
    }
    if !current_key.is_empty() {
        key_values.insert(current_key, current_value.trim().to_string());
    }

    let vpn_data_str = key_values.get("vpn.data").map(|s| s.as_str()).unwrap_or("");
    let vpn_data = parse_vpn_data(vpn_data_str);

    Ok(VpnConnection {
        name: name.to_string(),
        server_address: vpn_data.get("address").cloned().unwrap_or_default(),
        username: vpn_data.get("user").cloned().unwrap_or_default(),
        certificate_path: vpn_data.get("certificate").cloned().unwrap_or_default(),
        ike_proposal: vpn_data.get("ike").cloned().unwrap_or_default(),
        esp_proposal: vpn_data.get("esp").cloned().unwrap_or_default(),
        status: ConnectionStatus::Disconnected,
        dns_suffix: None,
        encap: vpn_data.get("encap").map(|v| v == "yes").unwrap_or(false),
        ipcomp: vpn_data.get("ipcomp").map(|v| v == "yes").unwrap_or(false),
        password: String::new(),
    })
}

// Parses "address = 1.2.3.4, user = foo, ike = aes256-sha256-modp2048, ..."
fn parse_vpn_data(data: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for pair in data.split(", ") {
        if let Some(eq_pos) = pair.find(" = ") {
            let key = pair[..eq_pos].trim().to_string();
            let value = pair[eq_pos + 3..].trim().to_string();
            if !key.is_empty() {
                result.insert(key, value);
            }
        }
    }
    result
}
