---
version: 1.0.0
last_updated: 2024-05-27
status: proposed
---

# Context and Command System Integration

## Overview

This document details the integration between the Context Management System and the Command System. The integration enables commands to access and modify contextual data, use context-aware execution flows, and properly manage state throughout command lifecycles.

## Integration Architecture

### Core Components

1. **Context-Aware Command Trait**
   - Extension of the base Command trait
   - Provides context requirements and access patterns
   - Manages context state updates

2. **Command Context Adapter**
   - Bridges the Command System and Context System
   - Handles context retrieval and updates
   - Manages context lifecycle during command execution

3. **Context Validation System**
   - Validates commands against context requirements
   - Ensures context integrity before and after execution
   - Prevents context corruption during command execution

4. **Context Change Tracking**
   - Tracks changes to context during command execution
   - Provides rollback capabilities for failed commands
   - Creates audit trail of context modifications

## Context-Aware Command

### Trait Definition

```rust
/// Trait for commands that are aware of context
pub trait ContextAwareCommand: Command {
    /// Gets the required context keys for this command
    fn required_context_keys(&self) -> HashSet<String>;
    
    /// Gets optional context keys that may enhance command behavior
    fn optional_context_keys(&self) -> HashSet<String>;
    
    /// Gets context keys this command will modify
    fn modified_context_keys(&self) -> HashSet<String>;
    
    /// Validates the context before command execution
    fn validate_context(&self, context: &dyn Context) -> Result<(), ContextValidationError>;
    
    /// Executes the command with context
    fn execute_with_context(&self, context: &mut dyn Context) -> Result<(), CommandError>;
    
    /// Rolls back context changes if command fails
    fn rollback_context_changes(&self, context: &mut dyn Context, changes: &ContextChanges) -> Result<(), ContextRollbackError>;
}
```

### Example Implementation

```rust
pub struct AnalysisCommand {
    name: String,
    description: String,
    analysis_type: AnalysisType,
    parameters: HashMap<String, Value>,
}

impl Command for AnalysisCommand {
    // Standard Command implementation
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    // Other required methods...
}

impl ContextAwareCommand for AnalysisCommand {
    fn required_context_keys(&self) -> HashSet<String> {
        HashSet::from([
            "current_project".into(),
            "active_dataset".into(),
            "user_preferences".into(),
        ])
    }
    
    fn optional_context_keys(&self) -> HashSet<String> {
        HashSet::from([
            "previous_analysis_results".into(),
            "saved_parameters".into(),
        ])
    }
    
    fn modified_context_keys(&self) -> HashSet<String> {
        HashSet::from([
            "analysis_history".into(),
            "last_analysis_result".into(),
        ])
    }
    
    fn validate_context(&self, context: &dyn Context) -> Result<(), ContextValidationError> {
        // Ensure required keys exist
        for key in self.required_context_keys() {
            if !context.has_key(&key) {
                return Err(ContextValidationError::MissingRequiredKey(key));
            }
        }
        
        // Validate dataset exists and is valid
        let dataset_id = context.get::<String>("active_dataset")?;
        if dataset_id.is_empty() {
            return Err(ContextValidationError::InvalidContextValue(
                "active_dataset".into(),
                "Dataset ID cannot be empty".into(),
            ));
        }
        
        // More validation logic...
        Ok(())
    }
    
    fn execute_with_context(&self, context: &mut dyn Context) -> Result<(), CommandError> {
        // Get required context values
        let project_id = context.get::<String>("current_project")?;
        let dataset_id = context.get::<String>("active_dataset")?;
        let preferences = context.get::<UserPreferences>("user_preferences")?;
        
        // Optional context values with defaults
        let previous_results = context
            .get::<Vec<AnalysisResult>>("previous_analysis_results")
            .unwrap_or_default();
        
        // Execute analysis
        let result = self.perform_analysis(project_id, dataset_id, &preferences, &previous_results)?;
        
        // Update context with results
        context.set("last_analysis_result", result.clone())?;
        
        // Append to history
        let mut history = context
            .get::<Vec<AnalysisResult>>("analysis_history")
            .unwrap_or_default();
        history.push(result);
        context.set("analysis_history", history)?;
        
        Ok(())
    }
    
    fn rollback_context_changes(&self, context: &mut dyn Context, changes: &ContextChanges) -> Result<(), ContextRollbackError> {
        // Restore previous values for modified keys
        for (key, value) in changes.previous_values() {
            if let Some(value) = value {
                context.set(key, value.clone())?;
            } else {
                context.remove(key)?;
            }
        }
        
        Ok(())
    }
}
```

