use crate::vpn::models::VpnConnection;

#[derive(Clone, Debug)]
pub struct VpnConnectionModel {
    pub connections: Vec<VpnConnection>,
}

impl VpnConnectionModel {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    pub fn add_connection(&mut self, conn: VpnConnection) {
        self.connections.push(conn);
    }

    pub fn remove_connection(&mut self, index: usize) {
        if index < self.connections.len() {
            self.connections.remove(index);
        }
    }

    pub fn get_connection(&self, index: usize) -> Option<&VpnConnection> {
        self.connections.get(index)
    }
}

impl Default for VpnConnectionModel {
    fn default() -> Self {
        Self::new()
    }
}
