---
title: CLI Scripting Capabilities Specification
version: 1.0.0
date: 2024-04-15
status: proposal
priority: high
author: DataScienceBioLab
---

# CLI Scripting Capabilities Specification

## Overview

This document defines the scripting capabilities for the Squirrel CLI system, providing advanced automation and extensibility features. The scripting engine will allow users to create complex workflows, automate repetitive tasks, and extend the CLI's functionality without modifying the core codebase.

## Design Goals

1. **Simplicity**: Easy to learn for users familiar with common scripting languages
2. **Power**: Expressive enough to handle complex automation tasks
3. **Integration**: Seamless integration with CLI commands and plugins
4. **Safety**: Secure execution environment with appropriate permissions
5. **Extensibility**: Ability to add new functions and capabilities

## Scripting Language Features

### Core Language Elements

The scripting language will support:

1. **Variables and Data Types**
   - Strings, numbers, booleans, arrays, objects
   - Strong typing with type inference
   - Variable declaration and assignment

2. **Control Structures**
   - Conditionals (if/else, switch)
   - Loops (for, while, foreach)
   - Error handling (try/catch/finally)

3. **Functions**
   - Function declaration and calling
   - Anonymous functions
   - Closures
   - Return values and error handling

4. **Modules**
   - Import/export capabilities
   - Namespace management
   - Dependency resolution

### Command Integration

The scripting language will provide native integration with CLI commands:

```
# Example: Running a CLI command
let result = run("find", ["--type", "md", "--path", "./docs"])

# Example: Running a command with piped output
let result = pipe(
    run("find", ["--type", "md"]),
    run("filter", ["--pattern", "spec"]),
    run("convert", ["--to", "html"])
)

# Example: Accessing command output
let files = result.output.split("\n")
```

### Expression Syntax

The language will use a familiar syntax inspired by JavaScript/TypeScript:

```
# Variable declaration
let name = "value"
const MAX_RETRIES = 3

# Arithmetic
let sum = a + b
let product = a * b

# Conditionals
if (condition) {
    # do something
} else {
    # do something else
}

# Loops
for (let i = 0; i < 10; i++) {
    # do something
}

while (condition) {
    # do something
}

for item in items {
    # do something with item
}

# Functions
function add(a, b) {
    return a + b
}

# Anonymous functions
let multiply = (a, b) => a * b
```

## Standard Library

The scripting engine will include a standard library with the following modules:

### File System Operations

```
# Reading and writing files
let content = fs.readFile("path/to/file")
fs.writeFile("path/to/file", content)

# Directory operations
let files = fs.listDir("path/to/dir")
fs.createDir("path/to/new/dir")
fs.removeDir("path/to/dir")

# Path manipulation
let fullPath = fs.joinPath(dir, filename)
let extension = fs.getExtension(path)
```

### String Manipulation

```
# String operations
let upper = str.toUpper(text)
let lower = str.toLower(text)
let parts = str.split(text, delimiter)
let joined = str.join(parts, delimiter)
let replaced = str.replace(text, pattern, replacement)
```

### System Interaction

```
# Environment variables
let path = sys.getEnv("PATH")
sys.setEnv("DEBUG", "true")

# Process management
let output = sys.exec("external-command", ["arg1", "arg2"])
let process = sys.spawn("long-running-process")
process.wait()

# Timing
sys.sleep(1000)  # Sleep for 1000ms
let start = sys.now()
```

### Network Operations

```
# HTTP requests
let response = http.get("https://example.com/api")
let data = http.post("https://example.com/api", { key: "value" })

# WebSocket
let ws = websocket.connect("wss://example.com/socket")
ws.onMessage(message => {
    # Handle message
})
```

### Data Processing

```
# JSON handling
let obj = json.parse(text)
let text = json.stringify(obj)

# CSV handling
let records = csv.parse(text)
let text = csv.stringify(records)

# Data filtering
let filtered = data.filter(items, item => item.value > 10)
let mapped = data.map(items, item => item.name)
```

## Scripting Engine Architecture

The scripting engine will be implemented with the following components:

### Parser

The parser will convert script text into an abstract syntax tree (AST):

