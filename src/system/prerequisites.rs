use crate::errors::{WIKEv2ConnectError, Result};
use async_process::Command;
use log::{debug, error, info, warn};
use which::which;

#[derive(Debug)]
pub struct PrerequisiteStatus {
    pub networkmanager: bool,
    pub networkmanager_strongswan: bool,
    pub strongswan: bool,
    pub openssl: bool,
    pub charon_nm: bool,
    pub crypto_policy_updated: bool,
}

impl PrerequisiteStatus {
    pub fn all_met(&self) -> bool {
        self.networkmanager
            && self.networkmanager_strongswan
            && self.strongswan
            && self.openssl
            && self.crypto_policy_updated
    }
}

pub async fn check_prerequisites() -> Result<PrerequisiteStatus> {
    debug!("Checking system prerequisites");

    let nm = check_package("NetworkManager").await;
    let nm_strongswan = check_package("NetworkManager-strongswan").await;
    let strongswan = check_package("strongswan").await;
    let openssl = which("openssl").is_ok();
    let charon_nm = check_command_exists("charon-nm").await;
    let crypto_policy = check_crypto_policy().await;

    let status = PrerequisiteStatus {
        networkmanager: nm,
        networkmanager_strongswan: nm_strongswan,
        strongswan: strongswan,
        openssl,
        charon_nm,
        crypto_policy_updated: crypto_policy,
    };

    debug!("Prerequisites status: {:?}", status);

    Ok(status)
}

pub async fn check_and_setup() -> Result<()> {
    let status = check_prerequisites().await?;

    if !status.all_met() {
        warn!(
            "Not all prerequisites are met. Attempting to install missing packages..."
        );
        install_missing_prerequisites(&status).await?;
    } else {
        info!("All prerequisites are met");
    }

    Ok(())
}

async fn check_package(package: &str) -> bool {
    let output = Command::new("rpm")
        .args(&["-q", package])
        .output()
        .await;

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

async fn check_command_exists(cmd: &str) -> bool {
    which(cmd).is_ok()
}

async fn check_crypto_policy() -> bool {
    // Check if SHA1 is enabled in crypto policies
    let output = Command::new("update-crypto-policies")
        .args(&["--show"])
        .output()
        .await;

    match output {
        Ok(out) => {
            let policy = String::from_utf8_lossy(&out.stdout);
            policy.contains("SHA1")
        }
        Err(_) => false,
    }
}

async fn install_missing_prerequisites(status: &PrerequisiteStatus) -> Result<()> {
    let mut packages_to_install = Vec::new();

    if !status.networkmanager {
        packages_to_install.push("NetworkManager");
    }
    if !status.networkmanager_strongswan {
        packages_to_install.push("NetworkManager-strongswan");
    }
    if !status.strongswan {
        packages_to_install.push("strongswan");
    }
    if !status.openssl {
        packages_to_install.push("openssl");
    }

    if !packages_to_install.is_empty() {
        info!("Installing packages: {:?}", packages_to_install);

        let output = Command::new("sudo")
            .args(&["dnf", "install", "-y"])
            .args(&packages_to_install)
            .output()
            .await
            .map_err(|e| {
                WIKEv2ConnectError::SystemError(format!("Failed to install packages: {}", e))
            })?;

        if !output.status.success() {
            error!(
                "Package installation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(WIKEv2ConnectError::MissingPrerequisite(
                "Failed to install required packages".to_string(),
            ));
        }

        info!("Packages installed successfully");
    }

    if !status.crypto_policy_updated {
        info!("Updating crypto policies to support SHA1");
        setup_crypto_policy().await?;
    }

    if !status.charon_nm {
        info!("Configuring strongSwan for NetworkManager");
        configure_strongswan().await?;
    }

    Ok(())
}

async fn setup_crypto_policy() -> Result<()> {
    let output = Command::new("sudo")
        .args(&["update-crypto-policies", "--set", "DEFAULT:SHA1"])
        .output()
        .await
        .map_err(|e| {
            WIKEv2ConnectError::SystemError(format!("Failed to update crypto policies: {}", e))
        })?;

    if !output.status.success() {
        error!(
            "Crypto policy update failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(WIKEv2ConnectError::SystemError(
            "Failed to update crypto policies".to_string(),
        ));
    }

    info!("Crypto policies updated");
    Ok(())
}

async fn configure_strongswan() -> Result<()> {
    debug!("Configuring strongSwan for WatchGuard compatibility");

    // Check if configuration already exists
    let config_path = "/etc/strongswan/strongswan.d/charon-nm.conf";

    // Read the config file
    let output = Command::new("sudo")
        .args(&["cat", config_path])
        .output()
        .await
        .map_err(|e| {
            WIKEv2ConnectError::SystemError(format!("Failed to read strongSwan config: {}", e))
        })?;

    let config = String::from_utf8_lossy(&output.stdout);

    // Check if already configured
    if config.contains("signature_authentication = no") {
        info!("strongSwan already configured for WatchGuard");
        return Ok(());
    }

    // Add configuration
    let sed_cmd = "s/load_modular = yes/load_modular = yes\\n    signature_authentication_constraints = no\\n    signature_authentication = no/";

    let output = Command::new("sudo")
        .args(&["sed", "-i", sed_cmd, config_path])
        .output()
        .await
        .map_err(|e| {
            WIKEv2ConnectError::SystemError(format!("Failed to configure strongSwan: {}", e))
        })?;

    if !output.status.success() {
        error!(
            "strongSwan configuration failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(WIKEv2ConnectError::SystemError(
            "Failed to configure strongSwan".to_string(),
        ));
    }

    info!("strongSwan configured successfully");

    // Restart charon-nm
    let _ = Command::new("sudo")
        .args(&["kill", "$(pgrep", "charon-nm)"])
        .output()
        .await;

    Ok(())
}

pub async fn restart_charon_nm() -> Result<()> {
    debug!("Restarting charon-nm");

    let output = Command::new("pgrep")
        .arg("charon-nm")
        .output()
        .await
        .map_err(|e| {
            WIKEv2ConnectError::SystemError(format!("Failed to get charon-nm PID: {}", e))
        })?;

    if output.status.success() {
        let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Command::new("sudo")
            .args(&["kill", &pid])
            .output()
            .await
            .map_err(|e| {
                WIKEv2ConnectError::SystemError(format!("Failed to kill charon-nm: {}", e))
            })?;

        info!("charon-nm restarted");
    }

    Ok(())
}