## Command Context Adapter

### Adapter Implementation

```rust
/// Adapter for integrating commands with context
pub struct CommandContextAdapter {
    context_manager: Arc<dyn ContextManager>,
    change_tracker: Arc<ContextChangeTracker>,
}

impl CommandContextAdapter {
    /// Creates a new command context adapter
    pub fn new(context_manager: Arc<dyn ContextManager>) -> Self {
        Self {
            context_manager,
            change_tracker: Arc::new(ContextChangeTracker::new()),
        }
    }
    
    /// Executes a context-aware command
    pub async fn execute_command(&self, command: &dyn ContextAwareCommand) -> Result<(), CommandError> {
        // Get context for command
        let context_id = self.determine_context_id(command);
        let mut context = self.context_manager.get_context(context_id).await?;
        
        // Validate context
        command.validate_context(&context).map_err(|e| CommandError::ContextValidationFailed(e))?;
        
        // Start tracking changes
        let tracking_id = self.change_tracker.start_tracking(&context).await?;
        
        // Execute command with context
        let result = command.execute_with_context(&mut context);
        
        if result.is_ok() {
            // Commit changes
            self.change_tracker.commit_changes(tracking_id).await?;
            // Update context in manager
            self.context_manager.update_context(context_id, context).await?;
        } else {
            // Get changes for rollback
            let changes = self.change_tracker.get_changes(tracking_id).await?;
            // Rollback changes
            command.rollback_context_changes(&mut context, &changes)
                .map_err(|e| CommandError::ContextRollbackFailed(e))?;
            // Discard changes
            self.change_tracker.discard_changes(tracking_id).await?;
        }
        
        result
    }
    
    /// Determines the context ID for a command
    fn determine_context_id(&self, command: &dyn ContextAwareCommand) -> ContextId {
        // Logic to determine appropriate context ID for the command
        // Could be based on command name, category, or other factors
        // Default to global context
        ContextId::Global
    }
    
    /// Validates required context keys exist
    pub async fn validate_context_requirements(&self, command: &dyn ContextAwareCommand, context_id: ContextId) -> Result<(), ContextValidationError> {
        let context = self.context_manager.get_context(context_id).await?;
        command.validate_context(&context)
    }
    
    /// Preloads context for a batch of commands
    pub async fn preload_contexts(&self, commands: &[&dyn ContextAwareCommand]) -> Result<(), ContextError> {
        let mut required_contexts = HashSet::new();
        
        for command in commands {
            required_contexts.insert(self.determine_context_id(*command));
        }
        
        for context_id in required_contexts {
            self.context_manager.preload_context(context_id).await?;
        }
        
        Ok(())
    }
}
```

## Context Change Tracking

### Change Tracker Implementation

