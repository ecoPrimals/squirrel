// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Environment Variable Names — Single Source of Truth
//!
//! All environment variable names used throughout the Squirrel system.
//! Using named constants prevents typos, enables refactoring, and makes
//! it trivial to audit which env vars the system reads.
//!
//! # Organization
//!
//! Variables are organized by domain. Each domain module re-exports its
//! constants so consumers can use either:
//! ```ignore
//! use universal_constants::env_vars::squirrel::SOCKET;
//! // or via the flat re-export:
//! use universal_constants::env_vars;
//! ```

// ============================================================================
// Squirrel-specific
// ============================================================================

/// Squirrel primal env vars
pub mod squirrel {
    /// Override UDS socket path (`--socket` CLI equivalent)
    pub const SOCKET: &str = "SQUIRREL_SOCKET";
    /// Family ID for multi-instance deployments
    pub const FAMILY_ID: &str = "SQUIRREL_FAMILY_ID";
    /// Node identifier
    pub const NODE_ID: &str = "SQUIRREL_NODE_ID";
    /// TCP port for JSON-RPC
    pub const PORT: &str = "SQUIRREL_PORT";
    /// Alias for PORT (legacy)
    pub const SERVER_PORT: &str = "SQUIRREL_SERVER_PORT";
    /// TCP bind address
    pub const BIND: &str = "SQUIRREL_BIND";
    /// Bind address (legacy alias)
    pub const BIND_ADDRESS: &str = "SQUIRREL_BIND_ADDRESS";
    /// Host (legacy)
    pub const HOST: &str = "SQUIRREL_HOST";
    /// IPC host (legacy)
    pub const IPC_HOST: &str = "SQUIRREL_IPC_HOST";
    /// HTTP port
    pub const HTTP_PORT: &str = "SQUIRREL_HTTP_PORT";
    /// WebSocket port
    pub const WEBSOCKET_PORT: &str = "SQUIRREL_WEBSOCKET_PORT";
    /// gRPC port
    pub const GRPC_PORT: &str = "SQUIRREL_GRPC_PORT";
    /// Daemonize flag
    pub const DAEMON: &str = "SQUIRREL_DAEMON";
    /// Internal flag: child is already daemonized
    pub const DAEMONIZED: &str = "SQUIRREL_DAEMONIZED";
    /// Config file path override
    pub const CONFIG: &str = "SQUIRREL_CONFIG";
    /// Environment mode (dev/staging/prod)
    pub const ENV: &str = "SQUIRREL_ENV";
    /// Log level override
    pub const LOG_LEVEL: &str = "SQUIRREL_LOG_LEVEL";
    /// JSON logging format
    pub const LOG_JSON: &str = "SQUIRREL_LOG_JSON";
    /// Default AI provider name
    pub const DEFAULT_AI_PROVIDER: &str = "SQUIRREL_DEFAULT_AI_PROVIDER";
    /// AI config path
    pub const AI_CONFIG: &str = "SQUIRREL_AI_CONFIG";
    /// Enable AI subsystem
    pub const AI_ENABLED: &str = "SQUIRREL_AI_ENABLED";
    /// AI logging
    pub const AI_ENABLE_LOGGING: &str = "SQUIRREL_AI_ENABLE_LOGGING";
    /// AI inference timeout (seconds)
    pub const AI_INFERENCE_TIMEOUT_SECS: &str = "SQUIRREL_AI_INFERENCE_TIMEOUT_SECS";
    /// AI max retries
    pub const AI_MAX_RETRIES: &str = "SQUIRREL_AI_MAX_RETRIES";
    /// AI request timeout
    pub const AI_REQUEST_TIMEOUT: &str = "SQUIRREL_AI_REQUEST_TIMEOUT";
    /// MCP endpoint
    pub const MCP_ENDPOINT: &str = "SQUIRREL_MCP_ENDPOINT";
    /// Plugin directories
    pub const PLUGIN_DIRS: &str = "SQUIRREL_PLUGIN_DIRS";
    /// Plugin path
    pub const PLUGIN_PATH: &str = "SQUIRREL_PLUGIN_PATH";
    /// Plugin load timeout (seconds)
    pub const PLUGIN_LOAD_TIMEOUT_SECS: &str = "SQUIRREL_PLUGIN_LOAD_TIMEOUT_SECS";
    /// JWT secret
    pub const JWT_SECRET: &str = "SQUIRREL_JWT_SECRET";
    /// Trust domain for mTLS/SPIFFE
    pub const TRUST_DOMAIN: &str = "SQUIRREL_TRUST_DOMAIN";
    /// Rate limit whitelist
    pub const RATE_LIMIT_WHITELIST: &str = "SQUIRREL_RATE_LIMIT_WHITELIST";
    /// Connection timeout (seconds)
    pub const CONNECTION_TIMEOUT_SECS: &str = "SQUIRREL_CONNECTION_TIMEOUT_SECS";
    /// Request timeout (seconds)
    pub const REQUEST_TIMEOUT_SECS: &str = "SQUIRREL_REQUEST_TIMEOUT_SECS";
    /// Operation timeout (seconds)
    pub const OPERATION_TIMEOUT_SECS: &str = "SQUIRREL_OPERATION_TIMEOUT_SECS";
    /// Database timeout (seconds)
    pub const DATABASE_TIMEOUT_SECS: &str = "SQUIRREL_DATABASE_TIMEOUT_SECS";
    /// Discovery timeout (seconds)
    pub const DISCOVERY_TIMEOUT_SECS: &str = "SQUIRREL_DISCOVERY_TIMEOUT_SECS";
    /// Health check timeout (seconds)
    pub const HEALTH_CHECK_TIMEOUT_SECS: &str = "SQUIRREL_HEALTH_CHECK_TIMEOUT_SECS";
    /// Heartbeat interval (seconds)
    pub const HEARTBEAT_INTERVAL_SECS: &str = "SQUIRREL_HEARTBEAT_INTERVAL_SECS";
    /// Session timeout (seconds)
    pub const SESSION_TIMEOUT_SECS: &str = "SQUIRREL_SESSION_TIMEOUT_SECS";
    /// Registry socket for discovery
    pub const REGISTRY_SOCKET: &str = "SQUIRREL_REGISTRY_SOCKET";
    /// Ecosystem IPC service
    pub const ECOSYSTEM_IPC_SERVICE: &str = "SQUIRREL_ECOSYSTEM_IPC_SERVICE";
    /// Instance capacity
    pub const INSTANCE_CAPACITY: &str = "SQUIRREL_INSTANCE_CAPACITY";
    /// IPC retry base delay (ms)
    pub const RETRY_BASE_DELAY_MS: &str = "SQUIRREL_RETRY_BASE_DELAY_MS";
    /// IPC retry max attempts
    pub const RETRY_MAX_ATTEMPTS: &str = "SQUIRREL_RETRY_MAX_ATTEMPTS";
    /// IPC retry max delay (ms)
    pub const RETRY_MAX_DELAY_MS: &str = "SQUIRREL_RETRY_MAX_DELAY_MS";
    /// Resource: CPU
    pub const RESOURCE_CPU: &str = "SQUIRREL_RESOURCE_CPU";
    /// Resource: GPU
    pub const RESOURCE_GPU: &str = "SQUIRREL_RESOURCE_GPU";
    /// Resource: memory
    pub const RESOURCE_MEMORY: &str = "SQUIRREL_RESOURCE_MEMORY";
    /// Resource: network
    pub const RESOURCE_NETWORK: &str = "SQUIRREL_RESOURCE_NETWORK";
    /// Resource: storage
    pub const RESOURCE_STORAGE: &str = "SQUIRREL_RESOURCE_STORAGE";
}

