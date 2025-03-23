---
title: CLI Enhanced Architecture Specification
version: 1.0.0
date: 2024-04-15
status: proposal
priority: high
author: DataScienceBioLab
---

# CLI Enhanced Architecture Specification

## Overview

This document outlines enhanced specifications for the Squirrel CLI system, designed to expand its capabilities, improve integration with other components, and create a more powerful and flexible command-line experience. These enhancements build upon the existing CLI implementation while introducing new architectural patterns and capabilities.

## Current Limitations

The current CLI implementation has several limitations that these enhancements aim to address:

1. Limited integration with the core command system (`specs/commands/`)
2. Basic command execution without composition or pipelining
3. Partial plugin system implementation without dynamic discovery
4. Limited scripting and automation capabilities
5. Basic MCP integration without bidirectional communication
6. Limited state management across command executions

## Enhanced Architecture

The proposed architecture expands the CLI system with the following structure:

```
cli/
├── core/               # Core CLI functionality
│   ├── command/        # Common command infrastructure
│   ├── ui/             # User interface components
│   └── registry/       # Enhanced command registry
├── commands/           # Command implementations
│   ├── core/           # Core commands (help, version, etc.)
│   ├── mcp/            # MCP-related commands
│   ├── plugin/         # Plugin-related commands
│   └── scripting/      # Scripting commands
├── plugins/            # Plugin system
│   ├── loader/         # Plugin loading and management
│   ├── api/            # Plugin API definitions
│   └── sandbox/        # Plugin isolation
├── scripting/          # Scripting engine
│   ├── parser/         # Script parsing
│   ├── executor/       # Script execution
│   └── stdlib/         # Standard library for scripts
└── integrations/       # External system integrations
    ├── mcp/            # MCP integration
    ├── filesystem/     # File system operations
    └── network/        # Network operations
```

## Key Enhancements

### 1. Enhanced Command Registry

The command registry will be enhanced with middleware support, hierarchical organization, and improved discovery:

```rust
pub struct CommandRegistry {
    // Store commands hierarchically
    commands: HashMap<String, Arc<dyn Command>>,
    
    // Support command groups
    groups: HashMap<String, CommandGroup>,
    
    // Track command history
    history: VecDeque<CommandExecution>,
    
    // Support middleware
    middleware: Vec<Box<dyn CommandMiddleware>>,
    
    // Plugin support
    plugin_registry: Arc<PluginRegistry>,
}

impl CommandRegistry {
    // Register a command with middleware support
    pub fn register(&mut self, command: Arc<dyn Command>) -> Result<()> {
        // Command validation and registration logic
        // ...
        Ok(())
    }
    
    // Execute a command with middleware chain
    pub fn execute(&self, name: &str, args: &[String]) -> Result<CommandOutput> {
        // Find the command
        let command = self.find_command(name)?;
        
        // Create execution context
        let mut context = ExecutionContext::new(args);
        
        // Apply pre-execution middleware
        for middleware in &self.middleware {
            middleware.before_execution(&mut context)?;
        }
        
        // Execute the command
        let result = command.execute(&context)?;
        
        // Apply post-execution middleware
        let mut output = CommandOutput::new(result);
        for middleware in &self.middleware {
            middleware.after_execution(&mut output)?;
        }
        
        // Record in history
        self.record_execution(name, args, &output);
        
        Ok(output)
    }
    
    // Command discovery API
    pub fn discover(&mut self) -> Result<Vec<CommandInfo>> {
        // Implementation logic
        // ...
        Ok(Vec::new())
    }
}
```

#### Command Groups

Commands can be organized into logical groups:

```rust
pub struct CommandGroup {
    name: String,
    description: String,
    commands: HashSet<String>,
}

impl CommandGroup {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            commands: HashSet::new(),
        }
    }
    
    pub fn add_command(&mut self, command_name: &str) {
        self.commands.insert(command_name.to_string());
    }
    
    pub fn commands(&self) -> impl Iterator<Item = &String> {
        self.commands.iter()
    }
}
```

#### Command Middleware

The middleware pattern allows for cross-cutting concerns:

```rust
pub trait CommandMiddleware: Send + Sync {
    /// Called before command execution
    fn before_execution(&self, context: &mut ExecutionContext) -> Result<()>;
    
    /// Called after command execution
    fn after_execution(&self, output: &mut CommandOutput) -> Result<()>;
}

/// Example middleware: logging
pub struct LoggingMiddleware {
    log_level: log::Level,
}

impl CommandMiddleware for LoggingMiddleware {
    fn before_execution(&self, context: &mut ExecutionContext) -> Result<()> {
        log::log!(self.log_level, "Executing command: {} with args: {:?}", 
            context.command_name(), context.args());
        Ok(())
    }
    
    fn after_execution(&self, output: &mut CommandOutput) -> Result<()> {
        log::log!(self.log_level, "Command result: {:?}", output.status());
        Ok(())
    }
}
```

### 2. Command Composability

Enable command composition through pipelines:

```rust
pub struct CommandPipeline {
    stages: Vec<PipelineStage>,
}

pub struct PipelineStage {
    command: String,
    args: Vec<String>,
}

impl CommandPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }
    
    pub fn add_stage(&mut self, command: &str, args: Vec<String>) -> &mut Self {
        self.stages.push(PipelineStage {
            command: command.to_string(),
            args,
        });
        self
    }
    
    pub fn execute(&self, registry: &CommandRegistry) -> Result<CommandOutput> {
        if self.stages.is_empty() {
            return Err(Error::EmptyPipeline);
        }
        
        let mut current_output: Option<CommandOutput> = None;
        
        for stage in &self.stages {
            // Clone the args to avoid borrowing issues
            let mut stage_args = stage.args.clone();
            
            // If we have output from previous stage, add it as input
            if let Some(prev_output) = current_output {
                match prev_output.format() {
                    OutputFormat::Text => {
                        stage_args.push(prev_output.to_string());
                    }
                    OutputFormat::Json => {
                        stage_args.push(format!("--input-json={}", prev_output.to_json()?));
                    }
                    // Other formats...
                }
            }
            
            // Execute the current stage
            current_output = Some(registry.execute(&stage.command, &stage_args)?);
        }
        
        // Return the output of the final stage
        current_output.ok_or(Error::PipelineExecutionFailed)
    }
}
```

### 3. Advanced Scripting Engine

Implement a powerful scripting engine for CLI automation:

```rust
pub struct ScriptEngine {
    registry: Arc<CommandRegistry>,
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    current_script: Option<Script>,
}

impl ScriptEngine {
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self {
            registry,
            variables: HashMap::new(),
            functions: HashMap::new(),
            current_script: None,
        }
    }
    
    pub fn execute_script(&mut self, script_text: &str) -> Result<ScriptResult> {
        // Parse the script
        let script = self.parse_script(script_text)?;
        
        // Set as current script
        self.current_script = Some(script.clone());
        
        // Execute the script
        let result = self.execute(&script)?;
        
        // Clear current script
        self.current_script = None;
        
        Ok(result)
    }
    
    pub fn load_script_file(&mut self, path: &Path) -> Result<ScriptResult> {
        // Read the file
        let script_text = fs::read_to_string(path)?;
        
        // Execute the script
        self.execute_script(&script_text)
    }
    
    fn parse_script(&self, script_text: &str) -> Result<Script> {
        // Implementation of script parsing
        // ...
        Ok(Script::default())
    }
    
    fn execute(&mut self, script: &Script) -> Result<ScriptResult> {
        // Implementation of script execution
        // ...
        Ok(ScriptResult::default())
    }
}
```

#### Script Structure

Scripts will support a variety of control structures and commands:

```rust
pub struct Script {
    statements: Vec<Statement>,
    functions: HashMap<String, Function>,
}

pub enum Statement {
    Command(CommandStatement),
    Variable(VariableStatement),
    Conditional(ConditionalStatement),
    Loop(LoopStatement),
    FunctionCall(FunctionCallStatement),
    Pipeline(PipelineStatement),
}

pub struct CommandStatement {
    command: String,
    args: Vec<Expression>,
}

pub struct ConditionalStatement {
    condition: Expression,
    then_block: Vec<Statement>,
    else_block: Option<Vec<Statement>>,
}

pub struct LoopStatement {
    kind: LoopKind,
    body: Vec<Statement>,
}

pub enum LoopKind {
    For { variable: String, iterable: Expression },
    While { condition: Expression },
}
```

#### Script Standard Library

The scripting engine will include a comprehensive standard library:

```rust
pub struct StdLib {
    functions: HashMap<String, NativeFunction>,
}

impl StdLib {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        // File system functions
        functions.insert("read_file".to_string(), NativeFunction::new(Self::read_file));
        functions.insert("write_file".to_string(), NativeFunction::new(Self::write_file));
        functions.insert("file_exists".to_string(), NativeFunction::new(Self::file_exists));
        
        // String manipulation
        functions.insert("concat".to_string(), NativeFunction::new(Self::concat));
        functions.insert("split".to_string(), NativeFunction::new(Self::split));
        functions.insert("trim".to_string(), NativeFunction::new(Self::trim));
        
        // System interactions
        functions.insert("get_env".to_string(), NativeFunction::new(Self::get_env));
        functions.insert("set_env".to_string(), NativeFunction::new(Self::set_env));
        functions.insert("sleep".to_string(), NativeFunction::new(Self::sleep));
        
        Self { functions }
    }
    
    // Function implementations
    fn read_file(args: &[Value]) -> Result<Value> {
        // Implementation
        // ...
        Ok(Value::String(String::new()))
    }
    
    // Other function implementations...
}
```

### 4. Enhanced Plugin System

Expand the plugin system with dynamic loading, dependency management, and sandboxing:

```rust
pub struct PluginRegistry {
    plugins: HashMap<String, LoadedPlugin>,
    search_paths: Vec<PathBuf>,
    sandbox_manager: SandboxManager,
}

pub struct LoadedPlugin {
    metadata: PluginMetadata,
    instance: Arc<dyn Plugin>,
    state: PluginState,
    dependencies: Vec<PluginDependency>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            search_paths: vec![],
            sandbox_manager: SandboxManager::new(),
        }
    }
    
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    pub fn discover_plugins(&mut self) -> Result<Vec<PluginMetadata>> {
        let mut discovered = Vec::new();
        
        for path in &self.search_paths {
            let entries = fs::read_dir(path)?;
            
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                
                if self.is_plugin(&path) {
                    if let Some(metadata) = self.read_plugin_metadata(&path)? {
                        discovered.push(metadata);
                    }
                }
            }
        }
        
        Ok(discovered)
    }
    
    pub fn load_plugin(&mut self, name: &str) -> Result<&LoadedPlugin> {
        // Check if already loaded
        if self.plugins.contains_key(name) {
            return Ok(&self.plugins[name]);
        }
        
        // Find plugin
        let plugin_path = self.find_plugin(name)?;
        
        // Read metadata
        let metadata = self.read_plugin_metadata(&plugin_path)?
            .ok_or_else(|| Error::InvalidPlugin(name.to_string()))?;
        
        // Check dependencies
        for dep in &metadata.dependencies {
            self.load_plugin(&dep.name)?;
        }
        
        // Load plugin in sandbox
        let instance = self.sandbox_manager.load_plugin(&plugin_path)?;
        
        // Initialize plugin
        instance.initialize()?;
        
        // Store loaded plugin
        let loaded = LoadedPlugin {
            metadata: metadata.clone(),
            instance,
            state: PluginState::Loaded,
            dependencies: metadata.dependencies.clone(),
        };
        
        self.plugins.insert(name.to_string(), loaded);
        
        Ok(&self.plugins[name])
    }
    
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        // Implementation
        // ...
        Ok(())
    }
    
    // Helper methods...
}
```

#### Plugin Sandboxing

Ensure plugins run in isolated environments:

```rust
pub struct SandboxManager {
    sandboxes: HashMap<String, Sandbox>,
}

pub struct Sandbox {
    id: String,
    resource_limits: ResourceLimits,
    permissions: Permissions,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            sandboxes: HashMap::new(),
        }
    }
    
    pub fn load_plugin(&mut self, path: &Path) -> Result<Arc<dyn Plugin>> {
        // Create sandbox for plugin
        let plugin_name = path.file_stem().unwrap().to_string_lossy().to_string();
        let sandbox = self.create_sandbox(&plugin_name)?;
        
        // Load plugin in sandbox
        let plugin = sandbox.load_plugin(path)?;
        
        // Store sandbox
        self.sandboxes.insert(plugin_name, sandbox);
        
        Ok(plugin)
    }
    
    fn create_sandbox(&self, name: &str) -> Result<Sandbox> {
        // Implementation
        // ...
        Ok(Sandbox {
            id: name.to_string(),
            resource_limits: ResourceLimits::default(),
            permissions: Permissions::default(),
        })
    }
}

impl Sandbox {
    pub fn load_plugin(&self, path: &Path) -> Result<Arc<dyn Plugin>> {
        // Implementation using libloading or similar
        // ...
        Ok(Arc::new(DummyPlugin {}))
    }
}
```

