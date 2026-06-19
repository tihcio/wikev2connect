/// Example: Using WIKEv2Connect programmatically
///
/// This example demonstrates how to use the core VPN management functions
/// without the GUI.

use WIKEv2Connect::vpn::{VpnManager, models::VpnProposal};
use WIKEv2Connect::config;
use WIKEv2Connect::cert;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("WIKEv2Connect CLI Example");

    // Example 1: Parse PowerShell configuration
    let ps_script = r#"
    param(
        [string]$ServerAddress = 'vpn.example.com',
        [string]$Name = 'MyVPN',
        [string]$DnsSuffix = 'example.local'
    )
    $DHGroup = 'Group14'
    $EncryptionMethod = 'AES256'
    $IntegrityCheckMethod = 'SHA256'
    $CipherTransformConstants = 'AES256'
    $AuthenticationTransformConstants = 'SHA196'
    "#;

    match config::parse_powershell_config(ps_script).await {
        Ok(cfg) => {
            info!("Config parsed: {:?}", cfg);
            let proposal = cfg.to_proposal();
            info!("IKE: {}, ESP: {}", proposal.ike, proposal.esp);
        }
        Err(e) => {
            eprintln!("Failed to parse config: {}", e);
        }
    }

    // Example 2: List existing VPN connections
    match VpnManager::list_connections().await {
        Ok(conns) => {
            info!("Found {} VPN connections", conns.len());
            for conn in conns {
                info!("  - {}: {}", conn.name, conn.server_address);
            }
        }
        Err(e) => {
            eprintln!("Failed to list connections: {}", e);
        }
    }

    // Example 3: Create a new VPN connection
    let proposal = VpnProposal {
        ike: "aes256-sha256-modp2048".to_string(),
        esp: "aes256-sha1".to_string(),
    };

    match VpnManager::create_connection(
        "TestVPN",
        "vpn.example.com",
        &proposal,
        "/etc/pki/ca-trust/source/anchors/MyVPN-WatchGuard.pem",
        "Firebox-DB\\\\user",
        "password123",
    ).await {
        Ok(conn) => {
            info!("Connection created: {}", conn.name);
        }
        Err(e) => {
            eprintln!("Failed to create connection: {}", e);
        }
    }

    // Example 4: Connect to VPN
    match VpnManager::connect("TestVPN").await {
        Ok(_) => {
            info!("Connected to VPN");
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
        }
    }

    // Example 5: Get connection status
    match VpnManager::get_connection_status("TestVPN").await {
        Ok(status) => {
            info!("Status: {}", status);
        }
        Err(e) => {
            eprintln!("Failed to get status: {}", e);
        }
    }

    Ok(())
}