// ============================================================================
// Ecosystem / biomeOS
// ============================================================================

/// Ecosystem orchestration env vars
pub mod ecosystem {
    /// biomeOS family ID
    pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
    /// Generic family ID (lowest priority)
    pub const FAMILY_ID: &str = "FAMILY_ID";
    /// biomeOS socket path (legacy)
    pub const BIOMEOS_SOCKET_PATH: &str = "BIOMEOS_SOCKET_PATH";
    /// biomeOS socket (discovery)
    pub const BIOMEOS_SOCKET: &str = "BIOMEOS_SOCKET";
    /// biomeOS insecure mode flag
    pub const BIOMEOS_INSECURE: &str = "BIOMEOS_INSECURE";
    /// biomeOS endpoint
    pub const BIOMEOS_ENDPOINT: &str = "BIOMEOS_ENDPOINT";
    /// biomeOS port
    pub const BIOMEOS_PORT: &str = "BIOMEOS_PORT";
    /// biomeOS UI endpoint
    pub const BIOMEOS_UI_ENDPOINT: &str = "BIOMEOS_UI_ENDPOINT";
    /// biomeOS websocket URL
    pub const BIOMEOS_WEBSOCKET_URL: &str = "BIOMEOS_WEBSOCKET_URL";
    /// Ecosystem endpoint (capability-first)
    pub const ECOSYSTEM_ENDPOINT: &str = "ECOSYSTEM_ENDPOINT";
    /// Ecosystem port
    pub const ECOSYSTEM_PORT: &str = "ECOSYSTEM_PORT";
    /// Ecosystem orchestrator socket
    pub const ECOSYSTEM_ORCHESTRATOR_SOCKET: &str = "ECOSYSTEM_ORCHESTRATOR_SOCKET";
    /// Ecosystem service mesh endpoint
    pub const ECOSYSTEM_SERVICE_MESH_ENDPOINT: &str = "ECOSYSTEM_SERVICE_MESH_ENDPOINT";
    /// Ecosystem service timeout (ms)
    pub const ECOSYSTEM_SERVICE_TIMEOUT_MS: &str = "ECOSYSTEM_SERVICE_TIMEOUT_MS";
    /// Ecosystem websocket URL (capability-first)
    pub const ECOSYSTEM_WEBSOCKET_URL: &str = "ECOSYSTEM_WEBSOCKET_URL";
    /// Ecosystem router service ID
    pub const ECOSYSTEM_ROUTER_SERVICE_ID: &str = "ECOSYSTEM_ROUTER_SERVICE_ID";
    /// Neural API socket
    pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";
    /// Node ID
    pub const NODE_ID: &str = "NODE_ID";
    /// Biome ID
    pub const BIOME_ID: &str = "BIOME_ID";
}

// ============================================================================
// AI / Inference Providers
// ============================================================================

/// AI provider env vars
pub mod ai {
    /// Default AI provider
    pub const DEFAULT_PROVIDER: &str = "AI_DEFAULT_PROVIDER";
    /// HTTP provider config (JSON)
    pub const HTTP_PROVIDERS: &str = "AI_HTTP_PROVIDERS";
    /// AI provider socket paths (comma-separated)
    pub const PROVIDER_SOCKETS: &str = "AI_PROVIDER_SOCKETS";
    /// AI service host
    pub const SERVICE_HOST: &str = "AI_SERVICE_HOST";
    /// AI service name
    pub const SERVICE_NAME: &str = "AI_SERVICE_NAME";
    /// AI request timeout (ms)
    pub const REQUEST_TIMEOUT_MS: &str = "AI_REQUEST_TIMEOUT_MS";
    /// AI intelligence interval (seconds)
    pub const INTELLIGENCE_INTERVAL_SECS: &str = "AI_INTELLIGENCE_INTERVAL_SECS";
    /// Inference endpoint (generic, primal-agnostic)
    pub const INFERENCE_ENDPOINT: &str = "INFERENCE_ENDPOINT";
    /// AI inference endpoint (prefixed variant)
    pub const AI_INFERENCE_ENDPOINT: &str = "AI_INFERENCE_ENDPOINT";

