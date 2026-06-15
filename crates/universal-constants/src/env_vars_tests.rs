// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;

#[test]
fn flat_reexports_match_original_values() {
    assert_eq!(BIND_ADDRESS, "MCP_BIND_ADDRESS");
    assert_eq!(WEBSOCKET_PORT, "MCP_WEBSOCKET_PORT");
    assert_eq!(HTTP_PORT, "MCP_HTTP_PORT");
    assert_eq!(ADMIN_PORT, "MCP_ADMIN_PORT");
    assert_eq!(METRICS_PORT, "MCP_METRICS_PORT");
    assert_eq!(MAX_CONNECTIONS, "MAX_CONNECTIONS");
    assert_eq!(CONNECTION_TIMEOUT, "MCP_CONNECTION_TIMEOUT");
    assert_eq!(REQUEST_TIMEOUT, "REQUEST_TIMEOUT");
    assert_eq!(OPERATION_TIMEOUT, "OPERATION_TIMEOUT");
    assert_eq!(DATABASE_TIMEOUT, "DATABASE_TIMEOUT");
    assert_eq!(HEARTBEAT_INTERVAL, "SERVICE_MESH_HEARTBEAT_INTERVAL");
    assert_eq!(INITIAL_DELAY, "SERVICE_MESH_INITIAL_DELAY_MS");
    assert_eq!(MAX_MESSAGE_SIZE, "MCP_MAX_MESSAGE_SIZE");
    assert_eq!(BUFFER_SIZE, "BUFFER_SIZE");
    assert_eq!(SERVICE_MESH_MAX_SERVICES, "SERVICE_MESH_MAX_SERVICES");
    assert_eq!(ECOSYSTEM_REGISTRATION_URL, "ECOSYSTEM_REGISTRATION_URL");
    assert_eq!(ECOSYSTEM_HEALTH_URL, "ECOSYSTEM_HEALTH_URL");
    assert_eq!(ECOSYSTEM_METRICS_URL, "ECOSYSTEM_METRICS_URL");
    assert_eq!(DEBUG_MODE, "SQUIRREL_DEBUG");
    assert_eq!(VERBOSE_LOGGING, "SQUIRREL_VERBOSE");
    assert_eq!(LOG_LEVEL, "RUST_LOG");
}

#[test]
fn squirrel_module_constants() {
    assert_eq!(squirrel::SOCKET, "SQUIRREL_SOCKET");
    assert_eq!(squirrel::FAMILY_ID, "SQUIRREL_FAMILY_ID");
    assert_eq!(squirrel::NODE_ID, "SQUIRREL_NODE_ID");
    assert_eq!(squirrel::PORT, "SQUIRREL_PORT");
    assert_eq!(squirrel::BIND, "SQUIRREL_BIND");
}

#[test]
fn ecosystem_module_constants() {
    assert_eq!(ecosystem::BIOMEOS_FAMILY_ID, "BIOMEOS_FAMILY_ID");
    assert_eq!(ecosystem::FAMILY_ID, "FAMILY_ID");
    assert_eq!(ecosystem::BIOMEOS_SOCKET_PATH, "BIOMEOS_SOCKET_PATH");
    assert_eq!(ecosystem::NEURAL_API_SOCKET, "NEURAL_API_SOCKET");
}

#[test]
fn ai_module_constants() {
    assert_eq!(ai::PROVIDER_SOCKETS, "AI_PROVIDER_SOCKETS");
    assert_eq!(ai::openai::API_KEY, "OPENAI_API_KEY");
    assert_eq!(ai::anthropic::API_KEY, "ANTHROPIC_API_KEY");
    assert_eq!(ai::ollama::ENDPOINT, "OLLAMA_ENDPOINT");
    assert_eq!(ai::gemini::API_KEY, "GEMINI_API_KEY");
}

#[test]
fn mcp_module_constants() {
    assert_eq!(mcp::ENV, "MCP_ENV");
    assert_eq!(mcp::SERVER_URL, "MCP_SERVER_URL");
    assert_eq!(mcp::TIMEOUT_MS, "MCP_TIMEOUT_MS");
    assert_eq!(mcp::client::HOST, "MCP_CLIENT_HOST");
    assert_eq!(mcp::cli::HOST, "CLI_MCP_HOST");
}

#[test]
fn security_module_constants() {
    assert_eq!(security::ENDPOINT, "SECURITY_ENDPOINT");
    assert_eq!(security::JWT_SECRET, "JWT_SECRET");
    assert_eq!(security::TLS_CERT_PATH, "TLS_CERT_PATH");
}

#[test]
fn discovery_module_constants() {
    assert_eq!(discovery::SOCKET, "DISCOVERY_SOCKET");
    assert_eq!(
        discovery::CAPABILITY_REGISTRY_SOCKET,
        "CAPABILITY_REGISTRY_SOCKET"
    );
}

#[test]
fn primals_module_constants() {
    assert_eq!(primals::BEARDOG_ENDPOINT, "BEARDOG_ENDPOINT");
    assert_eq!(primals::SONGBIRD_ENDPOINT, "SONGBIRD_ENDPOINT");
    assert_eq!(primals::NESTGATE_ENDPOINT, "NESTGATE_ENDPOINT");
    assert_eq!(primals::TOADSTOOL_ENDPOINT, "TOADSTOOL_ENDPOINT");
}

#[test]
fn songbird_federation_env_vars() {
    assert_eq!(
        primals::SONGBIRD_FEDERATION_ENABLED,
        "SONGBIRD_FEDERATION_ENABLED"
    );
    assert_eq!(
        primals::SONGBIRD_FEDERATION_PORT,
        "SONGBIRD_FEDERATION_PORT"
    );
    assert_eq!(
        primals::SONGBIRD_FEDERATION_BIND,
        "SONGBIRD_FEDERATION_BIND"
    );
    assert_eq!(primals::SONGBIRD_PEERS, "SONGBIRD_PEERS");
    assert_eq!(
        primals::SONGBIRD_SERVICE_CONFIG_PATH,
        "SONGBIRD_SERVICE_CONFIG_PATH"
    );
    assert_eq!(federation::ENABLED, "FEDERATION_ENABLED");
}

#[test]
fn btsp_trust_env_vars() {
    assert_eq!(btsp::CAPABILITY_SOCKET, "BTSP_CAPABILITY_SOCKET");
    assert_eq!(btsp::PROVIDER_SOCKET, "BTSP_PROVIDER_SOCKET");
    assert_eq!(btsp::HANDSHAKE_TIMEOUT_MS, "BTSP_HANDSHAKE_TIMEOUT_MS");
    assert_eq!(btsp::BIRDSONG_KEY_LABEL, "BTSP_BIRDSONG_KEY_LABEL");
    assert_eq!(btsp::LINEAGE_ROOT_PREFIX, "BTSP_LINEAGE_ROOT_PREFIX");
    assert_eq!(btsp::LINEAGE_MAX_DEPTH, "BTSP_LINEAGE_MAX_DEPTH");
}
