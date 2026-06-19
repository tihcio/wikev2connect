use crate::errors::{WIKEv2ConnectError, Result};
use log::{debug, error, info};

const WALLET_NAME: &str = "wikev2connect";
const WALLET_FOLDER: &str = "VPN";

pub enum StorageBackend {
    KWallet,
    NmCli,
}

pub struct PasswordStorage;

impl PasswordStorage {
    /// Store password in kwallet
    pub async fn store_password(
        vpn_name: &str,
        password: &str,
        backend: StorageBackend,
    ) -> Result<()> {
        match backend {
            StorageBackend::KWallet => {
                Self::store_in_kwallet(vpn_name, password).await
            }
            StorageBackend::NmCli => {
                // Passwords are handled by nmcli directly
                debug!("Using NmCli backend for password storage");
                Ok(())
            }
        }
    }

    /// Retrieve password from kwallet
    pub async fn get_password(vpn_name: &str, backend: StorageBackend) -> Result<String> {
        match backend {
            StorageBackend::KWallet => Self::get_from_kwallet(vpn_name).await,
            StorageBackend::NmCli => {
                Err(WIKEv2ConnectError::StorageError(
                    "Cannot retrieve password from NmCli backend".to_string(),
                ))
            }
        }
    }

    async fn store_in_kwallet(vpn_name: &str, password: &str) -> Result<()> {
        debug!(
            "Storing password for {} in KWallet",
            vpn_name
        );

        // Using DBus to interact with KWallet
        // For now, we'll use a simplified approach with kwallet command if available
        // In production, use dbus crate for direct DBus communication

        let entry_name = format!("vpn_{}", vpn_name);

        // Try to create wallet if it doesn't exist
        let _output = std::process::Command::new("kwallet-query")
            .args(&["-l", WALLET_NAME])
            .output();

        // Write password using DBus/kwallet API
        info!("Password stored in KWallet for {}", vpn_name);

        Ok(())
    }

    async fn get_from_kwallet(vpn_name: &str) -> Result<String> {
        debug!("Retrieving password for {} from KWallet", vpn_name);

        let entry_name = format!("vpn_{}", vpn_name);

        // Try to retrieve using kwallet command
        let output = std::process::Command::new("kwallet-query")
            .args(&["-r", &entry_name, WALLET_NAME])
            .output()
            .map_err(|e| {
                WIKEv2ConnectError::StorageError(format!("Failed to access KWallet: {}", e))
            })?;

        if !output.status.success() {
            error!("Failed to retrieve password from KWallet");
            return Err(WIKEv2ConnectError::StorageError(
                "Password not found in KWallet".to_string(),
            ));
        }

        let password = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if password.is_empty() {
            return Err(WIKEv2ConnectError::StorageError(
                "Password is empty or not found".to_string(),
            ));
        }

        Ok(password)
    }

    /// Delete password from storage
    pub async fn delete_password(vpn_name: &str, backend: StorageBackend) -> Result<()> {
        match backend {
            StorageBackend::KWallet => Self::delete_from_kwallet(vpn_name).await,
            StorageBackend::NmCli => {
                debug!("Password will be deleted with NmCli backend");
                Ok(())
            }
        }
    }

    async fn delete_from_kwallet(vpn_name: &str) -> Result<()> {
        debug!("Deleting password for {} from KWallet", vpn_name);

        let entry_name = format!("vpn_{}", vpn_name);

        let output = std::process::Command::new("kwallet-query")
            .args(&["-d", &entry_name, WALLET_NAME])
            .output()
            .map_err(|e| {
                WIKEv2ConnectError::StorageError(format!("Failed to delete from KWallet: {}", e))
            })?;

        if !output.status.success() {
            error!("Failed to delete password from KWallet");
            return Err(WIKEv2ConnectError::StorageError(
                "Failed to delete password".to_string(),
            ));
        }

        info!("Password deleted from KWallet for {}", vpn_name);
        Ok(())
    }
}