```rust
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Script> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        
        Ok(Script { 
            statements,
            functions: HashMap::new(),
        })
    }
    
    fn statement(&mut self) -> Result<Statement> {
        // Implementation of statement parsing logic
        // ...
    }
    
    // Other parsing methods...
}
```

### Executor

The executor will evaluate the AST and produce results:

```rust
pub struct Executor {
    environment: Environment,
    registry: Arc<CommandRegistry>,
}

impl Executor {
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self {
            environment: Environment::new(),
            registry,
        }
    }
    
    pub fn execute(&mut self, script: &Script) -> Result<Value> {
        let mut last_value = Value::Null;
        
        // Register functions
        for (name, function) in &script.functions {
            self.environment.define(name, Value::Function(function.clone()));
        }
        
        // Execute statements
        for statement in &script.statements {
            last_value = self.execute_statement(statement)?;
        }
        
        Ok(last_value)
    }
    
    fn execute_statement(&mut self, statement: &Statement) -> Result<Value> {
        match statement {
            Statement::Expression(expr) => self.evaluate(expr),
            Statement::Variable(var) => self.execute_variable(var),
            Statement::If(if_stmt) => self.execute_if(if_stmt),
            Statement::While(while_stmt) => self.execute_while(while_stmt),
            Statement::For(for_stmt) => self.execute_for(for_stmt),
            Statement::Function(func) => self.execute_function(func),
            Statement::Return(ret) => self.execute_return(ret),
            Statement::Block(block) => self.execute_block(block),
            Statement::Try(try_stmt) => self.execute_try(try_stmt),
        }
    }
    
    // Other execution methods...
}
```

### Environment

The environment will track variables and their values:

```rust
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }
    
    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }
    
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }
    
    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(Error::UndefinedVariable(name.to_string()))
        }
    }
    
    pub fn assign(&mut self, name: &str, value: Value) -> Result<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(Error::UndefinedVariable(name.to_string()))
        }
    }
}
```

### Standard Library Implementation

The standard library modules will be implemented as native functions:

```rust
pub struct StdLib {
    modules: HashMap<String, Module>,
}

impl StdLib {
    pub fn new() -> Self {
        let mut modules = HashMap::new();
        
        // File system module
        modules.insert("fs".to_string(), Self::create_fs_module());
        
        // String module
        modules.insert("str".to_string(), Self::create_str_module());
        
        // System module
        modules.insert("sys".to_string(), Self::create_sys_module());
        
        // Network module
        modules.insert("http".to_string(), Self::create_http_module());
        
        // Data module
        modules.insert("data".to_string(), Self::create_data_module());
        
        Self { modules }
    }
    
    pub fn register_in_environment(&self, env: &mut Environment) {
        for (name, module) in &self.modules {
            env.define(name, Value::Module(module.clone()));
        }
    }
    
    fn create_fs_module() -> Module {
        let mut module = Module::new("fs");
        
        // Register file system functions
        module.add_function("readFile", NativeFunction::new(Self::fs_read_file));
        module.add_function("writeFile", NativeFunction::new(Self::fs_write_file));
        module.add_function("listDir", NativeFunction::new(Self::fs_list_dir));
        // ... other functions
        
        module
    }
    
    // Implementation of native functions
    fn fs_read_file(args: &[Value]) -> Result<Value> {
        // Validate arguments
        if args.len() != 1 {
            return Err(Error::InvalidArgumentCount);
        }
        
        let path = args[0].as_string()?;
        
        // Read file
        match fs::read_to_string(path) {
            Ok(content) => Ok(Value::String(content)),
            Err(err) => Err(Error::IoError(err.to_string())),
        }
    }
    
    // Other module creation and function implementations...
}
```

## Script Debugging

The scripting engine will include debugging capabilities:

### Debugger Interface

