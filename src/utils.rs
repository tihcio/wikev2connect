use crate::errors::{WIKEv2ConnectError, Result};
use log::{debug, info, error};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::ZipArchive;

pub struct ZipExtractor;

impl ZipExtractor {
    /// Extract ZIP file to temporary directory
    pub async fn extract_zip<P: AsRef<Path>>(zip_path: P) -> Result<TempDir> {
        let zip_path = zip_path.as_ref();

        if !zip_path.exists() {
            error!("ZIP file not found: {:?}", zip_path);
            return Err(WIKEv2ConnectError::FileError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ZIP file not found: {:?}", zip_path),
            )));
        }

        let temp_dir = TempDir::new()
            .map_err(|e| WIKEv2ConnectError::FileError(e))?;

        let file = fs::File::open(zip_path)
            .map_err(|e| WIKEv2ConnectError::FileError(e))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| WIKEv2ConnectError::ZipError(e))?;

        debug!("Extracting ZIP file: {:?}", zip_path);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| WIKEv2ConnectError::ZipError(e))?;

            let outpath = temp_dir.path().join(file.mangled_name());

            if file.is_dir() {
                fs::create_dir_all(&outpath)
                    .map_err(|e| WIKEv2ConnectError::FileError(e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)
                            .map_err(|e| WIKEv2ConnectError::FileError(e))?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| WIKEv2ConnectError::FileError(e))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| WIKEv2ConnectError::FileError(e))?;
            }
        }

        info!("ZIP file extracted to: {:?}", temp_dir.path());
        Ok(temp_dir)
    }

    /// Find specific file types in extracted ZIP
    pub fn find_file<P: AsRef<Path>>(dir: P, extension: &str) -> Result<Vec<PathBuf>> {
        let mut results = Vec::new();

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
            {
                results.push(path.to_path_buf());
            }
        }

        Ok(results)
    }

    /// Find file by name pattern
    pub fn find_file_by_name<P: AsRef<Path>>(dir: P, name: &str) -> Result<Option<PathBuf>> {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(name) || n.eq_ignore_ascii_case(name))
                .unwrap_or(false)
            {
                return Ok(Some(path.to_path_buf()));
            }
        }

        Ok(None)
    }
}

/// Parse file name to extract client name
pub fn extract_client_name_from_filename(filename: &str) -> String {
    filename
        .split('.')
        .next()
        .unwrap_or("Unknown")
        .to_string()
}

/// Validate certificate file
pub fn validate_certificate_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .map_err(|e| WIKEv2ConnectError::FileError(e))?;

    if !content.contains("BEGIN CERTIFICATE") {
        return Err(WIKEv2ConnectError::CertError(
            "Invalid certificate format: missing BEGIN CERTIFICATE marker".to_string(),
        ));
    }

    if !content.contains("END CERTIFICATE") {
        return Err(WIKEv2ConnectError::CertError(
            "Invalid certificate format: missing END CERTIFICATE marker".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_client_name() {
        assert_eq!(extract_client_name_from_filename("MyVPN.pem"), "MyVPN");
        assert_eq!(extract_client_name_from_filename("Client123.crt"), "Client123");
    }
}
