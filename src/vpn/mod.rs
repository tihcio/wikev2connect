pub mod nmcli;
pub mod models;

use crate::errors::Result;
use models::{VpnConnection, VpnProposal};
use std::collections::HashMap;

#[derive(Clone)]
pub struct VpnManager;

impl VpnManager {
    pub async fn list_connections() -> Result<Vec<VpnConnection>> {
        nmcli::list_vpn_connections().await
    }

    pub async fn create_connection(
        name: &str,
        server: &str,
        proposal: &VpnProposal,
        cert_path: &str,
        username: &str,
        password: &str,
    ) -> Result<VpnConnection> {
        nmcli::create_vpn_connection(name, server, proposal, cert_path, username, password).await
    }

    pub async fn modify_connection(
        name: &str,
        updates: HashMap<String, String>,
    ) -> Result<()> {
        nmcli::modify_vpn_connection(name, updates).await
    }

    pub async fn delete_connection(name: &str) -> Result<()> {
        nmcli::delete_vpn_connection(name).await
    }

    pub async fn connect(name: &str) -> Result<()> {
        nmcli::connect_vpn(name).await
    }

    pub async fn disconnect(name: &str) -> Result<()> {
        nmcli::disconnect_vpn(name).await
    }

    pub async fn get_connection_status(name: &str) -> Result<String> {
        nmcli::get_connection_status(name).await
    }
}
