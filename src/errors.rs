use thiserror::Error;

#[derive(Error, Debug)]
pub enum WIKEv2ConnectError {
    #[error("VPN connection error: {0}")]
    VpnError(String),

    #[error("Configuration parsing error: {0}")]
    ConfigError(String),

    #[error("Certificate error: {0}")]
    CertError(String),

    #[error("System error: {0}")]
    SystemError(String),

    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Command execution failed: {0}")]
    CommandError(String),

    #[error("Missing prerequisite: {0}")]
    MissingPrerequisite(String),

    #[error("GUI error: {0}")]
    GuiError(String),
}

pub type Result<T> = std::result::Result<T, WIKEv2ConnectError>;
