// This module is now replaced by window.rs
// Kept for backwards compatibility

use crate::errors::Result;
use log::info;

pub async fn launch_main_window() -> Result<()> {
    info!("Main window initialized");
    
    // Redirects to new GUI implementation
    // This function is no longer used in Fase 2
    Ok(())
}
