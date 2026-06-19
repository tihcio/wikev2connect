use crate::gui::state::AppState;
use crate::vpn::models::VpnConnection;
use crate::errors::Result;
use log::info;

pub struct VpnListWidget {
    pub state: AppState,
}

impl VpnListWidget {
    pub fn new(_x: i32, _y: i32, _w: i32, _h: i32, state: AppState) -> Result<Self> {
        info!("Creating VPN list widget");

        Ok(Self { state })
    }

    pub fn load_connections(&mut self, _connections: Vec<VpnConnection>) {
        info!("Loaded connections");
    }

    pub fn refresh(&mut self) {
        let conns = self.state.get_connections();
        self.load_connections(conns);
        info!("Refreshed VPN list");
    }
}

pub struct ConnectionFormWidget;

impl ConnectionFormWidget {
    pub fn new() -> Self {
        Self
    }
}

pub struct StatusIndicator;

impl StatusIndicator {
    pub fn new() -> Self {
        Self
    }
}
