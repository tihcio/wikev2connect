use crate::errors::{WIKEv2ConnectError, Result};
use async_process::Command;
use log::{debug, info, error};
use std::fs;
use std::path::{Path, PathBuf};

const CERT_ANCHORS_DIR: &str = "/etc/pki/ca-trust/source/anchors";

// GUI askpass programs to try in order (KDE first, then GNOME fallback)
const ASKPASS_CANDIDATES: &[&str] = &[
    "/usr/bin/ksshaskpass",
    "/usr/lib64/ssh/gnome-ssh-askpass",
    "/usr/libexec/seahorse/ssh-askpass",
];

/// Run a command as root via `sudo --askpass`, using the first available GUI askpass.
/// Returns true if the command succeeded.
async fn sudo_gui(args: &[&str]) -> bool {
    let askpass = ASKPASS_CANDIDATES.iter().find(|p| Path::new(p).exists());
    if let Some(ap) = askpass {
        if let Ok(out) = Command::new("sudo")
            .env("SUDO_ASKPASS", ap)
            .arg("--askpass")
            .args(args)
            .output()
            .await
        {
            return out.status.success();
        }
    }
    // Fallback: plain sudo (works when called from a terminal or with cached credentials)
    Command::new("sudo")
        .args(args)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub async fn install_certificate<P: AsRef<Path>>(
    cert_path: P,
    cert_name: &str,
) -> Result<PathBuf> {
    let cert_path = cert_path.as_ref();

    if !cert_path.exists() {
        return Err(WIKEv2ConnectError::CertError(format!(
            "File certificato non trovato: {:?}", cert_path
        )));
    }

    let dest_name = format!("{}-WatchGuard.pem", cert_name);
    let dest_path = PathBuf::from(CERT_ANCHORS_DIR).join(&dest_name);

    // If already installed at the destination, nothing to do
    if dest_path.exists() {
        debug!("Certificate already installed at {:?}", dest_path);
        return Ok(dest_path);
    }

    debug!("Installing certificate {:?} -> {:?}", cert_path, dest_path);

    let src = cert_path.to_string_lossy();
    let dst = dest_path.to_string_lossy();

    if !sudo_gui(&["cp", "--", src.as_ref(), dst.as_ref()]).await {
        error!("Failed to copy certificate to {}", dst);
        return Err(WIKEv2ConnectError::CertError(format!(
            "Impossibile installare il certificato in {}.\n\
             Assicurarsi di avere i permessi di amministratore.", CERT_ANCHORS_DIR
        )));
    }

    // update-ca-trust regenerates the consolidated CA bundle; ignore errors
    // (strongSwan reads the cert directly from the anchors path anyway)
    if !sudo_gui(&["update-ca-trust"]).await {
        error!("update-ca-trust failed, continuing anyway");
    }

    info!("Certificato installato: {}", dest_name);
    Ok(dest_path)
}

pub async fn update_ca_trust() -> Result<()> {
    if sudo_gui(&["update-ca-trust"]).await {
        info!("CA trust database updated");
        Ok(())
    } else {
        Err(WIKEv2ConnectError::CertError("update-ca-trust fallito".into()))
    }
}

pub fn get_installed_cert_path(cert_name: &str) -> PathBuf {
    let filename = format!("{}-WatchGuard.pem", cert_name);
    PathBuf::from(CERT_ANCHORS_DIR).join(filename)
}

pub fn certificate_exists(cert_name: &str) -> bool {
    get_installed_cert_path(cert_name).exists()
}

pub async fn delete_certificate(cert_name: &str) -> Result<()> {
    let cert_path = get_installed_cert_path(cert_name);

    if !cert_path.exists() {
        return Err(WIKEv2ConnectError::CertError(format!(
            "Certificate not found: {}",
            cert_name
        )));
    }

    let output = Command::new("sudo")
        .args(&["rm", cert_path.to_str().unwrap()])
        .output()
        .await
        .map_err(|e| {
            WIKEv2ConnectError::SystemError(format!("Failed to delete certificate: {}", e))
        })?;

    if !output.status.success() {
        error!(
            "Failed to delete certificate: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(WIKEv2ConnectError::CertError(
            "Failed to delete certificate".to_string(),
        ));
    }

    // Update CA trust after deletion
    update_ca_trust().await?;

    info!("Certificate deleted: {}", cert_name);
    Ok(())
}

/// Convert DER certificate to PEM format
pub async fn convert_der_to_pem<P: AsRef<Path>>(der_path: P, pem_path: P) -> Result<()> {
    let output = Command::new("openssl")
        .args(&[
            "x509",
            "-inform",
            "DER",
            "-in",
            der_path.as_ref().to_str().unwrap(),
            "-out",
            pem_path.as_ref().to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| {
            WIKEv2ConnectError::SystemError(format!("Failed to convert DER to PEM: {}", e))
        })?;

    if !output.status.success() {
        error!(
            "DER to PEM conversion failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(WIKEv2ConnectError::CertError(
            "Failed to convert certificate format".to_string(),
        ));
    }

    debug!("Converted DER certificate to PEM");
    Ok(())
}
