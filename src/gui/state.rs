use crate::errors::Result;
use crate::vpn::VpnManager;
use crate::vpn::models::VpnConnection;
use parking_lot::RwLock;
use std::sync::Arc;
use log::info;

/// Application-wide state
#[derive(Clone)]
pub struct AppState {
    pub connections: Arc<RwLock<Vec<VpnConnection>>>,
    pub selected_connection: Arc<RwLock<Option<String>>>,
    pub is_loading: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            selected_connection: Arc::new(RwLock::new(None)),
            is_loading: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn refresh_connections(&self) -> Result<()> {
        info!("Refreshing VPN connections list");
        
        *self.is_loading.write() = true;
        
        match VpnManager::list_connections().await {
            Ok(conns) => {
                *self.connections.write() = conns;
                info!("Refreshed {} connections", self.connections.read().len());
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to refresh connections: {}", e);
                Err(e)
            }
        }
        .finally(|| {
            *self.is_loading.write() = false;
        })
    }

    pub fn get_connections(&self) -> Vec<VpnConnection> {
        self.connections.read().clone()
    }

    pub fn select_connection(&self, name: String) {
        *self.selected_connection.write() = Some(name);
    }

    pub fn get_selected(&self) -> Option<String> {
        self.selected_connection.read().clone()
    }

    pub fn is_loading(&self) -> bool {
        *self.is_loading.read()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

trait Finally<T> {
    fn finally<F: FnOnce()>(self, f: F) -> Self;
}

impl<T> Finally<T> for Result<T> {
    fn finally<F: FnOnce()>(self, f: F) -> Self {
        f();
        self
    }
}