```rust
pub struct Debugger {
    executor: Executor,
    breakpoints: HashSet<BreakpointLocation>,
    current_location: Option<SourceLocation>,
    state: DebuggerState,
}

impl Debugger {
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
            breakpoints: HashSet::new(),
            current_location: None,
            state: DebuggerState::Running,
        }
    }
    
    pub fn set_breakpoint(&mut self, location: BreakpointLocation) {
        self.breakpoints.insert(location);
    }
    
    pub fn remove_breakpoint(&mut self, location: &BreakpointLocation) {
        self.breakpoints.remove(location);
    }
    
    pub fn start(&mut self, script: &Script) -> Result<Value> {
        self.state = DebuggerState::Running;
        let result = self.execute_with_debugging(script);
        self.state = DebuggerState::Stopped;
        result
    }
    
    pub fn step_over(&mut self) -> Result<DebuggerEvent> {
        // Implementation
        // ...
        Ok(DebuggerEvent::StepCompleted)
    }
    
    pub fn step_into(&mut self) -> Result<DebuggerEvent> {
        // Implementation
        // ...
        Ok(DebuggerEvent::StepCompleted)
    }
    
    pub fn step_out(&mut self) -> Result<DebuggerEvent> {
        // Implementation
        // ...
        Ok(DebuggerEvent::StepCompleted)
    }
    
    pub fn continue_execution(&mut self) -> Result<DebuggerEvent> {
        // Implementation
        // ...
        Ok(DebuggerEvent::BreakpointHit(self.current_location.unwrap()))
    }
    
    pub fn get_variables(&self) -> Result<HashMap<String, Value>> {
        // Implementation
        // ...
        Ok(HashMap::new())
    }
    
    // Other debugging methods...
}
```

### Breakpoint Support

```rust
pub struct BreakpointLocation {
    script_name: String,
    line: usize,
    column: Option<usize>,
}

pub struct SourceLocation {
    script_name: String,
    line: usize,
    column: usize,
}

pub enum DebuggerState {
    Running,
    Paused,
    Stopped,
}

pub enum DebuggerEvent {
    BreakpointHit(SourceLocation),
    StepCompleted,
    Exception(Error, SourceLocation),
    Terminated,
}
```

## Script Testing

To ensure script reliability, the system will include testing capabilities:

```rust
pub struct TestRunner {
    engine: ScriptEngine,
    results: Vec<TestResult>,
}

impl TestRunner {
    pub fn new(engine: ScriptEngine) -> Self {
        Self {
            engine,
            results: Vec::new(),
        }
    }
    
    pub fn run_tests(&mut self, script_path: &Path) -> Result<TestSummary> {
        // Load and parse the script
        let script_text = fs::read_to_string(script_path)?;
        let script = self.engine.parse_script(&script_text)?;
        
        // Find test functions (functions starting with "test_")
        let test_functions = self.find_test_functions(&script);
        
        let mut results = Vec::new();
        
        // Run each test function
        for test_func in test_functions {
            let result = self.run_test(test_func, &script);
            results.push(result);
        }
        
        self.results = results.clone();
        
        // Create summary
        let passed = results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed = results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let errored = results.iter().filter(|r| r.status == TestStatus::Error).count();
        
        Ok(TestSummary {
            total: results.len(),
            passed,
            failed,
            errored,
            results,
        })
    }
    
    fn run_test(&self, test_name: &str, script: &Script) -> TestResult {
        // Implementation of test running
        // ...
        TestResult {
            name: test_name.to_string(),
            status: TestStatus::Passed,
            message: None,
            duration: Duration::from_millis(0),
        }
    }
    
    // Other test running methods...
}
```

## Integration with CLI Commands

The scripting engine will seamlessly integrate with CLI commands:

```rust
pub struct ScriptCommand {
    name: String,
    script_path: PathBuf,
}

impl Command for ScriptCommand {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn execute(&self, context: &ExecutionContext) -> Result<CommandOutput> {
        // Create script engine
        let mut engine = ScriptEngine::new(context.registry().clone());
        
        // Load and execute script
        let result = engine.load_script_file(&self.script_path)?;
        
        // Convert result to command output
        Ok(CommandOutput::from(result))
    }
    
    fn help(&self) -> CommandHelp {
        CommandHelp {
            name: self.name.clone(),
            description: format!("Script command from file: {}", self.script_path.display()),
            usage: format!("{} [arguments...]", self.name),
            examples: vec![],
            args: vec![],
            subcommands: vec![],
        }
    }
}
```

