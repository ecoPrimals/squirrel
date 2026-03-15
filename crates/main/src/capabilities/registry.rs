// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability Registry Loader
//!
//! Loads `capability_registry.toml` at startup and serves as the single source
//! of truth for `capability.discover` and `tool.list` responses.
//!
//! Pattern adopted from wetSpring's `capability_registry.toml`.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use tracing::{info, warn};

/// Parsed capability registry (from capability_registry.toml)
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    pub primal: PrimalInfo,
    pub capabilities: Vec<CapabilityDef>,
}

/// Primal metadata from the [primal] section
#[derive(Debug, Clone)]
pub struct PrimalInfo {
    pub name: String,
    pub version: String,
    pub domain: String,
    pub license: String,
    pub transport: String,
    pub protocol: String,
}

/// A single capability definition with optional JSON Schema
#[derive(Debug, Clone)]
pub struct CapabilityDef {
    pub method: String,
    pub domain: String,
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
}

// --- TOML deserialization shapes (private) ---

#[derive(Deserialize)]
struct RawRegistry {
    primal: RawPrimal,
    capabilities: BTreeMap<String, RawCapability>,
}

#[derive(Deserialize)]
struct RawPrimal {
    name: String,
    version: String,
    domain: String,
    license: String,
    #[serde(default = "default_transport")]
    transport: String,
    #[serde(default = "default_protocol")]
    protocol: String,
}

fn default_transport() -> String {
    "unix_socket".to_string()
}
fn default_protocol() -> String {
    "jsonrpc_2.0".to_string()
}

#[derive(Deserialize)]
struct RawCapability {
    method: String,
    domain: String,
    description: String,
    input_schema: Option<toml::Value>,
}

impl CapabilityRegistry {
    /// Load from a TOML file path. Falls back to compiled-in defaults on error.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match toml::from_str::<RawRegistry>(&contents) {
                Ok(raw) => {
                    let registry = Self::from_raw(raw);
                    info!(
                        "Loaded capability registry: {} capabilities from {}",
                        registry.capabilities.len(),
                        path.display()
                    );
                    registry
                }
                Err(e) => {
                    warn!(
                        "Failed to parse {}: {e}. Using compiled defaults.",
                        path.display()
                    );
                    Self::compiled_defaults()
                }
            },
            Err(e) => {
                warn!(
                    "Failed to read {}: {e}. Using compiled defaults.",
                    path.display()
                );
                Self::compiled_defaults()
            }
        }
    }

    fn from_raw(raw: RawRegistry) -> Self {
        let capabilities = raw
            .capabilities
            .into_values()
            .map(|c| CapabilityDef {
                method: c.method,
                domain: c.domain,
                description: c.description,
                input_schema: c.input_schema.and_then(|v| toml_to_json(v)),
            })
            .collect();

        Self {
            primal: PrimalInfo {
                name: raw.primal.name,
                version: raw.primal.version,
                domain: raw.primal.domain,
                license: raw.primal.license,
                transport: raw.primal.transport,
                protocol: raw.primal.protocol,
            },
            capabilities,
        }
    }

    /// Return just the method names (for capability.discover response)
    pub fn method_names(&self) -> Vec<&str> {
        self.capabilities.iter().map(|c| c.method.as_str()).collect()
    }

    /// Find a capability by method name
    pub fn find(&self, method: &str) -> Option<&CapabilityDef> {
        self.capabilities.iter().find(|c| c.method == method)
    }

    /// Hardcoded fallback matching the methods in jsonrpc_server dispatch table
    fn compiled_defaults() -> Self {
        let methods = [
            ("ai.query", "ai.inference", "Route prompt to best model"),
            ("ai.list_providers", "ai.discovery", "List AI providers"),
            ("ai.complete", "ai.inference", "Text completion"),
            ("ai.chat", "ai.inference", "Chat completion"),
            (
                "capability.announce",
                "capability.routing",
                "Accept announcements",
            ),
            (
                "capability.discover",
                "capability.discovery",
                "Report capabilities",
            ),
            ("system.health", "system.monitoring", "Health check"),
            (
                "system.status",
                "system.monitoring",
                "Status (health alias)",
            ),
            ("system.metrics", "system.monitoring", "Server metrics"),
            ("system.ping", "system.monitoring", "Connectivity test"),
            ("discovery.peers", "discovery.network", "Scan for primals"),
            ("tool.execute", "tool.orchestration", "Execute tools"),
            ("tool.list", "tool.discovery", "List tools"),
        ];

        Self {
            primal: PrimalInfo {
                name: "squirrel".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                domain: "ai".to_string(),
                license: "AGPL-3.0-or-later".to_string(),
                transport: "unix_socket".to_string(),
                protocol: "jsonrpc_2.0".to_string(),
            },
            capabilities: methods
                .into_iter()
                .map(|(method, domain, desc)| CapabilityDef {
                    method: method.to_string(),
                    domain: domain.to_string(),
                    description: desc.to_string(),
                    input_schema: None,
                })
                .collect(),
        }
    }
}