```rust
/// Tracks changes to context
pub struct ContextChangeTracker {
    active_tracking: Arc<RwLock<HashMap<String, ContextSnapshot>>>,
    changes: Arc<RwLock<HashMap<String, ContextChanges>>>,
}

impl ContextChangeTracker {
    /// Creates a new context change tracker
    pub fn new() -> Self {
        Self {
            active_tracking: Arc::new(RwLock::new(HashMap::new())),
            changes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Starts tracking changes to a context
    pub async fn start_tracking(&self, context: &dyn Context) -> Result<String, ContextError> {
        let tracking_id = Uuid::new_v4().to_string();
        let snapshot = context.create_snapshot()?;
        
        let mut active = self.active_tracking.write().await;
        active.insert(tracking_id.clone(), snapshot);
        
        Ok(tracking_id)
    }
    
    /// Gets changes made to a context
    pub async fn get_changes(&self, tracking_id: &str) -> Result<ContextChanges, ContextError> {
        let changes = self.changes.read().await;
        changes.get(tracking_id)
            .cloned()
            .ok_or_else(|| ContextError::TrackingNotFound(tracking_id.to_string()))
    }
    
    /// Records changes to a context
    pub async fn record_changes(&self, tracking_id: &str, context: &dyn Context) -> Result<(), ContextError> {
        let active = self.active_tracking.read().await;
        let snapshot = active.get(tracking_id)
            .ok_or_else(|| ContextError::TrackingNotFound(tracking_id.to_string()))?;
        
        let changes = context.diff_from_snapshot(snapshot)?;
        
        let mut changes_map = self.changes.write().await;
        changes_map.insert(tracking_id.to_string(), changes);
        
        Ok(())
    }
    
    /// Commits changes to a context
    pub async fn commit_changes(&self, tracking_id: &str) -> Result<(), ContextError> {
        let mut active = self.active_tracking.write().await;
        active.remove(tracking_id);
        
        let mut changes = self.changes.write().await;
        changes.remove(tracking_id);
        
        Ok(())
    }
    
    /// Discards changes to a context
    pub async fn discard_changes(&self, tracking_id: &str) -> Result<(), ContextError> {
        let mut active = self.active_tracking.write().await;
        active.remove(tracking_id);
        
        let mut changes = self.changes.write().await;
        changes.remove(tracking_id);
        
        Ok(())
    }
}
```

### Context Changes

```rust
/// Changes made to a context
pub struct ContextChanges {
    modified_keys: HashSet<String>,
    added_keys: HashSet<String>,
    removed_keys: HashSet<String>,
    previous_values: HashMap<String, Option<Box<dyn Any + Send + Sync>>>,
    current_values: HashMap<String, Option<Box<dyn Any + Send + Sync>>>,
}

impl ContextChanges {
    /// Creates a new context changes object
    pub fn new() -> Self {
        Self {
            modified_keys: HashSet::new(),
            added_keys: HashSet::new(),
            removed_keys: HashSet::new(),
            previous_values: HashMap::new(),
            current_values: HashMap::new(),
        }
    }
    
    /// Gets modified keys
    pub fn modified_keys(&self) -> &HashSet<String> {
        &self.modified_keys
    }
    
    /// Gets added keys
    pub fn added_keys(&self) -> &HashSet<String> {
        &self.added_keys
    }
    
    /// Gets removed keys
    pub fn removed_keys(&self) -> &HashSet<String> {
        &self.removed_keys
    }
    
    /// Gets all changed keys
    pub fn all_changed_keys(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        result.extend(self.modified_keys.iter().cloned());
        result.extend(self.added_keys.iter().cloned());
        result.extend(self.removed_keys.iter().cloned());
        result
    }
    
    /// Gets previous values
    pub fn previous_values(&self) -> &HashMap<String, Option<Box<dyn Any + Send + Sync>>> {
        &self.previous_values
    }
    
    /// Gets current values
    pub fn current_values(&self) -> &HashMap<String, Option<Box<dyn Any + Send + Sync>>> {
        &self.current_values
    }
    
    /// Adds a modification
    pub fn add_modification(
        &mut self,
        key: String,
        previous: Option<Box<dyn Any + Send + Sync>>,
        current: Option<Box<dyn Any + Send + Sync>>,
    ) {
        if previous.is_none() && current.is_some() {
            self.added_keys.insert(key.clone());
        } else if previous.is_some() && current.is_none() {
            self.removed_keys.insert(key.clone());
        } else {
            self.modified_keys.insert(key.clone());
        }
        
        self.previous_values.insert(key.clone(), previous);
        self.current_values.insert(key, current);
    }
}
```