## Security Considerations

The scripting engine will implement security measures:

1. **Sandboxed Execution**
   - Scripts run in isolated environments with limited access to the system
   - Resource limits (memory, CPU, execution time)
   - File system access controls

2. **Permission System**
   - Scripts require explicit permissions for sensitive operations
   - Users can grant or deny permissions at runtime
   - Persistent permission settings for trusted scripts

3. **Script Signing**
   - Support for signed scripts to verify authenticity
   - Integration with plugin system for trust verification
   - Warning for unsigned scripts

```rust
pub struct ScriptSecurity {
    permissions: Permissions,
    resource_limits: ResourceLimits,
    signature_verifier: Option<SignatureVerifier>,
}

impl ScriptSecurity {
    pub fn new() -> Self {
        Self {
            permissions: Permissions::default(),
            resource_limits: ResourceLimits::default(),
            signature_verifier: None,
        }
    }
    
    pub fn check_permission(&self, permission: Permission) -> Result<()> {
        if self.permissions.has(permission) {
            Ok(())
        } else {
            Err(Error::PermissionDenied(permission))
        }
    }
    
    pub fn verify_script(&self, script_path: &Path) -> Result<VerificationResult> {
        if let Some(verifier) = &self.signature_verifier {
            verifier.verify(script_path)
        } else {
            // No verifier, return unverified
            Ok(VerificationResult::Unverified)
        }
    }
    
    pub fn set_resource_limit(&mut self, limit: ResourceLimit) {
        self.resource_limits.set(limit);
    }
}

pub struct Permissions {
    file_read: bool,
    file_write: bool,
    network: bool,
    environment: bool,
    command_execution: bool,
}

pub enum Permission {
    FileRead(PathBuf),
    FileWrite(PathBuf),
    Network(String),
    Environment(String),
    CommandExecution(String),
}

pub struct ResourceLimits {
    max_memory: usize,
    max_execution_time: Duration,
    max_file_size: usize,
}

pub enum VerificationResult {
    Verified(ScriptSignature),
    Unverified,
    Invalid(String),
}
```

## Scripting Tools

The CLI will include tools for script development:

1. **Script Creation**
   - Commands to create new script templates
   - Scaffolding for common script types

2. **Script Validation**
   - Syntax checking and linting
   - Best practice recommendations

3. **Script Documentation**
   - Documentation generation from script comments
   - Example usage and parameter descriptions

```rust
pub struct ScriptTool {
    // Tool implementation
}

impl ScriptTool {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn create_template(&self, template_type: &str, output_path: &Path) -> Result<()> {
        // Implementation of template creation
        // ...
        Ok(())
    }
    
    pub fn validate(&self, script_path: &Path) -> Result<ValidationResult> {
        // Implementation of script validation
        // ...
        Ok(ValidationResult {
            valid: true,
            warnings: vec![],
            errors: vec![],
        })
    }
    
    pub fn generate_docs(&self, script_path: &Path, output_path: &Path) -> Result<()> {
        // Implementation of documentation generation
        // ...
        Ok(())
    }
}
```

## Implementation Path

The scripting capabilities will be implemented in the following phases:

### Phase 1: Core Engine (3 weeks)
1. Implement the parser and AST representation
2. Create the basic execution environment
3. Implement variable handling and simple expressions
4. Add support for control structures

### Phase 2: Standard Library (2 weeks)
1. Implement file system operations
2. Add string manipulation functions
3. Create system interaction capabilities
4. Implement basic CLI command integration

### Phase 3: Advanced Features (3 weeks)
1. Add full command integration with pipelines
2. Implement modules and imports
3. Create debugging capabilities
4. Add security features

### Phase 4: Tools and Testing (2 weeks)
1. Implement script testing framework
2. Create script tools for development
3. Add documentation generation
4. Build script examples and tutorials

## Success Criteria

The scripting capabilities will be considered successful when:

1. Users can create and run scripts to automate common tasks
2. Scripts can interact with CLI commands and other system components
3. The scripting language is easy to learn and understand
4. Scripts execute securely with appropriate permissions
5. Development tools assist users in creating high-quality scripts

