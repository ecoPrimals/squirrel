use std::fmt;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub port: u16,
    pub protocol: String,
    pub service: String,
    pub description: String,
}

pub struct PortManager {
    ports: HashMap<u16, PortConfig>,
}

impl PortManager {
    pub fn new() -> Self {
        Self {
            ports: HashMap::new(),
        }
    }

    pub fn add_port(&mut self, config: PortConfig) -> Result<()> {
        if self.ports.contains_key(&config.port) {
            return Err("Port already in use".into());
        }
        self.ports.insert(config.port, config);
        Ok(())
    }

    pub fn remove_port(&mut self, port: u16) -> Option<PortConfig> {
        self.ports.remove(&port)
    }

    pub fn get_port(&self, port: u16) -> Option<&PortConfig> {
        self.ports.get(&port)
    }

    pub fn list_ports(&self) -> Vec<&PortConfig> {
        self.ports.values().collect()
    }

    pub fn is_port_available(&self, port: u16) -> bool {
        !self.ports.contains_key(&port)
    }
} 