## Context Validation

### Validation Rules

```rust
/// Error for context validation
#[derive(Debug, Clone)]
pub enum ContextValidationError {
    /// Required key is missing
    MissingRequiredKey(String),
    /// Context value is invalid
    InvalidContextValue(String, String),
    /// Context reference is invalid
    InvalidContextReference(String, String),
    /// Context type is invalid
    InvalidContextType(String, String, String),
    /// Custom validation failed
    CustomValidationFailed(String),
}

/// Context validation rule
pub trait ContextValidationRule: Send + Sync {
    /// Gets the name of the rule
    fn name(&self) -> &'static str;
    
    /// Gets the description of the rule
    fn description(&self) -> &'static str;
    
    /// Validates a context
    fn validate(&self, context: &dyn Context) -> Result<(), ContextValidationError>;
}

/// Required key validation rule
pub struct RequiredKeyRule {
    key: String,
    description: String,
}

impl RequiredKeyRule {
    /// Creates a new required key rule
    pub fn new(key: String) -> Self {
        Self {
            description: format!("Context must contain the key '{}'", key),
            key,
        }
    }
}

impl ContextValidationRule for RequiredKeyRule {
    fn name(&self) -> &'static str {
        "RequiredKey"
    }
    
    fn description(&self) -> &'static str {
        &self.description
    }
    
    fn validate(&self, context: &dyn Context) -> Result<(), ContextValidationError> {
        if !context.has_key(&self.key) {
            return Err(ContextValidationError::MissingRequiredKey(self.key.clone()));
        }
        
        Ok(())
    }
}

/// Type validation rule
pub struct TypeValidationRule<T: 'static + Send + Sync> {
    key: String,
    description: String,
    _phantom: PhantomData<T>,
}

impl<T: 'static + Send + Sync> TypeValidationRule<T> {
    /// Creates a new type validation rule
    pub fn new(key: String) -> Self {
        Self {
            description: format!("Context key '{}' must be of type '{}'", key, std::any::type_name::<T>()),
            key,
            _phantom: PhantomData,
        }
    }
}

impl<T: 'static + Send + Sync> ContextValidationRule for TypeValidationRule<T> {
    fn name(&self) -> &'static str {
        "TypeValidation"
    }
    
    fn description(&self) -> &'static str {
        &self.description
    }
    
    fn validate(&self, context: &dyn Context) -> Result<(), ContextValidationError> {
        if !context.has_key(&self.key) {
            return Ok(());  // Skip validation if key doesn't exist
        }
        
        if !context.is_type::<T>(&self.key) {
            return Err(ContextValidationError::InvalidContextType(
                self.key.clone(),
                std::any::type_name::<T>().to_string(),
                context.type_name(&self.key).unwrap_or_else(|| "unknown".to_string()),
            ));
        }
        
        Ok(())
    }
}
```

### Validation Executor

```rust
/// Executes context validation rules
pub struct ContextValidator {
    rules: Vec<Box<dyn ContextValidationRule>>,
}

impl ContextValidator {
    /// Creates a new context validator
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    /// Adds a validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ContextValidationRule>) {
        self.rules.push(rule);
    }
    
    /// Adds a required key rule
    pub fn require_key(&mut self, key: &str) {
        self.add_rule(Box::new(RequiredKeyRule::new(key.to_string())));
    }
    
    /// Adds a type validation rule
    pub fn require_type<T: 'static + Send + Sync>(&mut self, key: &str) {
        self.add_rule(Box::new(TypeValidationRule::<T>::new(key.to_string())));
    }
    
    /// Validates a context against all rules
    pub fn validate(&self, context: &dyn Context) -> Result<(), Vec<ContextValidationError>> {
        let mut errors = Vec::new();
        
        for rule in &self.rules {
            if let Err(error) = rule.validate(context) {
                errors.push(error);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Creates a validator for a context-aware command
    pub fn for_command(command: &dyn ContextAwareCommand) -> Self {
        let mut validator = Self::new();
        
        // Add required key rules
        for key in command.required_context_keys() {
            validator.require_key(&key);
        }
        
        // Add custom command validation rules
        if let Some(cmd) = command.as_any().downcast_ref::<dyn HasValidationRules>() {
            for rule in cmd.validation_rules() {
                validator.add_rule(rule);
            }
        }
        
        validator
    }
}
```