    /// `OpenAI` provider
    pub mod openai {
        /// API key
        pub const API_KEY: &str = "OPENAI_API_KEY";
        /// Base URL
        pub const BASE_URL: &str = "OPENAI_BASE_URL";
        /// API base (alias)
        pub const API_BASE: &str = "OPENAI_API_BASE";
        /// API base URL (alias)
        pub const API_BASE_URL: &str = "OPENAI_API_BASE_URL";
        /// Endpoint (alias)
        pub const ENDPOINT: &str = "OPENAI_ENDPOINT";
        /// Default model
        pub const DEFAULT_MODEL: &str = "OPENAI_DEFAULT_MODEL";
    }

    /// Anthropic provider
    pub mod anthropic {
        /// API key
        pub const API_KEY: &str = "ANTHROPIC_API_KEY";
        /// Base URL
        pub const BASE_URL: &str = "ANTHROPIC_BASE_URL";
        /// API base (alias)
        pub const API_BASE: &str = "ANTHROPIC_API_BASE";
        /// API base URL (alias)
        pub const API_BASE_URL: &str = "ANTHROPIC_API_BASE_URL";
        /// Endpoint (alias)
        pub const ENDPOINT: &str = "ANTHROPIC_ENDPOINT";
        /// Default model
        pub const DEFAULT_MODEL: &str = "ANTHROPIC_DEFAULT_MODEL";
    }

    /// Ollama provider (local)
    pub mod ollama {
        /// Endpoint URL
        pub const ENDPOINT: &str = "OLLAMA_ENDPOINT";
        /// Host
        pub const HOST: &str = "OLLAMA_HOST";
        /// Port
        pub const PORT: &str = "OLLAMA_PORT";
        /// Full URL (alias)
        pub const URL: &str = "OLLAMA_URL";
        /// Default model
        pub const DEFAULT_MODEL: &str = "OLLAMA_DEFAULT_MODEL";
    }

    /// Gemini provider
    pub mod gemini {
        /// API key
        pub const API_KEY: &str = "GEMINI_API_KEY";
        /// Base URL
        pub const BASE_URL: &str = "GEMINI_BASE_URL";
        /// API base (alias)
        pub const API_BASE: &str = "GEMINI_API_BASE";
        /// Default model
        pub const DEFAULT_MODEL: &str = "GEMINI_DEFAULT_MODEL";
    }

    /// `HuggingFace` provider
    pub mod huggingface {
        /// API key
        pub const API_KEY: &str = "HUGGINGFACE_API_KEY";
    }

    /// Local AI (llama.cpp etc.)
    pub mod local {
        /// Endpoint
        pub const ENDPOINT: &str = "LOCAL_AI_ENDPOINT";
        /// Host
        pub const HOST: &str = "LOCAL_AI_HOST";
        /// Port
        pub const PORT: &str = "LOCAL_AI_PORT";
        /// Full URL
        pub const URL: &str = "LOCAL_AI_URL";
        /// Default model
        pub const DEFAULT_MODEL: &str = "LOCAL_AI_DEFAULT_MODEL";
        /// llama.cpp endpoint
        pub const LLAMACPP_ENDPOINT: &str = "LLAMACPP_ENDPOINT";
    }
}

// ============================================================================
// MCP Protocol
// ============================================================================

/// MCP protocol env vars
pub mod mcp {
    /// MCP environment (dev/staging/prod)
    pub const ENV: &str = "MCP_ENV";
    /// MCP environment (alias)
    pub const ENVIRONMENT: &str = "MCP_ENVIRONMENT";
    /// MCP host
    pub const HOST: &str = "MCP_HOST";
    /// MCP port
    pub const PORT: &str = "MCP_PORT";
    /// MCP server URL
    pub const SERVER_URL: &str = "MCP_SERVER_URL";
    /// MCP server port
    pub const SERVER_PORT: &str = "MCP_SERVER_PORT";
    /// MCP server host
    pub const SERVER_HOST: &str = "MCP_SERVER_HOST";
    /// MCP server endpoint
    pub const SERVER_ENDPOINT: &str = "MCP_SERVER_ENDPOINT";
    /// MCP endpoint (generic)
    pub const ENDPOINT: &str = "MCP_ENDPOINT";
    /// MCP timeout (ms)
    pub const TIMEOUT_MS: &str = "MCP_TIMEOUT_MS";
    /// MCP request timeout (ms)
    pub const REQUEST_TIMEOUT_MS: &str = "MCP_REQUEST_TIMEOUT_MS";
    /// MCP connection timeout (seconds)
    pub const CONNECTION_TIMEOUT_SECS: &str = "MCP_CONNECTION_TIMEOUT_SECS";
    /// MCP max message size
    pub const MAX_MESSAGE_SIZE: &str = "MCP_MAX_MESSAGE_SIZE";
    /// MCP max connections
    pub const MAX_CONNECTIONS: &str = "MCP_MAX_CONNECTIONS";
    /// MCP protocol version
    pub const PROTOCOL_VERSION: &str = "MCP_PROTOCOL_VERSION";
    /// MCP max reconnect attempts
    pub const MAX_RECONNECT_ATTEMPTS: &str = "MCP_MAX_RECONNECT_ATTEMPTS";
    /// MCP reconnect delay (ms)
    pub const RECONNECT_DELAY_MS: &str = "MCP_RECONNECT_DELAY_MS";
    /// MCP default model
    pub const DEFAULT_MODEL: &str = "MCP_DEFAULT_MODEL";
    /// MCP debug mode
    pub const DEBUG: &str = "MCP_DEBUG";
    /// MCP CORS origins
    pub const CORS_ORIGINS: &str = "MCP_CORS_ORIGINS";
    /// MCP heartbeat interval (seconds)
    pub const HEARTBEAT_INTERVAL_SECS: &str = "MCP_HEARTBEAT_INTERVAL_SECS";
    /// MCP coordination interval (seconds)
    pub const COORDINATION_INTERVAL_SECS: &str = "MCP_COORDINATION_INTERVAL_SECS";

