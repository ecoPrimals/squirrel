---
description: Architecture and Dependency Specifications for Squirrel MCP
version: 1.6.0
last_updated: 2024-12-19
owner: Core Team
---

# Squirrel Architecture Specifications

## Table of Contents
1. [Core Dependencies](#core-dependencies)
2. [Workspace Organization](#workspace-organization)
3. [Component Status](#component-status)
4. [Performance Targets](#performance-targets)
5. [Current Focus](#current-focus)
6. [Version History](#version-history)
7. [Development Process](#development-process)

## Core Dependencies

### Runtime & Async
- **Tokio**: `tokio = { version = "1.36", features = ["full"] }`
  - Multi-threading, Async I/O, Time utilities, Test utilities

### Serialization
- **Serde**: `serde = { version = "1.0", features = ["derive"] }`
- **JSON**: `serde_json = "1.0"`

### Error Handling
- **Structured Errors**: `thiserror = "1.0"`
- **Error Handling**: `anyhow = "1.0"`

### Observability
- **Tracing**: `tracing = "0.1"`
- **Logging**: `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`

## Workspace Organization

### Directory Structure
```
squirrel/
├── code/                   # All source code
│   ├── crates/            # Rust crates organized by purpose
│   │   ├── core/          # Core functionality
│   │   │   ├── context/       # Context management
│   │   │   ├── mcp/           # Machine Context Protocol
│   │   │   ├── plugins/       # Plugin system
│   │   │   ├── interfaces/    # Shared interfaces
│   │   │   └── core/          # Core utilities
│   │   ├── integration/   # Integration components
│   │   │   ├── api-clients/       # API client implementations
│   │   │   ├── context-adapter/   # Context adapter systems
│   │   │   ├── web/               # Web integration
│   │   │   └── mcp-pyo3-bindings/ # Python bindings
│   │   ├── services/      # Service implementations
│   │   │   ├── app/               # Application services
│   │   │   ├── commands/          # Command system
│   │   │   ├── dashboard-core/    # Dashboard core services
│   │   │   ├── monitoring/        # Monitoring services
│   │   │   └── nestgate-orchestrator/ # Central execution engine
│   │   ├── tools/         # Development tools
│   │   │   ├── ai-tools/          # AI integration tools
│   │   │   ├── cli/               # Command-line tools
│   │   │   └── rule-system/       # Rule system tools
│   │   └── ui/            # User interface components
│   │       ├── ui-tauri-react/    # Tauri React UI
│   │       └── ui-terminal/       # Terminal UI
│   └── src/               # Frontend source code
└── specs/                 # Specifications and design documents
    ├── core/              # Core specifications
    ├── integration/       # Integration specifications
    ├── services/          # Service specifications
    ├── tools/             # Tool specifications
    ├── ui/                # UI specifications
    └── archived/          # Archived specifications
```

## Component Status

### 🏗️ Core Components

#### Core/Context (95% Complete)
- **Purpose**: State tracking, file system context handling, editor integration
- **Key Features**:
  - State tracking (100%)
  - File system context handling (100%)
  - Real-time sync (95%)
  - Recovery features (90%)

#### Core/MCP (98% Complete)
- **Purpose**: Machine Context Protocol implementation
- **Key Features**:
  - Message handling (100%)
  - Client/Server implementation (100%)
  - Protocol validation (100%)
  - Resource management (100%)
  - UI integration (95%)
  - Tool lifecycle foundation (95%)
  - Security foundation (95%)
- **Documentation**: [core/mcp/PROGRESS.md](core/mcp/PROGRESS.md)

#### Core/Plugins (95% Complete)
- **Purpose**: Plugin system architecture with cross-platform sandboxing
- **Key Features**:
  - Plugin architecture (100%)
  - Security model (100%)
  - Cross-platform sandboxing (95%)
  - Resource monitoring (95%)
  - Plugin loading (90%)
  - Command registration (85%)
  - Lifecycle management (80%)
  - Discovery system (70%)
  - Dependency resolution (60%)
- **Documentation**: [core/plugins/IMPLEMENTATION_COMPLETE.md](core/plugins/IMPLEMENTATION_COMPLETE.md)

#### Core/Interfaces (100% Complete)
- **Purpose**: Shared interfaces and cross-component communication
- **Key Features**:
  - Common interfaces (100%)
  - Cross-component communication (100%)
  - Documentation (100%)

### 🚀 Service Components

#### Services/Nestgate-Orchestrator (95% Complete)
- **Purpose**: Central execution engine for MCP brain separation
- **Architecture**: MCP acts as pure "brain" (observer, thinker, planner) while orchestrator handles all execution
- **Key Features**:
  - Service lifecycle management (100%)
  - Port allocation and management (100%)
  - Health monitoring framework (100%)
  - gRPC service interface (100%)
  - Configuration management (100%)
  - Job coordination system (95%)
  - Resource requirements management (95%)
  - Integration tests (90%)
  - MCP brain integration (90%)
  - Error handling and recovery (90%)
  - Performance monitoring (85%)

#### Services/Monitoring (100% Complete)
- **Purpose**: Comprehensive monitoring and alerting system
- **Key Features**:
  - Metrics collection (100%)
  - Health monitoring (100%)
  - Alerting system (100%)
  - Network monitoring (100%)
  - Dashboard system (100%)
  - Analytics integration (100%)
  - Storage subsystem (100%)
  - Plugin architecture (100%)
  - Testing infrastructure (100%)
- **Documentation**: [services/monitoring/SPEC.md](services/monitoring/SPEC.md)

#### Services/App (95% Complete)
- **Purpose**: Application services and state management
- **Key Features**:
  - Configuration management (100%)
  - Error handling (100%)
  - Command integration (95%)
  - Context integration (95%)
  - Tests (90%)
  - MCP integration (90%)

#### Services/Commands (95% Complete)
- **Purpose**: Command registration, execution, and validation
- **Key Features**:
  - Command registration and execution (100%)
  - Command validation framework (100%)
  - Help system foundation (100%)
  - Output formatting system (100%)
  - Core commands implementation (95%)
  - Performance optimization (90%)

#### Services/Dashboard-Core (90% Complete)
- **Purpose**: Dashboard data models and widget framework
- **Key Features**:
  - Dashboard data models (100%)
  - Dashboard state management (95%)
  - Real-time updates (90%)
  - Widget framework (85%)
  - Filtering system (80%)

### 🔗 Integration Components

#### Integration/Web (85% Complete)
- **Purpose**: Web API endpoints and WebSocket implementation
- **Key Features**:
  - Core architecture (98%)
  - Database integration (100%)
  - WebSocket implementation (100%)
  - Command execution API (100%)
  - Build system (100%)
  - Axum 0.7 compatibility (100%)
  - Test infrastructure (95%)
  - API endpoints (90%)
  - Authentication & security (60%)
  - MCP integration (50%)
  - API documentation (20%)
- **Documentation**: [integration/web/Implementation.md](integration/web/Implementation.md)

#### Integration/API-Clients (85% Complete)
- **Purpose**: External API integration with authentication and rate limiting
- **Key Features**:
  - GitHub API integration (95%)
  - Authentication management (90%)
  - Error handling (90%)
  - Rate limiting (80%)
  - Caching (70%)

#### Integration/Context-Adapter (80% Complete)
- **Purpose**: Cross-platform context adaptation and synchronization
- **Key Features**:
  - Context adaptation framework (95%)
  - Error handling (90%)
  - Cross-platform adaptation (75%)
  - Synchronization (70%)

#### Integration/MCP-PyO3-Bindings (75% Complete)
- **Purpose**: Python bindings for ML model integration
- **Key Features**:
  - Core bindings (90%)
  - Python environment management (80%)
  - Error handling (80%)
  - ML model integration (70%)
  - Documentation (60%)

### 🛠️ Tools Components

#### Tools/CLI (90% Complete)
- **Purpose**: Command-line framework and utilities
- **Key Features**:
  - Command-line framework (100%)
  - Command registration (95%)
  - Argument parsing (95%)
  - Output formatting (90%)
  - Error handling (90%)
  - Documentation (70%)

#### Tools/Rule-System (80% Complete)
- **Purpose**: Rule definition, execution, and management
- **Key Features**:
  - Rule definition framework (90%)
  - Rule execution engine (85%)
  - Integration with context (80%)
  - Rule management (75%)
  - Documentation (70%)

#### Tools/AI-Tools (75% Complete)
- **Purpose**: AI integration and model management
- **Key Features**:
  - AI integration framework (90%)
  - MCP integration (80%)
  - Model management (80%)
  - Inference services (70%)
  - Documentation (70%)
  - Performance optimization (60%)
  - Security (60%)

### 🎨 UI Components

#### UI/TauriReact (90% Complete)
- **Purpose**: React-based desktop UI with Tauri integration
- **Key Features**:
  - Core React components (95%)
  - Tauri integration (95%)
  - State management (90%)
  - Theming (90%)
  - MCP integration (85%)
  - Performance (85%)
  - Tests (80%)
  - Accessibility (70%)

#### UI/Terminal (95% Complete)
- **Purpose**: Terminal-based UI with TUI components
- **Key Features**:
  - Terminal UI framework (100%)
  - Command display (100%)
  - Color theming (100%)
  - Status display (95%)
  - Progress indicators (90%)
  - Interactive forms (90%)
  - Responsive layout (90%)

### 🎯 Recently Completed Components

#### MCP Resilience Framework (100% Complete)
- Circuit breaker pattern for service calls
- Retry mechanisms with exponential backoff
- Bulkhead isolation implementation
- Rate limiting implementation
- Recovery strategies for failures
- State synchronization
- Health checking system
- Comprehensive testing with examples
- **Documentation**: [integration/RETRY_IMPLEMENTATION.md](integration/RETRY_IMPLEMENTATION.md)

#### Terminal UI MCP Integration (95% Complete)
- McpMetricsProvider interface
- RealMcpMetricsProvider implementation
- Cache-optimized metrics retrieval
- Performance tracking
- Protocol metrics visualization
- Connection management
- Command-line integration
- Event-driven updates
- **Documentation**: [ui/implementation/IMPLEMENTATION_PROGRESS.md](ui/implementation/IMPLEMENTATION_PROGRESS.md)

### 🚧 In Progress Components

#### MCP Observability Framework (55% Complete)
- Metrics collection and reporting (75%)
- Distributed tracing (60%)
- Structured logging (60%)
- Health checking system (65%)
- Alerting integration (50%)
- Dashboard integration (50%)
- **Documentation**: [core/mcp/observability-telemetry.md](core/mcp/observability-telemetry.md)

#### RBAC Enhancement for MCP (80% Complete)
- Fine-grained permission control (90%)
- Role inheritance improvements (85%)
- Permission validation framework (85%)
- RBAC integration with other components (70%)
- **Documentation**: [core/mcp/RBAC_IMPLEMENTATION_STATUS.md](core/mcp/RBAC_IMPLEMENTATION_STATUS.md)

#### Web Interface MCP Integration (70% Complete)
- Bidirectional communication (90%)
- Real-time updates (80%)
- Comprehensive error handling (70%)
- Secure authentication (60%)
- Fine-grained authorization (50%)

## Performance Targets

| Metric | Target |
|--------|--------|
| Command execution | < 50ms |
| Web API response time | < 200ms |
| Memory usage | < 100MB |
| Error rate | < 1% |
| Test coverage | > 90% |

## Current Focus

### 🎯 Priority 1: Core Infrastructure
1. **MCP Observability Framework**
   - Metrics collection and reporting
   - Distributed tracing
   - Structured logging
   - Event processing system
   - Alerting system
   - Dashboard integration

2. **RBAC Enhancement for MCP**
   - Fine-grained permission control
   - Role inheritance improvements
   - Permission validation framework
   - RBAC integration with other components

### 🎯 Priority 2: Integration & Tools
3. **Tool Lifecycle Completion**
   - Enhanced error recovery for tools
   - State transition validation
   - Comprehensive cleanup procedures
   - Resource tracking metrics

4. **Web Interface MCP Integration**
   - Bidirectional communication
   - Real-time updates
   - Secure authentication
   - Fine-grained authorization
   - Comprehensive error handling

### 🎯 Priority 3: AI & Advanced Features
5. **AI Tools Integration**
   - Model management system
   - AI tool framework
   - Inference optimization
   - Cross-platform deployment

## Version History

### v1.6.0 (2024-12-19)
- Added Nestgate Orchestrator to Service Components section
- Updated orchestrator implementation status to 95% complete
- Documented orchestrator as central execution engine for MCP brain separation
- Added details on service lifecycle management, port allocation, and job coordination
- Established architecture pattern where MCP acts as pure "brain" and orchestrator handles execution
- Reorganized and cleaned up specifications structure
- Improved readability and consistency across sections
- Updated version information and last revision date

### v1.5.0 (2024-09-25)
- Updated directory organization to match new structure
- Updated component status for all major components
- Added AI Tools integration as a current focus area
- Updated documentation paths to reflect new directory structure
- Added details on recently completed components
- Updated implementation percentages across all components
- Reorganized services section to match crate structure

### v1.4.0 (2024-09-15)
- Added Terminal UI MCP Integration to "Recently Completed Components"
- Updated UI Components implementation status to 90%
- Enhanced MCP Protocol implementation status to 98%
- Added detailed UI implementation in "Recently Completed Components"
- Updated MCP integration details with Terminal UI components
- Added metrics caching and visualization components
- Added UI implementation progress tracking
- Fixed performance targets for UI components

### v1.3.0 (2024-07-15)
- Updated Plugin System implementation status to 95%
- Added Plugin Sandbox implementation details to "Recently Completed Components"
- Fixed test reliability with proper process registration in sandbox tests
- Updated Resource Monitor integration with sandbox
- Enhanced Plugin System error handling
- Added Linux Sandbox implementation details
- Improved cross-platform build compatibility
- Updated documentation for sandbox implementation

### v1.2.0 (2024-04-10)
- Added Monitoring System to "Recently Completed Components"
- Updated Command System implementation status to 95%
- Enhanced Error Recovery System documentation
- Updated MCP Protocol implementation status
- Added Plugin System section with initial implementation details
- Added Performance Targets section
- Refined Current Focus areas
- Added Version History section

## Development Process

### Specification Update Process

We maintain up-to-date specifications through:

1. **Regular Updates**: All specifications updated before each sprint
2. **Validation Tools**: `specs/tools/spec_validation.sh` checks for:
   - Broken links in specifications
   - References to non-existent code files
   - Outdated "last_updated" dates
   - Inconsistent implementation percentages
3. **Update Guidelines**: See [SPEC_UPDATES_FOR_NEXT_SPRINT.md](SPEC_UPDATES_FOR_NEXT_SPRINT.md)
4. **PR Template**: Use specification update PR template

### Using the Validation Tool

Check all specifications:
```bash
./specs/tools/spec_validation.sh
```

Check team-specific specifications:
```bash
./specs/spec_validation.sh core         # Core team specifications
./specs/spec_validation.sh integration  # Integration team specifications
./specs/spec_validation.sh services     # Services team specifications
./specs/spec_validation.sh tools        # Tools team specifications
./specs/spec_validation.sh ui           # UI team specifications
```

### Feedback & Contact

**Feedback**: Please provide feedback on these specifications to the architecture team. We actively refine our approach based on implementation insights.

**Contact**: For questions or clarifications, contact the architecture team at architecture@squirrel-labs.org.

### References
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [Error Handling in Rust](https://blog.burntsushi.net/rust-error-handling/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---
*This document is maintained by the Core Team. Last revision: December 19, 2024.* 