## Parameter Resolution

### Context-Based Resolution

```rust
/// Resolves command parameters from context
pub struct ContextParameterResolver {
    context: Arc<dyn Context>,
}

impl ContextParameterResolver {
    /// Creates a new context parameter resolver
    pub fn new(context: Arc<dyn Context>) -> Self {
        Self { context }
    }
    
    /// Resolves parameters against context
    pub fn resolve_parameters(&self, params: &mut HashMap<String, Value>) -> Result<(), ParameterResolutionError> {
        for (_, value) in params.iter_mut() {
            self.resolve_value(value)?;
        }
        
        Ok(())
    }
    
    /// Resolves a single value
    fn resolve_value(&self, value: &mut Value) -> Result<(), ParameterResolutionError> {
        match value {
            Value::String(s) => {
                if let Some(resolved) = self.resolve_string(s)? {
                    *s = resolved;
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.resolve_value(item)?;
                }
            }
            Value::Object(obj) => {
                for (_, v) in obj.iter_mut() {
                    self.resolve_value(v)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Resolves context references in a string
    fn resolve_string(&self, s: &str) -> Result<Option<String>, ParameterResolutionError> {
        // Match ${context:key} or ${context:key:default}
        lazy_static! {
            static ref CONTEXT_REF_REGEX: Regex = Regex::new(r"\$\{context:([^:}]+)(?::([^}]+))?\}").unwrap();
        }
        
        if !CONTEXT_REF_REGEX.is_match(s) {
            return Ok(None);
        }
        
        let mut result = s.to_string();
        
        for cap in CONTEXT_REF_REGEX.captures_iter(s) {
            let full_match = cap.get(0).unwrap().as_str();
            let key = cap.get(1).unwrap().as_str();
            let default = cap.get(2).map(|m| m.as_str());
            
            let replacement = if self.context.has_key(key) {
                // Handle different value types
                if self.context.is_type::<String>(key) {
                    self.context.get::<String>(key).unwrap_or_default()
                } else if self.context.is_type::<i64>(key) {
                    self.context.get::<i64>(key).unwrap_or_default().to_string()
                } else if self.context.is_type::<f64>(key) {
                    self.context.get::<f64>(key).unwrap_or_default().to_string()
                } else if self.context.is_type::<bool>(key) {
                    self.context.get::<bool>(key).unwrap_or_default().to_string()
                } else {
                    return Err(ParameterResolutionError::UnsupportedType(key.to_string()));
                }
            } else if let Some(default_value) = default {
                default_value.to_string()
            } else {
                return Err(ParameterResolutionError::MissingContextKey(key.to_string()));
            };
            
            result = result.replace(full_match, &replacement);
        }
        
        Ok(Some(result))
    }
}
```

## Command Registration

### Registration with Context Requirements