    /// MCP client-specific vars
    pub mod client {
        /// Client host
        pub const HOST: &str = "MCP_CLIENT_HOST";
        /// Client port
        pub const PORT: &str = "MCP_CLIENT_PORT";
        /// Client connect timeout (seconds)
        pub const CONNECT_TIMEOUT_SECS: &str = "MCP_CLIENT_CONNECT_TIMEOUT_SECS";
        /// Client request timeout (seconds)
        pub const REQUEST_TIMEOUT_SECS: &str = "MCP_CLIENT_REQUEST_TIMEOUT_SECS";
        /// Client max retries
        pub const MAX_RETRIES: &str = "MCP_CLIENT_MAX_RETRIES";
    }

    /// CLI-specific MCP vars
    pub mod cli {
        /// CLI MCP host
        pub const HOST: &str = "CLI_MCP_HOST";
        /// CLI MCP port
        pub const PORT: &str = "CLI_MCP_PORT";
        /// CLI output format
        pub const OUTPUT_FORMAT: &str = "CLI_OUTPUT_FORMAT";
    }
}

// ============================================================================
// Network & Connection
// ============================================================================

/// Network env vars (flat names shared across subsystems)
pub mod network {
    /// Bind address
    pub const BIND_ADDRESS: &str = "MCP_BIND_ADDRESS";
    /// WebSocket port
    pub const WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";
    /// HTTP port
    pub const HTTP_PORT: &str = "MCP_HTTP_PORT";
    /// Admin port
    pub const ADMIN_PORT: &str = "MCP_ADMIN_PORT";
    /// Metrics port
    pub const METRICS_PORT: &str = "MCP_METRICS_PORT";
    /// Max connections
    pub const MAX_CONNECTIONS: &str = "MAX_CONNECTIONS";
    /// Generic port
    pub const PORT: &str = "PORT";
    /// Generic bind addr
    pub const BIND_ADDR: &str = "BIND_ADDR";
    /// Server bind address
    pub const SERVER_BIND_ADDRESS: &str = "SERVER_BIND_ADDRESS";
    /// Server port
    pub const SERVER_PORT: &str = "SERVER_PORT";
    /// Network host
    pub const NETWORK_HOST: &str = "NETWORK_HOST";
    /// Network port
    pub const NETWORK_PORT: &str = "NETWORK_PORT";
    /// Network connection timeout (ms)
    pub const NETWORK_CONNECTION_TIMEOUT_MS: &str = "NETWORK_CONNECTION_TIMEOUT_MS";
    /// Network read timeout (ms)
    pub const NETWORK_READ_TIMEOUT_MS: &str = "NETWORK_READ_TIMEOUT_MS";
    /// Network write timeout (ms)
    pub const NETWORK_WRITE_TIMEOUT_MS: &str = "NETWORK_WRITE_TIMEOUT_MS";
    /// Network max connections
    pub const NETWORK_MAX_CONNECTIONS: &str = "NETWORK_MAX_CONNECTIONS";
    /// Network HTTP socket
    pub const NETWORK_HTTP_SOCKET: &str = "NETWORK_HTTP_SOCKET";
    /// Service host
    pub const SERVICE_HOST: &str = "SERVICE_HOST";
    /// Service port
    pub const SERVICE_PORT: &str = "SERVICE_PORT";
    /// Service address
    pub const SERVICE_ADDRESS: &str = "SERVICE_ADDRESS";
    /// Service IP
    pub const SERVICE_IP: &str = "SERVICE_IP";
    /// Dev bind address
    pub const DEV_BIND_ADDRESS: &str = "DEV_BIND_ADDRESS";
    /// Dev server host
    pub const DEV_SERVER_HOST: &str = "DEV_SERVER_HOST";
    /// Storage capability endpoint (capability-first naming)
    pub const STORAGE_ENDPOINT: &str = "STORAGE_ENDPOINT";
    /// Storage capability port
    pub const STORAGE_PORT: &str = "STORAGE_PORT";
    /// Storage service port
    pub const STORAGE_SERVICE_PORT: &str = "STORAGE_SERVICE_PORT";
    /// Security capability endpoint (capability-first naming)
    pub const SECURITY_ENDPOINT: &str = "SECURITY_ENDPOINT";
    /// Security capability port
    pub const SECURITY_PORT: &str = "SECURITY_PORT";
    /// Security service port
    pub const SECURITY_SERVICE_PORT: &str = "SECURITY_SERVICE_PORT";
    /// Service mesh endpoint (capability-first naming)
    pub const SERVICE_MESH_ENDPOINT: &str = "SERVICE_MESH_ENDPOINT";
    /// Service mesh port
    pub const SERVICE_MESH_PORT: &str = "SERVICE_MESH_PORT";
}

// ============================================================================
// Timeouts
// ============================================================================