### 5. Enhanced MCP Integration

Improve MCP integration with bidirectional communication and advanced features:

```rust
pub struct McpClient {
    connection: Option<McpConnection>,
    config: McpClientConfig,
    session: Option<McpSession>,
}

pub struct McpConnection {
    websocket: WebSocketClient,
    address: String,
    status: ConnectionStatus,
}

impl McpClient {
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            connection: None,
            config,
            session: None,
        }
    }
    
    pub async fn connect(&mut self, address: &str) -> Result<()> {
        // Implementation
        // ...
        Ok(())
    }
    
    pub async fn send_message(&mut self, message: McpMessage) -> Result<McpResponse> {
        // Implementation
        // ...
        Ok(McpResponse::default())
    }
    
    pub async fn start_interactive_session(&mut self) -> Result<McpSession> {
        // Implementation
        // ...
        let session = McpSession::new(self.connection.as_ref().unwrap().clone());
        self.session = Some(session.clone());
        Ok(session)
    }
    
    pub async fn subscribe_to_events(&mut self, event_type: &str) -> Result<EventSubscription> {
        // Implementation
        // ...
        Ok(EventSubscription::default())
    }
}

pub struct McpSession {
    connection: McpConnection,
    history: VecDeque<McpMessage>,
    prompt_handler: Option<Box<dyn PromptHandler>>,
}

impl McpSession {
    pub fn new(connection: McpConnection) -> Self {
        Self {
            connection,
            history: VecDeque::new(),
            prompt_handler: None,
        }
    }
    
    pub fn set_prompt_handler(&mut self, handler: Box<dyn PromptHandler>) {
        self.prompt_handler = Some(handler);
    }
    
    pub async fn run_interactive(&mut self) -> Result<()> {
        // Implementation of interactive mode
        // ...
        Ok(())
    }
}
```

### 6. CLI State Management

Implement robust state management across command executions:

```rust
pub struct CliState {
    config: Config,
    context: CommandContext,
    session: Session,
    environment: Environment,
}

impl CliState {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            context: CommandContext::default(),
            session: Session::default(),
            environment: Environment::default(),
        }
    }
    
    pub fn save(&self) -> Result<()> {
        // Save state to disk
        let state_dir = self.get_state_dir()?;
        fs::create_dir_all(&state_dir)?;
        
        // Save config
        let config_path = state_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&self.config)?;
        fs::write(config_path, config_json)?;
        
        // Save context
        let context_path = state_dir.join("context.json");
        let context_json = serde_json::to_string_pretty(&self.context)?;
        fs::write(context_path, context_json)?;
        
        // Save session
        let session_path = state_dir.join("session.json");
        let session_json = serde_json::to_string_pretty(&self.session)?;
        fs::write(session_path, session_json)?;
        
        Ok(())
    }
    
    pub fn load() -> Result<Self> {
        // Load state from disk
        let state_dir = Self::get_default_state_dir()?;
        
        // Load config
        let config = if let Ok(config_json) = fs::read_to_string(state_dir.join("config.json")) {
            serde_json::from_str(&config_json)?
        } else {
            Config::default()
        };
        
        // Load context
        let context = if let Ok(context_json) = fs::read_to_string(state_dir.join("context.json")) {
            serde_json::from_str(&context_json)?
        } else {
            CommandContext::default()
        };
        
        // Load session
        let session = if let Ok(session_json) = fs::read_to_string(state_dir.join("session.json")) {
            serde_json::from_str(&session_json)?
        } else {
            Session::default()
        };
        
        // Create environment
        let environment = Environment::from_current()?;
        
        Ok(Self {
            config,
            context,
            session,
            environment,
        })
    }
    
    fn get_state_dir(&self) -> Result<PathBuf> {
        Self::get_default_state_dir()
    }
    
    fn get_default_state_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| Error::HomeDirectoryNotFound)?;
        Ok(home_dir.join(".squirrel").join("state"))
    }
}
```

## Implementation Path

The implementation will follow a phased approach:

### Phase 1: Architecture Foundation (2 weeks)
1. Refine the command registry with middleware support
   - Implement the `CommandMiddleware` trait
   - Add logging and telemetry middleware
   - Integrate with the existing command system
   