```rust
/// Extension to CommandRegistry for context-aware commands
impl CommandRegistry {
    /// Registers a context-aware command
    pub fn register_context_aware_command(&mut self, command: Box<dyn ContextAwareCommand>) -> Result<(), RegistrationError> {
        // Get required context keys
        let required_keys = command.required_context_keys();
        let optional_keys = command.optional_context_keys();
        let modified_keys = command.modified_context_keys();
        
        // Store context metadata
        let metadata = CommandContextMetadata {
            required_keys,
            optional_keys,
            modified_keys,
        };
        
        self.context_metadata.insert(command.name().to_string(), metadata);
        
        // Register the command as a regular command
        self.register_command(command)?;
        
        Ok(())
    }
    
    /// Gets context metadata for a command
    pub fn get_context_metadata(&self, command_name: &str) -> Option<&CommandContextMetadata> {
        self.context_metadata.get(command_name)
    }
}

/// Metadata about a command's context requirements
pub struct CommandContextMetadata {
    required_keys: HashSet<String>,
    optional_keys: HashSet<String>,
    modified_keys: HashSet<String>,
}

impl CommandContextMetadata {
    /// Gets required context keys
    pub fn required_keys(&self) -> &HashSet<String> {
        &self.required_keys
    }
    
    /// Gets optional context keys
    pub fn optional_keys(&self) -> &HashSet<String> {
        &self.optional_keys
    }
    
    /// Gets modified context keys
    pub fn modified_keys(&self) -> &HashSet<String> {
        &self.modified_keys
    }
    
    /// Checks if a command requires a key
    pub fn requires_key(&self, key: &str) -> bool {
        self.required_keys.contains(key)
    }
    
    /// Checks if a command modifies a key
    pub fn modifies_key(&self, key: &str) -> bool {
        self.modified_keys.contains(key)
    }
}
```

## Context Dependencies

### Dependency Analysis

```rust
/// Analyzes dependencies between commands based on context
pub struct CommandDependencyAnalyzer {
    registry: Arc<CommandRegistry>,
}

impl CommandDependencyAnalyzer {
    /// Creates a new command dependency analyzer
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }
    
    /// Analyzes dependencies between commands
    pub fn analyze_dependencies(&self) -> CommandDependencyGraph {
        let mut graph = CommandDependencyGraph::new();
        
        // Get all command names
        let command_names: Vec<String> = self.registry.list_commands()
            .iter()
            .map(|info| info.name.clone())
            .collect();
        
        // Add nodes for each command
        for name in &command_names {
            graph.add_node(name.clone());
        }
        
        // Analyze dependencies
        for producer in &command_names {
            if let Some(producer_metadata) = self.registry.get_context_metadata(producer) {
                for consumer in &command_names {
                    if producer == consumer {
                        continue;
                    }
                    
                    if let Some(consumer_metadata) = self.registry.get_context_metadata(consumer) {
                        // Check if producer modifies keys that consumer requires
                        for key in producer_metadata.modified_keys() {
                            if consumer_metadata.requires_key(key) {
                                graph.add_edge(producer.clone(), consumer.clone(), key.clone());
                            }
                        }
                    }
                }
            }
        }
        
        graph
    }
    
    /// Gets commands that might conflict with a given command
    pub fn find_conflicts(&self, command_name: &str) -> HashSet<String> {
        let mut conflicts = HashSet::new();
        
        if let Some(command_metadata) = self.registry.get_context_metadata(command_name) {
            let modified_keys = command_metadata.modified_keys();
            
            for other_name in self.registry.list_commands().iter().map(|info| &info.name) {
                if other_name == command_name {
                    continue;
                }
                
                if let Some(other_metadata) = self.registry.get_context_metadata(other_name) {
                    // Check for key modification conflicts
                    for key in other_metadata.modified_keys() {
                        if modified_keys.contains(key) {
                            conflicts.insert(other_name.clone());
                            break;
                        }
                    }
                }
            }
        }
        
        conflicts
    }
    
    /// Gets commands that must run before a given command
    pub fn get_prerequisites(&self, command_name: &str) -> HashSet<String> {
        let mut prerequisites = HashSet::new();
        
        if let Some(command_metadata) = self.registry.get_context_metadata(command_name) {
            let required_keys = command_metadata.required_keys();
            
            for other_name in self.registry.list_commands().iter().map(|info| &info.name) {
                if other_name == command_name {
                    continue;
                }
                
                if let Some(other_metadata) = self.registry.get_context_metadata(other_name) {
                    // Check if other command provides required keys
                    for key in other_metadata.modified_keys() {
                        if required_keys.contains(key) {
                            prerequisites.insert(other_name.clone());
                            break;
                        }
                    }
                }
            }
        }
        
        prerequisites
    }
}
```