/// Timeout env vars
pub mod timeout {
    /// Connection timeout (generic)
    pub const CONNECTION: &str = "MCP_CONNECTION_TIMEOUT";
    /// Request timeout (generic)
    pub const REQUEST: &str = "REQUEST_TIMEOUT";
    /// Operation timeout
    pub const OPERATION: &str = "OPERATION_TIMEOUT";
    /// Database timeout
    pub const DATABASE: &str = "DATABASE_TIMEOUT";
    /// Heartbeat interval (service mesh)
    pub const HEARTBEAT_INTERVAL: &str = "SERVICE_MESH_HEARTBEAT_INTERVAL";
    /// Initial delay (service mesh)
    pub const INITIAL_DELAY: &str = "SERVICE_MESH_INITIAL_DELAY_MS";
}

// ============================================================================
// Discovery & Service Mesh
// ============================================================================

/// Discovery env vars
pub mod discovery {
    /// Discovery socket
    pub const SOCKET: &str = "DISCOVERY_SOCKET";
    /// Discovery endpoint
    pub const ENDPOINT: &str = "DISCOVERY_ENDPOINT";
    /// Discovery port
    pub const PORT: &str = "DISCOVERY_PORT";
    /// Registration endpoint
    pub const REGISTRATION_ENDPOINT: &str = "REGISTRATION_ENDPOINT";
    /// Discovery auth token
    pub const AUTH_TOKEN: &str = "DISCOVERY_AUTH_TOKEN";
    /// Discovery batch size
    pub const BATCH_SIZE: &str = "DISCOVERY_BATCH_SIZE";
    /// Discovery flush interval
    pub const FLUSH_INTERVAL: &str = "DISCOVERY_FLUSH_INTERVAL";
    /// Socket scan directory
    pub const SOCKET_SCAN_DIR: &str = "SOCKET_SCAN_DIR";
    /// Primal auto-discovery flag
    pub const PRIMAL_AUTO_DISCOVERY: &str = "PRIMAL_AUTO_DISCOVERY";
    /// Capability registry socket
    pub const CAPABILITY_REGISTRY_SOCKET: &str = "CAPABILITY_REGISTRY_SOCKET";
    /// Service mesh host
    pub const SERVICE_MESH_HOST: &str = "SERVICE_MESH_HOST";
    /// Service mesh port
    pub const SERVICE_MESH_PORT: &str = "SERVICE_MESH_PORT";
    /// Service mesh endpoint
    pub const SERVICE_MESH_ENDPOINT: &str = "SERVICE_MESH_ENDPOINT";
    /// Service discovery host
    pub const SERVICE_DISCOVERY_HOST: &str = "SERVICE_DISCOVERY_HOST";
    /// Service discovery port
    pub const SERVICE_DISCOVERY_PORT: &str = "SERVICE_DISCOVERY_PORT";
    /// Service discovery ports (multi)
    pub const SERVICE_DISCOVERY_PORTS: &str = "SERVICE_DISCOVERY_PORTS";
    /// Service discovery URL
    pub const SERVICE_DISCOVERY_URL: &str = "SERVICE_DISCOVERY_URL";
    /// Service registry endpoint
    pub const SERVICE_REGISTRY_ENDPOINT: &str = "SERVICE_REGISTRY_ENDPOINT";
    /// Service registry type
    pub const SERVICE_REGISTRY_TYPE: &str = "SERVICE_REGISTRY_TYPE";
    /// Consul HTTP address
    pub const CONSUL_HTTP_ADDR: &str = "CONSUL_HTTP_ADDR";
}

// ============================================================================
// Security & Auth
// ============================================================================

/// Security env vars
pub mod security {
    /// Security endpoint
    pub const ENDPOINT: &str = "SECURITY_ENDPOINT";
    /// Security host
    pub const HOST: &str = "SECURITY_HOST";
    /// Security port
    pub const PORT: &str = "SECURITY_PORT";
    /// Security socket
    pub const SOCKET: &str = "SECURITY_SOCKET";
    /// Security service endpoint
    pub const SERVICE_ENDPOINT: &str = "SECURITY_SERVICE_ENDPOINT";
    /// Security service host
    pub const SERVICE_HOST: &str = "SECURITY_SERVICE_HOST";
    /// Security service name
    pub const SERVICE_NAME: &str = "SECURITY_SERVICE_NAME";
    /// Security auth service endpoint
    pub const AUTH_SERVICE_ENDPOINT: &str = "SECURITY_AUTH_SERVICE_ENDPOINT";
    /// Security authentication port
    pub const AUTHENTICATION_PORT: &str = "SECURITY_AUTHENTICATION_PORT";
    /// Security token file
    pub const TOKEN_FILE: &str = "SECURITY_TOKEN_FILE";
    /// Security trust domain
    pub const TRUST_DOMAIN: &str = "SECURITY_TRUST_DOMAIN";
    /// JWT secret
    pub const JWT_SECRET: &str = "JWT_SECRET";
    /// JWT key ID
    pub const JWT_KEY_ID: &str = "JWT_KEY_ID";
    /// JWT expiry (hours)
    pub const JWT_EXPIRY_HOURS: &str = "JWT_EXPIRY_HOURS";
    /// TLS cert path
    pub const TLS_CERT_PATH: &str = "TLS_CERT_PATH";
    /// TLS key path
    pub const TLS_KEY_PATH: &str = "TLS_KEY_PATH";
    /// CA cert path
    pub const CA_CERT_PATH: &str = "CA_CERT_PATH";
}

// ============================================================================
// External Primals (ecosystem peers)
// ============================================================================

