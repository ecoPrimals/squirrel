## New Project Structure

```
src/
├── core/
│   ├── context/
│   │   ├── tracker.rs      # Context management
│   │   │   ├── lifecycle.rs  # Context lifecycle
│   │   │   ├── registry.rs   # Context registry
│   │   │   └── events.rs     # Context events
│   │   ├── state.rs        # State handling
│   │   │   ├── store.rs      # State store
│   │   │   ├── snapshot.rs   # State snapshots
│   │   │   └── diff.rs       # State diffing
│   │   ├── sync.rs         # Synchronization
│   │   │   ├── protocol.rs   # Sync protocol
│   │   │   ├── conflict.rs   # Conflict resolution
│   │   │   └── queue.rs      # Sync queue
│   │   ├── persistence.rs  # State persistence
│   │   │   ├── storage.rs    # Storage backend
│   │   │   ├── cache.rs      # Caching layer
│   │   │   └── migration.rs  # Data migration
│   │   └── mod.rs          # Public interface
│   ├── commands/
│   │   ├── lifecycle.rs    # Command execution
│   │   │   ├── stages.rs     # Execution stages
│   │   │   ├── pipeline.rs   # Command pipeline
│   │   │   └── rollback.rs   # Rollback handling
│   │   ├── validation.rs   # Command validation
│   │   │   ├── rules.rs      # Validation rules
│   │   │   ├── schema.rs     # Command schema
│   │   │   └── context.rs    # Validation context
│   │   ├── hooks.rs        # Hook system
│   │   │   ├── registry.rs   # Hook registry
│   │   │   ├── chain.rs      # Hook chain
│   │   │   └── events.rs     # Hook events
│   │   ├── resources.rs    # Resource management
│   │   │   ├── allocator.rs  # Resource allocation
│   │   │   ├── pool.rs       # Resource pool
│   │   │   └── limits.rs     # Resource limits
│   │   └── mod.rs          # Public interface
│   ├── error/             # Error handling
│   │   ├── types.rs        # Error types
│   │   ├── context.rs      # Error context
│   │   ├── recovery.rs     # Error recovery
│   │   └── mod.rs          # Error exports
│   ├── events/            # Event system
│   │   ├── bus.rs          # Event bus
│   │   ├── handler.rs      # Event handlers
│   │   ├── filter.rs       # Event filtering
│   │   └── mod.rs          # Event exports
│   ├── metrics/           # Metrics system
│   │   ├── collector.rs    # Metrics collection
│   │   ├── registry.rs     # Metrics registry
│   │   ├── exporter.rs     # Metrics export
│   │   └── mod.rs          # Metrics exports
│   └── mod.rs              # Core system exports
├── mcp/
│   ├── messages/           # Message definitions
│   │   ├── request.rs      # Request message types
│   │   │   ├── command.rs   # Command requests
│   │   │   ├── query.rs     # Query requests
│   │   │   └── control.rs   # Control requests
│   │   ├── response.rs     # Response message types
│   │   │   ├── success.rs   # Success responses
│   │   │   ├── error.rs     # Error responses
│   │   │   └── status.rs    # Status responses
│   │   ├── common/         # Common message components
│   │   │   ├── header.rs    # Message headers
│   │   │   ├── payload.rs   # Payload types
│   │   │   └── metadata.rs  # Message metadata
│   │   └── mod.rs          # Message exports
│   ├── protocol/          # Protocol implementation
│   │   ├── handler.rs      # Message handlers
│   │   │   ├── router.rs    # Message routing
│   │   │   ├── processor.rs # Message processing
│   │   │   └── middleware.rs # Protocol middleware
│   │   ├── validator.rs    # Protocol validation
│   │   │   ├── schema.rs    # Message schemas
│   │   │   ├── rules.rs     # Validation rules
│   │   │   └── context.rs   # Validation context
│   │   ├── security/       # Security features
│   │   │   ├── auth.rs      # Authentication
│   │   │   ├── crypto.rs    # Cryptography
│   │   │   └── policy.rs    # Security policies
│   │   └── mod.rs          # Protocol exports
│   ├── transport/         # Transport layer
│   │   ├── tcp.rs         # TCP transport
│   │   │   ├── connection.rs # Connection management
│   │   │   ├── buffer.rs    # Buffer handling
│   │   │   └── stream.rs    # Stream management
│   │   ├── websocket.rs   # WebSocket transport
│   │   │   ├── session.rs   # Session management
│   │   │   ├── frame.rs     # Frame handling
│   │   │   └── upgrade.rs   # Protocol upgrade
│   │   ├── common/        # Common transport features
│   │   │   ├── config.rs    # Transport config
│   │   │   ├── error.rs     # Transport errors
│   │   │   └── metrics.rs   # Transport metrics
│   │   └── mod.rs         # Transport exports
│   ├── session/          # Session management
│   │   ├── manager.rs     # Session manager
│   │   │   ├── lifecycle.rs # Session lifecycle
│   │   │   ├── state.rs     # Session state
│   │   │   └── events.rs    # Session events
│   │   ├── store.rs       # Session storage
│   │   │   ├── memory.rs    # Memory storage
│   │   │   ├── file.rs      # File storage
│   │   │   └── cache.rs     # Session cache
│   │   └── mod.rs         # Session exports
│   ├── error/            # Error handling
│   │   ├── types.rs       # Error types
│   │   ├── codes.rs       # Error codes
│   │   ├── context.rs     # Error context
│   │   └── mod.rs         # Error exports
│   └── mod.rs             # MCP system exports
├── ai/
│   └── mcp-tools/        # AI-specific MCP tools
│       ├── code/         # Code analysis tools
│       │   ├── parser.rs  # Code parsing
│       │   │   ├── ast.rs      # Abstract syntax tree
│       │   │   ├── lexer.rs    # Token lexer
│       │   │   └── visitor.rs  # AST visitor pattern
│       │   ├── analyzer.rs # Code analysis
│       │   │   ├── metrics.rs  # Code metrics
│       │   │   ├── patterns.rs # Pattern detection
│       │   │   └── types.rs    # Type analysis
│       │   ├── cache.rs   # Analysis caching
│       │   └── mod.rs     # Code tools exports
│       ├── chat/         # Chat interaction tools
│       │   ├── handler.rs # Chat message handling
│       │   │   ├── message.rs  # Message types
│       │   │   ├── router.rs   # Message routing
│       │   │   └── queue.rs    # Message queue
│       │   ├── context.rs # Chat context management
│       │   │   ├── memory.rs   # Context memory
│       │   │   ├── history.rs  # Chat history
│       │   │   └── state.rs    # Context state
│       │   ├── tools.rs   # Chat-specific tools
│       │   └── mod.rs     # Chat tools exports
│       ├── exec/         # Execution tools
│       │   ├── runner.rs  # Command execution
│       │   │   ├── process.rs  # Process management
│       │   │   ├── sandbox.rs  # Execution sandbox
│       │   │   └── timeout.rs  # Timeout handling
│       │   ├── monitor.rs # Execution monitoring
│       │   │   ├── metrics.rs  # Performance metrics
│       │   │   ├── logging.rs  # Execution logs
│       │   │   └── alerts.rs   # Alert system
│       │   ├── security.rs # Security controls
│       │   └── mod.rs     # Execution exports
│       ├── common/       # Shared functionality
│       │   ├── config.rs  # Tool configuration
│       │   ├── error.rs   # Error handling
│       │   ├── metrics.rs # Shared metrics
│       │   └── mod.rs     # Common exports
│       └── mod.rs        # AI tools exports
├── data/             # Data management layer
│   ├── storage/      # Storage implementations
│   │   ├── memory/   # In-memory storage
│   │   │   ├── store.rs    # Memory store
│   │   │   ├── cache.rs    # Memory cache
│   │   │   └── mod.rs      # Memory exports
│   │   ├── file/     # File-based storage
│   │   │   ├── store.rs    # File store
│   │   │   ├── cache.rs    # File cache
│   │   │   └── mod.rs      # File exports
│   │   └── mod.rs    # Storage exports
│   ├── versioning/   # Data versioning
│   │   ├── manager.rs # Version manager
│   │   ├── diff.rs    # Version diffing
│   │   └── mod.rs     # Versioning exports
│   ├── migration/    # Data migration
│   │   ├── manager.rs # Migration manager
│   │   ├── schema.rs  # Schema handling
│   │   └── mod.rs     # Migration exports
│   └── mod.rs        # Data system exports
├── deployment/       # Deployment system
│   ├── container/   # Container management
│   │   ├── builder.rs # Container builder
│   │   ├── config.rs  # Container config
│   │   └── mod.rs     # Container exports
│   ├── orchestration/ # Orchestration
│   │   ├── manager.rs # Orchestration manager
│   │   ├── config.rs  # Orchestration config
│   │   └── mod.rs     # Orchestration exports
│   ├── discovery/    # Service discovery
│   │   ├── provider.rs # Discovery provider
│   │   ├── config.rs   # Discovery config
│   │   └── mod.rs      # Discovery exports
│   └── mod.rs        # Deployment exports
├── monitoring/       # Monitoring system
│   ├── tracing/     # Distributed tracing
│   │   ├── collector.rs # Trace collector
│   │   ├── exporter.rs  # Trace exporter
│   │   └── mod.rs       # Tracing exports
│   ├── logging/     # Logging system
│   │   ├── collector.rs # Log collector
│   │   ├── exporter.rs  # Log exporter
│   │   └── mod.rs       # Logging exports
│   ├── metrics/     # Metrics system
│   │   ├── collector.rs # Metrics collector
│   │   ├── exporter.rs  # Metrics exporter
│   │   └── mod.rs       # Metrics exports
│   └── mod.rs       # Monitoring exports
├── security/        # Security system
│   ├── auth/       # Authentication
│   │   ├── provider.rs # Auth provider
│   │   ├── config.rs   # Auth config
│   │   └── mod.rs      # Auth exports
│   ├── encryption/ # Encryption
│   │   ├── provider.rs # Encryption provider
│   │   ├── config.rs   # Encryption config
│   │   └── mod.rs      # Encryption exports
│   ├── audit/      # Audit logging
│   │   ├── logger.rs   # Audit logger
│   │   ├── config.rs   # Audit config
│   │   └── mod.rs      # Audit exports
│   └── mod.rs      # Security exports
├── ui/
│   ├── components/        # UI components
│   │   ├── base/         # Base component traits
│   │   │   ├── traits.rs  # Component traits
│   │   │   ├── props.rs   # Common props
│   │   │   └── mod.rs     # Base exports
│   │   ├── layout/       # Layout components
│   │   │   ├── container.rs # Container component
│   │   │   ├── grid.rs    # Grid layout
│   │   │   ├── stack.rs   # Stack layout
│   │   │   └── mod.rs     # Layout exports
│   │   ├── controls/     # Control components
│   │   │   ├── button.rs  # Button component
│   │   │   ├── input.rs   # Input component
│   │   │   ├── select.rs  # Select component
│   │   │   └── mod.rs     # Controls exports
│   │   ├── feedback/     # Feedback components
│   │   │   ├── alert.rs   # Alert component
│   │   │   ├── progress.rs # Progress component
│   │   │   └── mod.rs     # Feedback exports
│   │   └── mod.rs        # Components exports
│   ├── layout/           # Layout management
│   │   ├── engine.rs     # Layout engine
│   │   ├── constraints.rs # Layout constraints
│   │   ├── algorithms.rs  # Layout algorithms
│   │   └── mod.rs        # Layout exports
│   ├── theme/            # Theme system
│   │   ├── colors.rs     # Color definitions
│   │   ├── typography.rs # Typography system
│   │   ├── spacing.rs    # Spacing system
│   │   ├── animation.rs  # Animation system
│   │   └── mod.rs        # Theme exports
│   ├── state/            # UI state management
│   │   ├── store.rs      # State store
│   │   ├── reducer.rs    # State reducers
│   │   └── mod.rs        # State exports
│   └── mod.rs            # UI system exports
└── lib.rs                # Library exports