### Dependency Graph

```rust
/// Dependency graph for commands based on context
pub struct CommandDependencyGraph {
    nodes: HashSet<String>,
    edges: HashMap<String, HashMap<String, HashSet<String>>>,
}

impl CommandDependencyGraph {
    /// Creates a new command dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
        }
    }
    
    /// Adds a node to the graph
    pub fn add_node(&mut self, command: String) {
        self.nodes.insert(command.clone());
        self.edges.entry(command).or_insert_with(HashMap::new);
    }
    
    /// Adds an edge to the graph
    pub fn add_edge(&mut self, from: String, to: String, key: String) {
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        
        let from_edges = self.edges.entry(from).or_insert_with(HashMap::new);
        let keys = from_edges.entry(to).or_insert_with(HashSet::new);
        keys.insert(key);
    }
    
    /// Gets all nodes in the graph
    pub fn nodes(&self) -> &HashSet<String> {
        &self.nodes
    }
    
    /// Gets all edges from a node
    pub fn edges_from(&self, command: &str) -> Option<&HashMap<String, HashSet<String>>> {
        self.edges.get(command)
    }
    
    /// Gets a topological sort of the graph
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        
        // Helper function for DFS
        fn visit(
            graph: &CommandDependencyGraph,
            node: &str,
            visited: &mut HashSet<String>,
            temp_visited: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) -> Result<(), String> {
            if temp_visited.contains(node) {
                return Err(format!("Cyclic dependency detected involving command '{}'", node));
            }
            
            if !visited.contains(node) {
                temp_visited.insert(node.to_string());
                
                if let Some(edges) = graph.edges_from(node) {
                    for dependent in edges.keys() {
                        visit(graph, dependent, visited, temp_visited, result)?;
                    }
                }
                
                temp_visited.remove(node);
                visited.insert(node.to_string());
                result.push(node.to_string());
            }
            
            Ok(())
        }
        
        // Visit each node
        for node in self.nodes.iter() {
            if !visited.contains(node) {
                visit(self, node, &mut visited, &mut temp_visited, &mut result)?;
            }
        }
        
        result.reverse();
        Ok(result)
    }
}
```

## Implementation Roadmap

### Phase 1: Basic Integration (1-2 Months)
- Context-aware command trait definition
- Basic context adapter implementation
- Simple context validation

### Phase 2: Enhanced Features (2-4 Months)
- Context change tracking
- Dependency analysis
- Parameter resolution

### Phase 3: Advanced Capabilities (4-6 Months)
- Complex validation rules
- Context lifecycle management
- Conflict detection and resolution

### Phase 4: Enterprise Features (6-12 Months)
- Command scheduling based on context dependencies
- Context versioning and history
- Context access control and security

## Integration Testing

### Test Scenarios

1. **Basic Command Execution**
   - Command execution with required context
   - Command execution with missing context
   - Command modification of context

2. **Error Handling**
   - Command rollback on failure
   - Context validation failures
   - Context update conflicts

3. **Performance Testing**
   - Context access performance
   - Context update performance
   - Validation performance

### Test Metrics

1. **Reliability Metrics**
   - Context consistency rate
   - Rollback success rate
   - Validation accuracy

2. **Performance Metrics**
   - Context access latency
   - Command overhead from context
   - Context update throughput

## Conclusion

The integration between the Context Management System and Command System provides a powerful framework for context-aware command execution. By defining clear interfaces and responsibilities, commands can access and modify contextual data while maintaining context integrity and enabling sophisticated features like dependency resolution, validation, and rollback.

<version>1.0.0</version> 