/// External primal env vars
pub mod primals {
    /// `BearDog` endpoint
    pub const BEARDOG_ENDPOINT: &str = "BEARDOG_ENDPOINT";
    /// `BearDog` socket
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";
    /// `BearDog` family seed
    pub const BEARDOG_FAMILY_SEED: &str = "BEARDOG_FAMILY_SEED";
    /// `Songbird` endpoint
    pub const SONGBIRD_ENDPOINT: &str = "SONGBIRD_ENDPOINT";
    /// `Songbird` port
    pub const SONGBIRD_PORT: &str = "SONGBIRD_PORT";
    /// `Songbird` socket
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";
    /// `Songbird` auth token
    pub const SONGBIRD_AUTH_TOKEN: &str = "SONGBIRD_AUTH_TOKEN";
    /// `Songbird` batch size
    pub const SONGBIRD_BATCH_SIZE: &str = "SONGBIRD_BATCH_SIZE";
    /// `Songbird` flush interval
    pub const SONGBIRD_FLUSH_INTERVAL: &str = "SONGBIRD_FLUSH_INTERVAL";
    /// `NestGate` endpoint
    pub const NESTGATE_ENDPOINT: &str = "NESTGATE_ENDPOINT";
    /// `NestGate` port
    pub const NESTGATE_PORT: &str = "NESTGATE_PORT";
    /// `ToadStool` endpoint
    pub const TOADSTOOL_ENDPOINT: &str = "TOADSTOOL_ENDPOINT";
    /// `ToadStool` port
    pub const TOADSTOOL_PORT: &str = "TOADSTOOL_PORT";
    /// Crypto endpoint
    pub const CRYPTO_ENDPOINT: &str = "CRYPTO_ENDPOINT";
    /// Crypto signing endpoint
    pub const CRYPTO_SIGNING_ENDPOINT: &str = "CRYPTO_SIGNING_ENDPOINT";
}

// ============================================================================
// Primal Configuration (generic)
// ============================================================================

/// Generic primal env vars
pub mod primal {
    /// Primal socket (generic coordination)
    pub const SOCKET: &str = "PRIMAL_SOCKET";
    /// Primal name
    pub const NAME: &str = "PRIMAL_NAME";
    /// Primal type
    pub const TYPE: &str = "PRIMAL_TYPE";
    /// Primal endpoint
    pub const ENDPOINT: &str = "PRIMAL_ENDPOINT";
    /// Primal port
    pub const PORT: &str = "PRIMAL_PORT";
    /// Primal bind address
    pub const BIND_ADDRESS: &str = "PRIMAL_BIND_ADDRESS";
    /// Primal capabilities
    pub const CAPABILITIES: &str = "PRIMAL_CAPABILITIES";
    /// Primal max instances per type
    pub const MAX_INSTANCES_PER_TYPE: &str = "PRIMAL_MAX_INSTANCES_PER_TYPE";
    /// Primal max instances per user
    pub const MAX_INSTANCES_PER_USER: &str = "PRIMAL_MAX_INSTANCES_PER_USER";
    /// Primal port range start
    pub const PORT_RANGE_START: &str = "PRIMAL_PORT_RANGE_START";
    /// Primal port range end
    pub const PORT_RANGE_END: &str = "PRIMAL_PORT_RANGE_END";
}

// ============================================================================
// BTSP (Binary Transport Specification Protocol)
// ============================================================================

/// BTSP env vars
pub mod btsp {
    /// Capability socket for BTSP
    pub const CAPABILITY_SOCKET: &str = "BTSP_CAPABILITY_SOCKET";
    /// Provider socket for BTSP
    pub const PROVIDER_SOCKET: &str = "BTSP_PROVIDER_SOCKET";
    /// Handshake timeout (ms)
    pub const HANDSHAKE_TIMEOUT_MS: &str = "BTSP_HANDSHAKE_TIMEOUT_MS";
}

// ============================================================================
// Compute
// ============================================================================

/// Compute env vars
pub mod compute {
    /// Compute endpoint
    pub const ENDPOINT: &str = "COMPUTE_ENDPOINT";
    /// Compute port
    pub const PORT: &str = "COMPUTE_PORT";
    /// Compute provider type
    pub const PROVIDER_TYPE: &str = "COMPUTE_PROVIDER_TYPE";
    /// Compute service endpoint
    pub const SERVICE_ENDPOINT: &str = "COMPUTE_SERVICE_ENDPOINT";
    /// Compute service host
    pub const SERVICE_HOST: &str = "COMPUTE_SERVICE_HOST";
    /// Compute service name
    pub const SERVICE_NAME: &str = "COMPUTE_SERVICE_NAME";
    /// Compute service port
    pub const SERVICE_PORT: &str = "COMPUTE_SERVICE_PORT";
}

// ============================================================================
// Storage
// ============================================================================

/// Storage env vars
pub mod storage {
    /// Storage endpoint
    pub const ENDPOINT: &str = "STORAGE_ENDPOINT";
    /// Storage port
    pub const PORT: &str = "STORAGE_PORT";
    /// Storage service endpoint
    pub const SERVICE_ENDPOINT: &str = "STORAGE_SERVICE_ENDPOINT";
    /// Storage service host
    pub const SERVICE_HOST: &str = "STORAGE_SERVICE_HOST";
    /// Storage service name
    pub const SERVICE_NAME: &str = "STORAGE_SERVICE_NAME";
    /// Storage service port
    pub const SERVICE_PORT: &str = "STORAGE_SERVICE_PORT";
}

// ============================================================================
// Database
// ============================================================================

