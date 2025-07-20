//! Port Management Configuration for Songbird Integration
//!
//! This module provides centralized port allocation and management that can be
//! controlled by Songbird's service mesh orchestration.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::ops::Range;

/// Port allocation strategy for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortAllocationStrategy {
    /// Fixed ports (not recommended for production)
    Fixed(u16),
    /// Dynamic allocation from a range
    Dynamic { range: Range<u16> },
    /// Songbird-managed allocation
    SongbirdManaged,
    /// Environment-based allocation
    EnvironmentBased { env_var: String, default: u16 },
}

/// Port configuration for ecosystem services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Service identification
    pub service_name: String,
    /// Port allocation strategy
    pub allocation_strategy: PortAllocationStrategy,
    /// Protocol used (HTTP, HTTPS, WebSocket, etc.)
    pub protocol: ServiceProtocol,
    /// Whether this service is externally accessible
    pub external_access: bool,
    /// Load balancing configuration
    pub load_balancing: Option<LoadBalancingConfig>,
    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,
    /// Custom metadata for Songbird
    pub songbird_metadata: HashMap<String, String>,
}

/// Service protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceProtocol {
    Http,
    Https,
    WebSocket,
    Grpc,
    Tcp,
    Udp,
    Custom { name: String, secure: bool },
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy
    pub strategy: LoadBalancingStrategy,
    /// Health check requirements
    pub health_check_required: bool,
    /// Session affinity
    pub session_affinity: bool,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin { weights: HashMap<String, u32> },
    IpHash,
    Random,
}

/// Health check configuration for ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check path (for HTTP services)
    pub path: String,
    /// Expected HTTP status code
    pub expected_status: u16,
    /// Check interval in seconds
    pub interval_secs: u32,
    /// Timeout for health checks
    pub timeout_secs: u32,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
}

/// Comprehensive port management for the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemPortConfig {
    /// Core squirrel service ports
    pub squirrel: SquirrelPorts,
    /// External primal service ports
    pub primals: PrimalsPortConfig,
    /// Development and testing ports
    pub development: DevelopmentPortConfig,
    /// Production port ranges
    pub production: ProductionPortConfig,
    /// Port allocation settings
    pub allocation: PortAllocationSettings,
}

/// Squirrel service port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquirrelPorts {
    /// Main HTTP API port
    pub api: PortConfig,
    /// WebSocket connections
    pub websocket: PortConfig,
    /// Health check endpoint
    pub health: PortConfig,
    /// Metrics collection
    pub metrics: PortConfig,
    /// Admin interface
    pub admin: PortConfig,
    /// MCP protocol port
    pub mcp: PortConfig,
    /// AI coordination port
    pub ai_coordination: PortConfig,
    /// Chaos engineering port
    pub chaos: PortConfig,
}

/// External primal service port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalsPortConfig {
    /// Songbird service mesh
    pub songbird: PortConfig,
    /// BearDog security service
    pub beardog: PortConfig,
    /// ToadStool compute service
    pub toadstool: PortConfig,
    /// NestGate storage service
    pub nestgate: PortConfig,
    /// BiomeOS orchestration
    pub biomeos: PortConfig,
}

/// Development environment port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentPortConfig {
    /// Base port for development services
    pub base_port: u16,
    /// Port offset for multiple instances
    pub port_offset: u16,
    /// Use localhost instead of bind addresses
    pub use_localhost: bool,
    /// Enable port auto-discovery
    pub auto_discovery: bool,
}

/// Production environment port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionPortConfig {
    /// Port ranges for different service types
    pub service_ranges: HashMap<String, Range<u16>>,
    /// Reserved ports that should not be allocated
    pub reserved_ports: HashSet<u16>,
    /// Minimum port number
    pub min_port: u16,
    /// Maximum port number
    pub max_port: u16,
    /// Require explicit port assignment
    pub explicit_assignment: bool,
}

/// Port allocation settings and policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortAllocationSettings {
    /// Default allocation strategy
    pub default_strategy: PortAllocationStrategy,
    /// Retry count for port allocation
    pub retry_count: u32,
    /// Timeout for port binding
    pub bind_timeout_secs: u32,
    /// Enable port conflict detection
    pub conflict_detection: bool,
    /// Enable integration with Songbird
    pub songbird_integration: bool,
}

impl Default for EcosystemPortConfig {
    fn default() -> Self {
        Self {
            squirrel: SquirrelPorts::default(),
            primals: PrimalsPortConfig::default(),
            development: DevelopmentPortConfig::default(),
            production: ProductionPortConfig::default(),
            allocation: PortAllocationSettings::default(),
        }
    }
}

