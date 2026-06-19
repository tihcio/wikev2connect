/// Example: Loading VPN configuration from ZIP file
///
/// Demonstrates how to extract and parse WatchGuard configuration files
/// from a ZIP archive.

use WIKEv2Connect::utils::ZipExtractor;
use WIKEv2Connect::config;
use log::info;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("WIKEv2Connect ZIP Loader Example");

    // Path to ZIP file from WatchGuard
    let zip_path = PathBuf::from("example_config.zip");

    if !zip_path.exists() {
        eprintln!("ZIP file not found: {:?}", zip_path);
        eprintln!("Please provide a WatchGuard configuration ZIP file");
        return Ok(());
    }

    // Extract ZIP to temporary directory
    match ZipExtractor::extract_zip(&zip_path).await {
        Ok(temp_dir) => {
            info!("ZIP extracted to: {:?}", temp_dir.path());

            // Find PowerShell configuration file
            if let Ok(ps_files) = ZipExtractor::find_file(temp_dir.path(), "ps1") {
                for ps_file in ps_files {
                    info!("Found PowerShell script: {:?}", ps_file);

                    // Parse the configuration
                    match tokio::fs::read_to_string(&ps_file).await {
                        Ok(content) => {
                            match config::parse_powershell_config(&content).await {
                                Ok(cfg) => {
                                    info!("Parsed configuration:");
                                    info!("  Name: {}", cfg.name);
                                    info!("  Server: {}", cfg.server_address);
                                    info!("  DH Group: {}", cfg.dh_group);
                                    info!("  Encryption: {}", cfg.encryption_method);
                                    info!("  Integrity: {}", cfg.integrity_check);

                                    let proposal = cfg.to_proposal();
                                    info!("  IKE Proposal: {}", proposal.ike);
                                    info!("  ESP Proposal: {}", proposal.esp);
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read file: {}", e);
                        }
                    }
                }
            }

            // Find certificate files
            if let Ok(pem_files) = ZipExtractor::find_file(temp_dir.path(), "pem") {
                for pem_file in pem_files {
                    info!("Found certificate: {:?}", pem_file);
                }
            }

            if let Ok(crt_files) = ZipExtractor::find_file(temp_dir.path(), "crt") {
                for crt_file in crt_files {
                    info!("Found CRT certificate: {:?}", crt_file);
                }
            }

            // Temporary directory is automatically cleaned up when temp_dir goes out of scope
        }
        Err(e) => {
            eprintln!("Failed to extract ZIP: {}", e);
        }
    }

    Ok(())
}