/// Database env vars
pub mod database {
    /// Database URL
    pub const URL: &str = "DATABASE_URL";
    /// Database URL (dev)
    pub const URL_DEV: &str = "DATABASE_URL_DEV";
    /// Database URL (staging)
    pub const URL_STAGING: &str = "DATABASE_URL_STAGING";
    /// Database port
    pub const PORT: &str = "DATABASE_PORT";
    /// Database max connections
    pub const MAX_CONNECTIONS: &str = "DATABASE_MAX_CONNECTIONS";
    /// Database timeout (seconds)
    pub const TIMEOUT_SECS: &str = "DATABASE_TIMEOUT_SECS";
    /// DB max connections (alias)
    pub const DB_MAX_CONNECTIONS: &str = "DB_MAX_CONNECTIONS";
    /// DB timeout (alias)
    pub const DB_TIMEOUT: &str = "DB_TIMEOUT";
    /// Postgres port
    pub const POSTGRES_PORT: &str = "POSTGRES_PORT";
}

// ============================================================================
// Monitoring & Logging
// ============================================================================

/// Monitoring env vars
pub mod monitoring {
    /// Monitoring enabled flag
    pub const ENABLED: &str = "MONITORING_ENABLED";
    /// Monitoring auth token
    pub const AUTH_TOKEN: &str = "MONITORING_AUTH_TOKEN";
    /// Monitoring batch size
    pub const BATCH_SIZE: &str = "MONITORING_BATCH_SIZE";
    /// Monitoring flush interval
    pub const FLUSH_INTERVAL: &str = "MONITORING_FLUSH_INTERVAL";
    /// Require monitoring provider
    pub const REQUIRE_PROVIDER: &str = "MONITORING_REQUIRE_PROVIDER";
    /// Metrics exporter endpoint
    pub const METRICS_EXPORTER_ENDPOINT: &str = "METRICS_EXPORTER_ENDPOINT";
    /// Metrics exporter port
    pub const METRICS_EXPORTER_PORT: &str = "METRICS_EXPORTER_PORT";
    /// Metrics port
    pub const METRICS_PORT: &str = "METRICS_PORT";
    /// Health check interval (seconds)
    pub const HEALTH_CHECK_INTERVAL_SECS: &str = "HEALTH_CHECK_INTERVAL_SECS";
}

/// Logging env vars
pub mod logging {
    /// Log level
    pub const LEVEL: &str = "LOG_LEVEL";
    /// Log include location
    pub const INCLUDE_LOCATION: &str = "LOG_INCLUDE_LOCATION";
    /// Log max entries
    pub const MAX_ENTRIES: &str = "LOG_MAX_ENTRIES";
    /// Log rotation size
    pub const ROTATION_SIZE: &str = "LOG_ROTATION_SIZE";
    /// Log send to host
    pub const SEND_TO_HOST: &str = "LOG_SEND_TO_HOST";
    /// Rust log filter
    pub const RUST_LOG: &str = "RUST_LOG";
}

// ============================================================================
// Performance / Sandbox
// ============================================================================

/// Performance tuning env vars
pub mod performance {
    /// Batch processor size
    pub const BATCH_PROCESSOR_SIZE: &str = "PERF_BATCH_PROCESSOR_SIZE";
    /// FS buffer size
    pub const FS_BUFFER_SIZE: &str = "PERF_FS_BUFFER_SIZE";
    /// Max plugin ID length
    pub const MAX_PLUGIN_ID_LENGTH: &str = "PERF_MAX_PLUGIN_ID_LENGTH";
    /// Session timeout (seconds)
    pub const SESSION_TIMEOUT_SECONDS: &str = "PERF_SESSION_TIMEOUT_SECONDS";
    /// String pool capacity
    pub const STRING_POOL_CAPACITY: &str = "PERF_STRING_POOL_CAPACITY";
}

/// Sandbox env vars
pub mod sandbox {
    /// CPU limit percent
    pub const CPU_LIMIT_PERCENT: &str = "SANDBOX_CPU_LIMIT_PERCENT";
    /// Execution timeout (seconds)
    pub const EXECUTION_TIMEOUT_SECONDS: &str = "SANDBOX_EXECUTION_TIMEOUT_SECONDS";
    /// File system access level
    pub const FILE_SYSTEM_ACCESS: &str = "SANDBOX_FILE_SYSTEM_ACCESS";
    /// Memory limit (MB)
    pub const MEMORY_LIMIT_MB: &str = "SANDBOX_MEMORY_LIMIT_MB";
    /// Network access level
    pub const NETWORK_ACCESS: &str = "SANDBOX_NETWORK_ACCESS";
    /// Security level
    pub const SECURITY_LEVEL: &str = "SANDBOX_SECURITY_LEVEL";
}

// ============================================================================
// HTTP Client
// ============================================================================

/// HTTP env vars
pub mod http {
    /// Default timeout (ms)
    pub const DEFAULT_TIMEOUT_MS: &str = "HTTP_DEFAULT_TIMEOUT_MS";
    /// Max redirects
    pub const MAX_REDIRECTS: &str = "HTTP_MAX_REDIRECTS";
    /// Max request size
    pub const MAX_REQUEST_SIZE: &str = "HTTP_MAX_REQUEST_SIZE";
    /// Max response size
    pub const MAX_RESPONSE_SIZE: &str = "HTTP_MAX_RESPONSE_SIZE";
    /// User agent
    pub const USER_AGENT: &str = "HTTP_USER_AGENT";
    /// HTTP capability socket
    pub const CAPABILITY_SOCKET: &str = "HTTP_CAPABILITY_SOCKET";
    /// Web UI URL override
    pub const WEB_UI_URL: &str = "WEB_UI_URL";
    /// Web UI port override
    pub const WEB_UI_PORT: &str = "WEB_UI_PORT";
}