## Appendix: Example Scripts

### Task Automation

```
# Automated documentation generation script

let SOURCE_DIR = "./src"
let DOCS_DIR = "./docs"

# Create docs directory if it doesn't exist
if (!fs.exists(DOCS_DIR)) {
    fs.mkdir(DOCS_DIR)
    println("Created documentation directory")
}

# Find all Rust files
let rust_files = run("find", [SOURCE_DIR, "--type", "file", "--name", "*.rs"])
    .output
    .split("\n")
    .filter(file => file.length > 0)

println(`Found ${rust_files.length} Rust files`)

# Generate documentation for each file
for file in rust_files {
    let basename = fs.basename(file, ".rs")
    let doc_file = fs.joinPath(DOCS_DIR, `${basename}.md`)
    
    println(`Generating documentation for ${basename}`)
    
    try {
        let result = run("doc-gen", [file, "--output", doc_file])
        println(`  Success: ${doc_file}`)
    } catch (e) {
        println(`  Error: ${e.message}`)
    }
}

println("Documentation generation complete")
```

### Data Processing

```
# Script to process log files and generate a report

let LOG_DIR = "./logs"
let OUTPUT_FILE = "./report.json"
let ERROR_PATTERN = "ERROR"

# Get all log files
let log_files = fs.listDir(LOG_DIR)
    .filter(file => file.endsWith(".log"))

println(`Processing ${log_files.length} log files`)

let errors = []

# Process each log file
for file_name in log_files {
    let file_path = fs.joinPath(LOG_DIR, file_name)
    let content = fs.readFile(file_path)
    
    # Split into lines
    let lines = content.split("\n")
    
    # Find error lines
    for line in lines {
        if (line.includes(ERROR_PATTERN)) {
            errors.push({
                file: file_name,
                line: line,
                timestamp: extractTimestamp(line)
            })
        }
    }
}

println(`Found ${errors.length} errors`)

# Sort errors by timestamp
errors.sort((a, b) => a.timestamp - b.timestamp)

# Generate report
let report = {
    total_files: log_files.length,
    total_errors: errors.length,
    errors: errors
}

# Write report to file
fs.writeFile(OUTPUT_FILE, json.stringify(report, null, 2))
println(`Report written to ${OUTPUT_FILE}`)

# Helper function to extract timestamp
function extractTimestamp(line) {
    let match = line.match(/\[(.*?)\]/)
    if (match) {
        return new Date(match[1]).getTime()
    }
    return 0
}
```

### System Monitoring

```
# System monitoring script

let INTERVAL = 5000  # 5 seconds
let ITERATIONS = 12  # Run for 1 minute
let OUTPUT_FILE = "./system_stats.json"

let stats = []

println("Starting system monitoring...")
println("Press Ctrl+C to stop")

for (let i = 0; i < ITERATIONS; i++) {
    # Get CPU stats
    let cpu = run("system", ["stats", "--cpu", "--format", "json"])
    let cpu_data = json.parse(cpu.output)
    
    # Get memory stats
    let memory = run("system", ["stats", "--memory", "--format", "json"])
    let memory_data = json.parse(memory.output)
    
    # Get disk stats
    let disk = run("system", ["stats", "--disk", "--format", "json"])
    let disk_data = json.parse(disk.output)
    
    # Combine stats
    let timestamp = sys.now()
    let combined = {
        timestamp: timestamp,
        cpu: cpu_data,
        memory: memory_data,
        disk: disk_data
    }
    
    # Add to stats array
    stats.push(combined)
    
    # Print current stats
    println(`[${new Date(timestamp).toISOString()}] CPU: ${cpu_data.usage}%, Memory: ${memory_data.used}/${memory_data.total} MB`)
    
    # Wait for next iteration
    if (i < ITERATIONS - 1) {
        sys.sleep(INTERVAL)
    }
}

# Save stats to file
fs.writeFile(OUTPUT_FILE, json.stringify(stats, null, 2))
println(`Statistics saved to ${OUTPUT_FILE}`)
```

<version>1.0.0</version> 