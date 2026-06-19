use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnProposal {
    pub ike: String, // e.g., "aes256-sha256-modp2048"
    pub esp: String, // e.g., "aes256-sha1"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnConnection {
    pub name: String,
    pub server_address: String,
    pub username: String,
    pub certificate_path: String,
    pub ike_proposal: String,
    pub esp_proposal: String,
    pub status: ConnectionStatus,
    pub dns_suffix: Option<String>,
    pub encap: bool,
    pub ipcomp: bool,
    #[serde(skip)]
    pub password: String, // transient: used in form/creation only, never persisted
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnConfig {
    pub name: String,
    pub server_address: String,
    pub dh_group: String,
    pub encryption_method: String,
    pub integrity_check: String,
    pub cipher_transform: String,
    pub auth_transform: String,
}

impl VpnConfig {
    pub fn to_proposal(&self) -> VpnProposal {
        let ike = format!(
            "{}-{}-{}",
            self.encryption_method.to_lowercase(),
            self.integrity_check.to_lowercase(),
            dh_group_to_proposal(&self.dh_group)
        );

        let esp = format!(
            "{}-{}",
            self.cipher_transform.to_lowercase(),
            auth_transform_to_name(&self.auth_transform)
        );

        VpnProposal { ike, esp }
    }
}

fn dh_group_to_proposal(group: &str) -> String {
    match group.to_lowercase().as_str() {
        "group14" => "modp2048".to_string(),
        "group19" => "ecp256".to_string(),
        "group20" => "ecp384".to_string(),
        _ => "modp2048".to_string(),
    }
}

fn auth_transform_to_name(transform: &str) -> String {
    match transform {
        "SHA196" => "sha1".to_string(),
        "SHA256128" => "sha256".to_string(),
        _ => "sha1".to_string(),
    }
}