impl Default for SquirrelPorts {
    fn default() -> Self {
        let is_production = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let strategy = if is_production {
            PortAllocationStrategy::SongbirdManaged
        } else {
            PortAllocationStrategy::EnvironmentBased {
                env_var: "SQUIRREL_PORT".to_string(),
                default: 8080,
            }
        };

        Self {
            api: PortConfig {
                service_name: "squirrel-api".to_string(),
                allocation_strategy: strategy.clone(),
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            websocket: PortConfig {
                service_name: "squirrel-websocket".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_WS_PORT".to_string(),
                    default: 8081,
                },
                protocol: ServiceProtocol::WebSocket,
                external_access: true,
                load_balancing: None,
                health_check: None,
                songbird_metadata: HashMap::new(),
            },
            health: PortConfig {
                service_name: "squirrel-health".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_HEALTH_PORT".to_string(),
                    default: 8082,
                },
                protocol: ServiceProtocol::Http,
                external_access: false,
                load_balancing: None,
                health_check: None,
                songbird_metadata: HashMap::new(),
            },
            metrics: PortConfig {
                service_name: "squirrel-metrics".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_METRICS_PORT".to_string(),
                    default: 9090,
                },
                protocol: ServiceProtocol::Http,
                external_access: false,
                load_balancing: None,
                health_check: None,
                songbird_metadata: HashMap::new(),
            },
            admin: PortConfig {
                service_name: "squirrel-admin".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_ADMIN_PORT".to_string(),
                    default: 8083,
                },
                protocol: ServiceProtocol::Http,
                external_access: false,
                load_balancing: None,
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            mcp: PortConfig {
                service_name: "squirrel-mcp".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_MCP_PORT".to_string(),
                    default: 8444,
                },
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            ai_coordination: PortConfig {
                service_name: "squirrel-ai".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_AI_PORT".to_string(),
                    default: 8445,
                },
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            chaos: PortConfig {
                service_name: "squirrel-chaos".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SQUIRREL_CHAOS_PORT".to_string(),
                    default: 8446,
                },
                protocol: ServiceProtocol::Http,
                external_access: false,
                load_balancing: None,
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
        }
    }
}