2. Establish clear integration between CLI and command specs
   - Create bridges between `specs/cli/` and `specs/commands/`
   - Standardize command interfaces
   - Document integration points
   
3. Enhance the plugin framework to support dynamic loading
   - Implement the basic `PluginRegistry`
   - Add plugin discovery
   - Create plugin metadata handling

### Phase 2: Command System Enhancement (3 weeks)
1. Implement command composition and pipelines
   - Create the `CommandPipeline` structure
   - Add support for command output to command input
   - Implement pipeline execution
   
2. Add advanced parameter support with strong typing
   - Enhance parameter parsing
   - Add validation rules
   - Support complex parameter types
   
3. Create a structured output system with multiple formats
   - Implement consistent output formatting
   - Add support for JSON, YAML, and table formats
   - Create filtering and transformation capabilities

### Phase 3: Scripting and Automation (4 weeks)
1. Develop the scripting engine
   - Implement script parsing
   - Create the execution engine
   - Add variable and function support
   
2. Add script debugging capabilities
   - Create a debugger interface
   - Add breakpoints and stepping
   - Implement variable inspection
   
3. Implement task automation features
   - Add scheduled task support
   - Create task templating
   - Implement task versioning

### Phase 4: Integration and Extension (3 weeks)
1. Enhance MCP integration with bidirectional communication
   - Implement WebSocket client
   - Add interactive sessions
   - Create event subscription
   
2. Implement plugin sandboxing
   - Create the sandbox manager
   - Add resource limits
   - Implement permission control
   
3. Add comprehensive scripting standard library
   - Implement file system functions
   - Add string manipulation
   - Create network operations

## Success Criteria

The enhanced CLI system will be considered successful when it:

1. Provides seamless integration with the core command system
2. Supports command composition through pipelines
3. Enables dynamic plugin discovery and loading
4. Offers a powerful scripting language for automation
5. Maintains robust state across command executions
6. Provides bidirectional communication with MCP servers
7. Ensures security through proper sandboxing

## Metrics

The following metrics will be used to evaluate the enhanced CLI system:

1. **Command execution latency**: < 50ms for simple commands
2. **Scripting performance**: < 100ms overhead for script execution
3. **Plugin load time**: < 200ms for plugin discovery and loading
4. **Memory usage**: < 50MB increase over the base CLI
5. **Pipeline throughput**: > 1000 commands/second for simple pipelines
6. **State persistence**: < 100ms for state save/load operations
7. **MCP communication**: < 20ms round-trip for local MCP servers

## Integration Points

The enhanced CLI system integrates with several other components:

1. **Core Command System**: Uses the command definitions and patterns
2. **MCP Protocol**: Communicates with MCP servers
3. **Plugin System**: Loads and manages plugins
4. **Scripting Engine**: Executes automation scripts
5. **State Management**: Maintains CLI state across executions

## Future Work

Beyond the scope of this specification, future work could include:

1. **Remote CLI**: Support for executing commands on remote systems
2. **GUI Integration**: Bridge between CLI and GUI interfaces
3. **Cloud Integration**: Support for cloud-based execution environments
4. **Natural Language Interface**: Support for natural language command input
5. **AI-Assisted Automation**: Integration with AI for command suggestions and automation

## Appendix: Example Usage

### Command Pipeline Example

```bash
# Find all markdown files, filter for those with "spec" in the name, and convert to HTML
squirrel find --type md | squirrel filter --pattern "spec" | squirrel convert --to html --output specs.html
```

### Script Example

```
# Example Squirrel CLI script

# Define variables
let output_dir = "./output"
let format = "json"

# Create output directory if it doesn't exist
if (!file_exists(output_dir)) {
    run("mkdir", ["-p", output_dir])
}

# Get system status and save to file
let status = run("status", ["--format", format])
write_file("${output_dir}/status.${format}", status)

# Loop through all plugins and get their status
for plugin in run("plugin", ["list"]) {
    let plugin_status = run("plugin", ["info", plugin, "--format", format])
    write_file("${output_dir}/plugin_${plugin}.${format}", plugin_status)
}

# Define a function
function check_mcp_server(host, port) {
    try {
        let result = run("mcp", ["status", "--host", host, "--port", port.toString()])
        return true
    } catch (e) {
        return false
    }
}

# Use the function
if (check_mcp_server("localhost", 7777)) {
    println("MCP server is running")
} else {
    println("MCP server is not running")
}
```

<version>1.0.0</version> 