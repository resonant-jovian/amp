use crate::models::Address;
use parking_lot::RwLock;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Thread-safe application state
#[derive(Clone)]
pub struct AppState {
    addresses: Arc<RwLock<Vec<Address>>>,
    sync_timestamp: Arc<RwLock<Option<String>>>,
}

pub struct AppStateSnapshot {
    pub addresses: Vec<Address>,
    pub last_sync: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            addresses: Arc::new(RwLock::new(Vec::new())),
            sync_timestamp: Arc::new(RwLock::new(None)),
        }
    }

    pub fn add_address(&self, address: Address) {
        let mut addrs = self.addresses.write();
        if !addrs.iter().any(|a| a.id == address.id) {
            addrs.push(address);
        }
    }

    pub fn remove_address(&self, id: &str) {
        let mut addrs = self.addresses.write();
        addrs.retain(|a| a.id != id);
    }

    pub fn get_addresses(&self) -> Vec<Address> {
        self.addresses.read().clone()
    }

    pub fn get_active_addresses(&self) -> Vec<Address> {
        self.addresses.read()
            .iter()
            .filter(|a| a.active)
            .cloned()
            .collect()
    }

    pub fn toggle_address(&self, id: &str) {
        let mut addrs = self.addresses.write();
        if let Some(addr) = addrs.iter_mut().find(|a| a.id == id) {
            addr.active = !addr.active;
        }
    }

    pub fn clear_all(&self) {
        self.addresses.write().clear();
    }

    pub fn set_sync_timestamp(&self, timestamp: String) {
        *self.sync_timestamp.write() = Some(timestamp);
    }

    pub fn get_sync_timestamp(&self) -> Option<String> {
        self.sync_timestamp.read().clone()
    }

    pub fn snapshot(&self) -> AppStateSnapshot {
        AppStateSnapshot {
            addresses: self.get_addresses(),
            last_sync: self.get_sync_timestamp(),
        }
    }

    pub fn address_count(&self) -> usize {
        self.addresses.read().len()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_management() {
        let state = AppState::new();

        let addr = Address {
            id: "1".to_string(),
            name: "Home".to_string(),
            street: "Storgatan 1".to_string(),
            coordinates: Some("55.6050, 13.0038".to_string()),
            active: true,
        };

        state.add_address(addr.clone());
        assert_eq!(state.address_count(), 1);

        state.toggle_address("1");
        let addrs = state.get_addresses();
        assert!(!addrs[0].active);

        state.clear_all();
        assert_eq!(state.address_count(), 0);
    }
}