// ============================================================================
// IPC Retry
// ============================================================================

/// IPC retry env vars
pub mod ipc {
    /// Retry base delay (ms)
    pub const RETRY_BASE_DELAY_MS: &str = "IPC_RETRY_BASE_DELAY_MS";
    /// Retry max attempts
    pub const RETRY_MAX_ATTEMPTS: &str = "IPC_RETRY_MAX_ATTEMPTS";
    /// Retry max delay (ms)
    pub const RETRY_MAX_DELAY_MS: &str = "IPC_RETRY_MAX_DELAY_MS";
}

// ============================================================================
// Deployment / Build
// ============================================================================

/// Deployment env vars
pub mod deploy {
    /// Deployment region
    pub const REGION: &str = "DEPLOYMENT_REGION";
    /// Availability zone
    pub const AVAILABILITY_ZONE: &str = "AVAILABILITY_ZONE";
    /// Data center
    pub const DATA_CENTER: &str = "DATA_CENTER";
    /// Node IP
    pub const NODE_IP: &str = "NODE_IP";
    /// Build timestamp
    pub const BUILD_TIMESTAMP: &str = "BUILD_TIMESTAMP";
    /// Git hash
    pub const GIT_HASH: &str = "GIT_HASH";
    /// Profile (release/debug)
    pub const PROFILE: &str = "PROFILE";
    /// Target triple
    pub const TARGET: &str = "TARGET";
    /// Environment name
    pub const ENVIRONMENT: &str = "ENVIRONMENT";
}

// ============================================================================
// Task Management
// ============================================================================

/// Task/compute delegation env vars
pub mod task {
    /// Task server endpoint
    pub const SERVER_ENDPOINT: &str = "TASK_SERVER_ENDPOINT";
    /// Task server socket
    pub const SERVER_SOCKET: &str = "TASK_SERVER_SOCKET";
}

// ============================================================================
// Session
// ============================================================================

/// Session env vars
pub mod session {
    /// Session timeout (seconds)
    pub const TIMEOUT_SECS: &str = "SESSION_TIMEOUT_SECS";
    /// Session max connections
    pub const MAX_CONNECTIONS: &str = "SESSION_MAX_CONNECTIONS";
    /// Average session duration (seconds)
    pub const AVERAGE_DURATION_SECS: &str = "AVERAGE_SESSION_DURATION_SECS";
    /// Context management interval (seconds)
    pub const CONTEXT_MANAGEMENT_INTERVAL_SECS: &str = "CONTEXT_MANAGEMENT_INTERVAL_SECS";
}

// ============================================================================
// Limits & Sizes (flat-name compat)
// ============================================================================

/// Size/limit env vars
pub mod limits {
    /// Maximum message size
    pub const MAX_MESSAGE_SIZE: &str = "MCP_MAX_MESSAGE_SIZE";
    /// Buffer size
    pub const BUFFER_SIZE: &str = "BUFFER_SIZE";
    /// Service mesh max services
    pub const SERVICE_MESH_MAX_SERVICES: &str = "SERVICE_MESH_MAX_SERVICES";
}

// ============================================================================
// Feature Flags
// ============================================================================

/// Feature flag env vars
pub mod flags {
    /// Debug mode
    pub const DEBUG_MODE: &str = "SQUIRREL_DEBUG";
    /// Verbose logging
    pub const VERBOSE_LOGGING: &str = "SQUIRREL_VERBOSE";
}

// ============================================================================
// System
// ============================================================================

/// System env vars
pub mod sys {
    /// Home directory
    pub const HOME: &str = "HOME";
    /// Hostname
    pub const HOSTNAME: &str = "HOSTNAME";
    /// Temp directory
    pub const TEMP: &str = "TEMP";
    /// User ID
    pub const UID: &str = "UID";
    /// XDG runtime directory
    pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
}

// ============================================================================
// Backward-compatible flat re-exports (old env_vars::BIND_ADDRESS style)
// ============================================================================

pub use limits::{BUFFER_SIZE, MAX_MESSAGE_SIZE, SERVICE_MESH_MAX_SERVICES};
pub use logging::RUST_LOG as LOG_LEVEL;
pub use network::{
    ADMIN_PORT, BIND_ADDRESS, HTTP_PORT, MAX_CONNECTIONS, METRICS_PORT, WEBSOCKET_PORT,
};
pub use timeout::{
    CONNECTION as CONNECTION_TIMEOUT, DATABASE as DATABASE_TIMEOUT, HEARTBEAT_INTERVAL,
    INITIAL_DELAY, OPERATION as OPERATION_TIMEOUT, REQUEST as REQUEST_TIMEOUT,
};

/// Ecosystem registration URL (capability-first; legacy `BIOMEOS_REGISTRATION_URL` read as fallback)
pub const ECOSYSTEM_REGISTRATION_URL: &str = "ECOSYSTEM_REGISTRATION_URL";
/// Ecosystem health URL (capability-first; legacy `BIOMEOS_HEALTH_URL` read as fallback)
pub const ECOSYSTEM_HEALTH_URL: &str = "ECOSYSTEM_HEALTH_URL";
/// Ecosystem metrics URL (capability-first; legacy `BIOMEOS_METRICS_URL` read as fallback)
pub const ECOSYSTEM_METRICS_URL: &str = "ECOSYSTEM_METRICS_URL";

/// Debug mode (flat re-export for backward compat)
pub const DEBUG_MODE: &str = "SQUIRREL_DEBUG";
/// Verbose logging (flat re-export for backward compat)
pub const VERBOSE_LOGGING: &str = "SQUIRREL_VERBOSE";

#[cfg(test)]
mod tests {
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
}