impl Default for PrimalsPortConfig {
    fn default() -> Self {
        let is_production = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        let strategy = if is_production {
            PortAllocationStrategy::SongbirdManaged
        } else {
            PortAllocationStrategy::Dynamic { range: 8500..8600 }
        };

        Self {
            songbird: PortConfig {
                service_name: "songbird".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "SONGBIRD_PORT".to_string(),
                    default: 8080,
                },
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            beardog: PortConfig {
                service_name: "beardog".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "BEARDOG_PORT".to_string(),
                    default: 8443,
                },
                protocol: ServiceProtocol::Https,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            toadstool: PortConfig {
                service_name: "toadstool".to_string(),
                allocation_strategy: strategy.clone(),
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            nestgate: PortConfig {
                service_name: "nestgate".to_string(),
                allocation_strategy: strategy.clone(),
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
            biomeos: PortConfig {
                service_name: "biomeos".to_string(),
                allocation_strategy: PortAllocationStrategy::EnvironmentBased {
                    env_var: "BIOMEOS_PORT".to_string(),
                    default: 5000,
                },
                protocol: ServiceProtocol::Http,
                external_access: true,
                load_balancing: Some(LoadBalancingConfig::default()),
                health_check: Some(HealthCheckConfig::default()),
                songbird_metadata: HashMap::new(),
            },
        }
    }
}

impl Default for DevelopmentPortConfig {
    fn default() -> Self {
        Self {
            base_port: env::var("DEV_BASE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            port_offset: env::var("DEV_PORT_OFFSET")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0),
            use_localhost: env::var("DEV_USE_LOCALHOST")
                .map(|v| v == "true")
                .unwrap_or(true),
            auto_discovery: env::var("DEV_AUTO_DISCOVERY")
                .map(|v| v == "true")
                .unwrap_or(true),
        }
    }
}

impl Default for ProductionPortConfig {
    fn default() -> Self {
        let mut service_ranges = HashMap::new();
        service_ranges.insert("web".to_string(), 8000..8100);
        service_ranges.insert("api".to_string(), 8100..8200);
        service_ranges.insert("internal".to_string(), 8200..8300);
        service_ranges.insert("monitoring".to_string(), 9000..9100);

        let mut reserved_ports = HashSet::new();
        reserved_ports.insert(22); // SSH
        reserved_ports.insert(80); // HTTP
        reserved_ports.insert(443); // HTTPS
        reserved_ports.insert(3306); // MySQL
        reserved_ports.insert(5432); // PostgreSQL
        reserved_ports.insert(6379); // Redis
        reserved_ports.insert(27017); // MongoDB

        Self {
            service_ranges,
            reserved_ports,
            min_port: 8000,
            max_port: 9999,
            explicit_assignment: true,
        }
    }
}

impl Default for PortAllocationSettings {
    fn default() -> Self {
        Self {
            default_strategy: PortAllocationStrategy::Dynamic { range: 8000..9000 },
            retry_count: 5,
            bind_timeout_secs: 30,
            conflict_detection: true,
            songbird_integration: env::var("SONGBIRD_INTEGRATION")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check_required: true,
            session_affinity: false,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            expected_status: 200,
            interval_secs: 30,
            timeout_secs: 5,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

/// Utility functions for port management
impl EcosystemPortConfig {
    /// Create configuration for development environment
    pub fn for_development() -> Self {
        let mut config = Self::default();
        config.development.use_localhost = true;
        config.development.auto_discovery = true;
        config
    }

    /// Create configuration for production environment
    pub fn for_production() -> Self {
        let mut config = Self::default();
        config.allocation.songbird_integration = true;
        config.production.explicit_assignment = true;
        config
    }

    /// Get effective port for a service
    pub fn get_service_port(&self, service_name: &str) -> Option<u16> {
        match service_name {
            "squirrel-api" => self.resolve_port(&self.squirrel.api),
            "squirrel-websocket" => self.resolve_port(&self.squirrel.websocket),
            "squirrel-health" => self.resolve_port(&self.squirrel.health),
            "squirrel-metrics" => self.resolve_port(&self.squirrel.metrics),
            "squirrel-admin" => self.resolve_port(&self.squirrel.admin),
            "squirrel-mcp" => self.resolve_port(&self.squirrel.mcp),
            "squirrel-ai" => self.resolve_port(&self.squirrel.ai_coordination),
            "squirrel-chaos" => self.resolve_port(&self.squirrel.chaos),
            "songbird" => self.resolve_port(&self.primals.songbird),
            "beardog" => self.resolve_port(&self.primals.beardog),
            "toadstool" => self.resolve_port(&self.primals.toadstool),
            "nestgate" => self.resolve_port(&self.primals.nestgate),
            "biomeos" => self.resolve_port(&self.primals.biomeos),
            _ => None,
        }
    }

    /// Resolve port from allocation strategy
    fn resolve_port(&self, port_config: &PortConfig) -> Option<u16> {
        match &port_config.allocation_strategy {
            PortAllocationStrategy::Fixed(port) => Some(*port),
            PortAllocationStrategy::EnvironmentBased { env_var, default } => env::var(env_var)
                .ok()
                .and_then(|p| p.parse().ok())
                .or(Some(*default)),
            PortAllocationStrategy::Dynamic { range } => {
                // In a real implementation, this would find an available port
                Some(range.start)
            }
            PortAllocationStrategy::SongbirdManaged => {
                // In production, Songbird would provide the port
                // For now, return None to indicate external management
                None
            }
        }
    }

    /// Generate service URL for a given service
    pub fn get_service_url(
        &self,
        service_name: &str,
        use_localhost: Option<bool>,
    ) -> Option<String> {
        let port = self.get_service_port(service_name)?;
        let host = if use_localhost.unwrap_or(self.development.use_localhost) {
            "localhost"
        } else {
            "0.0.0.0"
        };

        // Determine protocol
        let protocol = match service_name {
            name if name.contains("websocket") => "ws",
            name if name.contains("beardog") => "https",
            _ => "http",
        };

        Some(format!("{}://{}:{}", protocol, host, port))
    }

    /// Generate all service endpoints
    pub fn generate_service_endpoints(&self) -> HashMap<String, String> {
        let mut endpoints = HashMap::new();

        let services = vec![
            "squirrel-api",
            "squirrel-websocket",
            "squirrel-health",
            "squirrel-metrics",
            "squirrel-admin",
            "squirrel-mcp",
            "squirrel-ai",
            "squirrel-chaos",
            "songbird",
            "beardog",
            "toadstool",
            "nestgate",
            "biomeos",
        ];

        for service in services {
            if let Some(url) = self.get_service_url(service, None) {
                endpoints.insert(service.to_string(), url);
            }
        }

        endpoints
    }
}
