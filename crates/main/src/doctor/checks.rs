// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::time::Instant;

use super::{HealthCheck, HealthStatus};

/// Check binary and version
pub fn check_binary() -> HealthCheck {
    let start = Instant::now();
    HealthCheck {
        name: "Binary",
        status: HealthStatus::Ok,
        message: format!("squirrel v{}", env!("CARGO_PKG_VERSION")),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
        })),
    }
}

/// Check configuration
pub fn check_configuration() -> HealthCheck {
    use universal_constants::env_vars;
    let start = Instant::now();

    let squirrel_port = std::env::var(env_vars::squirrel::PORT).ok();
    let squirrel_socket = std::env::var(env_vars::squirrel::SOCKET).ok();
    let ai_provider_sockets = std::env::var(env_vars::ai::PROVIDER_SOCKETS).ok();

    let status = if ai_provider_sockets.is_none() {
        HealthStatus::Warning
    } else {
        HealthStatus::Ok
    };

    let message = if ai_provider_sockets.is_some() {
        "Configuration OK".to_string()
    } else {
        "AI_PROVIDER_SOCKETS not configured".to_string()
    };

    HealthCheck {
        name: "Configuration",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "squirrel_port": squirrel_port,
            "squirrel_socket": squirrel_socket,
            "ai_provider_sockets": ai_provider_sockets,
        })),
    }
}

/// Check AI providers
pub fn check_ai_providers(comprehensive: bool) -> HealthCheck {
    use universal_constants::env_vars;
    let start = Instant::now();

    let openai_key = std::env::var(env_vars::ai::openai::API_KEY).ok();
    let huggingface_key = std::env::var(env_vars::ai::huggingface::API_KEY).ok();
    let local_ai_url = std::env::var(env_vars::ai::local::ENDPOINT)
        .or_else(|_| std::env::var(env_vars::ai::ollama::URL))
        .ok();
    let ai_provider_sockets = std::env::var(env_vars::ai::PROVIDER_SOCKETS).ok();

    let provider_count = [
        openai_key.is_some(),
        huggingface_key.is_some(),
        local_ai_url.is_some() || comprehensive,
        ai_provider_sockets.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    let (status, message) = if provider_count == 0 {
        (
            HealthStatus::Warning,
            "No AI providers configured".to_string(),
        )
    } else {
        (
            HealthStatus::Ok,
            format!("{provider_count} AI provider(s) configured"),
        )
    };

    HealthCheck {
        name: "AI Providers",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "openai": openai_key.is_some(),
            "huggingface": huggingface_key.is_some(),
            "local_server": local_ai_url.is_some(),
            "universal": ai_provider_sockets.is_some(),
            "count": provider_count,
        })),
    }
}

/// Check discovered services via capability registry
pub fn check_discovered_services() -> HealthCheck {
    let start = Instant::now();

    let runtime_dir = std::env::var(universal_constants::env_vars::sys::XDG_RUNTIME_DIR)
        .or_else(|_| {
            std::env::var(universal_constants::env_vars::sys::UID)
                .map(|uid| format!("/run/user/{uid}"))
        })
        .unwrap_or_else(|_| "/tmp".to_string());

    let mut discovered = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().canonicalize()
                && path.extension().and_then(|s| s.to_str()) == Some("sock")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                discovered.push(name.to_string());
            }
        }
    }

    let count = discovered.len();
    let (status, message) = if discovered.is_empty() {
        (HealthStatus::Warning, "No services discovered".to_string())
    } else {
        (HealthStatus::Ok, format!("Discovered {count} service(s)"))
    };

    HealthCheck {
        name: "Ecosystem Services",
        status,
        message,
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "runtime_dir": runtime_dir,
            "discovered_services": discovered,
            "note": "Services discovered via Unix socket capability discovery"
        })),
    }
}

/// Check Unix socket health
pub fn check_unix_socket() -> HealthCheck {
    let start = Instant::now();

    let socket_path =
        universal_constants::network::get_socket_path(universal_constants::identity::PRIMAL_ID)
            .to_string_lossy()
            .into_owned();

    HealthCheck {
        name: "Unix Socket",
        status: HealthStatus::Ok,
        message: "Configuration OK".to_string(),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "socket_path": socket_path,
            "note": "Socket created on server start",
        })),
    }
}

/// Check RPC server configuration
pub fn check_rpc_server() -> HealthCheck {
    let start = Instant::now();

    let socket_path =
        universal_constants::network::get_socket_path(universal_constants::identity::PRIMAL_ID)
            .to_string_lossy()
            .into_owned();

    HealthCheck {
        name: "RPC Server",
        status: HealthStatus::Ok,
        message: format!("Will bind to socket {socket_path}"),
        duration_ms: start.elapsed().as_millis() as u64,
        details: Some(serde_json::json!({
            "socket_path": socket_path,
            "protocol": "JSON-RPC 2.0 + tarpc",
            "note": "Server not running in doctor mode",
        })),
    }
}