/// Convert a TOML Value to a serde_json::Value
fn toml_to_json(value: toml::Value) -> Option<serde_json::Value> {
    match value {
        toml::Value::String(s) => Some(serde_json::Value::String(s)),
        toml::Value::Integer(i) => Some(serde_json::json!(i)),
        toml::Value::Float(f) => Some(serde_json::json!(f)),
        toml::Value::Boolean(b) => Some(serde_json::Value::Bool(b)),
        toml::Value::Array(arr) => {
            let json_arr: Vec<serde_json::Value> =
                arr.into_iter().filter_map(toml_to_json).collect();
            Some(serde_json::Value::Array(json_arr))
        }
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (k, v) in table {
                if let Some(json_v) = toml_to_json(v) {
                    map.insert(k, json_v);
                }
            }
            Some(serde_json::Value::Object(map))
        }
        toml::Value::Datetime(dt) => Some(serde_json::Value::String(dt.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiled_defaults_has_core_methods() {
        let registry = CapabilityRegistry::compiled_defaults();
        let methods = registry.method_names();
        assert!(methods.contains(&"ai.query"));
        assert!(methods.contains(&"capability.discover"));
        assert!(methods.contains(&"tool.list"));
        assert!(methods.contains(&"system.health"));
        assert!(methods.len() >= 13);
    }

    #[test]
    fn test_find_capability() {
        let registry = CapabilityRegistry::compiled_defaults();
        let cap = registry.find("ai.query");
        assert!(cap.is_some());
        assert_eq!(cap.unwrap().domain, "ai.inference");
    }

    #[test]
    fn test_find_missing_capability() {
        let registry = CapabilityRegistry::compiled_defaults();
        assert!(registry.find("nonexistent.method").is_none());
    }

    #[test]
    fn test_load_real_file() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let registry_path = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("capability_registry.toml"));

        if let Some(path) = registry_path {
            if path.exists() {
                let registry = CapabilityRegistry::load(&path);
                assert_eq!(registry.primal.name, "squirrel");
                assert!(!registry.capabilities.is_empty());
                let methods = registry.method_names();
                assert!(methods.contains(&"ai.query"));
                assert!(methods.contains(&"context.create"));
            }
        }
    }

    #[test]
    fn test_load_missing_file_uses_defaults() {
        let registry = CapabilityRegistry::load(Path::new("/nonexistent/path.toml"));
        assert_eq!(registry.primal.name, "squirrel");
        assert!(!registry.capabilities.is_empty());
    }

    #[test]
    fn test_toml_to_json_conversion() {
        let toml_val = toml::Value::Table({
            let mut t = toml::map::Map::new();
            t.insert("type".to_string(), toml::Value::String("object".to_string()));
            t.insert(
                "required".to_string(),
                toml::Value::Array(vec![toml::Value::String("prompt".to_string())]),
            );
            t
        });
        let json = toml_to_json(toml_val).unwrap();
        assert_eq!(json["type"], "object");
        assert!(json["required"].is_array());
    }
}
