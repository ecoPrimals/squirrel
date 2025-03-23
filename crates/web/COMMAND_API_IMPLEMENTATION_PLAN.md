# Command Execution API Implementation Plan

## Overview

This document outlines the detailed implementation plan for the Command Execution API in the Squirrel Web Interface. The Command Execution API is a high-priority component that allows external systems to execute commands via the Squirrel platform, monitor their status, and retrieve results.

## Requirements

### API Endpoints

1. **Create Command**
   - `POST /api/commands`
   - Creates and queues a new command for execution
   - Authenticates the user and checks permissions
   - Validates command parameters
   - Returns a command ID and status URL

2. **List Available Commands**
   - `GET /api/commands/available`
   - Lists all available commands with their descriptions
   - Filters commands based on user permissions
   - Supports pagination
   - Returns command metadata (name, description, parameters)

3. **Get Command Status**
   - `GET /api/commands/:id`
   - Returns the current status of a command
   - Includes progress information
   - Provides error details if applicable
   - Returns result information if complete

4. **List Command History**
   - `GET /api/commands`
   - Lists all commands executed by the user
   - Supports filtering by status, date, and type
   - Supports pagination
   - Returns summary information for each command

5. **Cancel Command**
   - `DELETE /api/commands/:id`
   - Attempts to cancel a running command
   - Returns success or failure
   - Provides reason if cancellation fails

### WebSocket Integration

1. **Command Status Updates**
   - Real-time updates to command status
   - WebSocket event format: `CommandStatus`
   - Channel format: `command:{command_id}`
   - Includes progress, status, and result information

2. **Command List Updates**
   - Real-time updates to command list
   - WebSocket event format: `CommandListUpdate`
   - Channel format: `command:list`
   - Adds/removes commands from the list in real-time

### MCP Integration

1. **Command Execution**
   - Convert API request to MCP message format
   - Route command to appropriate MCP handler
   - Track command execution context
   - Handle timeout and errors

2. **Command Result Processing**
   - Parse MCP command responses
   - Store command results
   - Update command status
   - Notify WebSocket clients

## Database Schema

```sql
-- Command definition table
CREATE TABLE commands (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    schema TEXT NOT NULL,  -- JSON schema for command parameters
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Command execution history
CREATE TABLE command_executions (
    id TEXT PRIMARY KEY,
    command_name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    parameters TEXT NOT NULL,  -- JSON parameters
    status TEXT NOT NULL,      -- queued, running, completed, failed, cancelled
    progress REAL DEFAULT 0.0,
    result TEXT,               -- JSON result
    error TEXT,                -- Error information if failed
    started_at DATETIME,
    completed_at DATETIME,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

## Data Models

### Command Definition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Command ID
    pub id: String,
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// JSON schema for parameters
    pub parameter_schema: serde_json::Value,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}
```

### Command Execution

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    /// Execution ID
    pub id: String,
    /// Command name
    pub command_name: String,
    /// User ID
    pub user_id: String,
    /// Command parameters
    pub parameters: serde_json::Value,
    /// Execution status
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}
```

### Command Status

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### Request/Response Types

```rust
// Create command request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandRequest {
    /// Command name
    pub command: String,
    /// Command parameters
    pub parameters: serde_json::Value,
}

// Create command response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommandResponse {
    /// Command execution ID
    pub id: String,
    /// Status URL
    pub status_url: String,
}

// Command status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStatusResponse {
    /// Command execution ID
    pub id: String,
    /// Command name
    pub command: String,
    /// Execution status
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
    /// Start time
    pub started_at: Option<String>,
    /// Completion time
    pub completed_at: Option<String>,
    /// Time since creation
    pub elapsed: String,
}

// Command list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandListResponse {
    /// Available commands
    pub commands: Vec<CommandDefinition>,
}

// Command history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryResponse {
    /// Command executions
    pub executions: Vec<CommandExecution>,
}
```

## WebSocket Events

```rust
// Command status update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStatusEvent {
    /// Event type
    pub event: String,  // "command-status"
    /// Command execution ID
    pub id: String,
    /// Command name
    pub command: String,
    /// Execution status
    pub status: CommandStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Result (if completed)
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<String>,
}

// Command list update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandListUpdateEvent {
    /// Event type
    pub event: String,  // "command-list-update"
    /// Action (add, remove, update)
    pub action: String,
    /// Command definition
    pub command: CommandDefinition,
}
```

## Service Layer

```rust
/// Command service interface
#[async_trait]
pub trait CommandService: Send + Sync + 'static {
    /// Create and execute a new command
    async fn create_command(
        &self,
        user_id: &str,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, AppError>;
    
    /// Get available commands
    async fn get_available_commands(
        &self,
        user_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<CommandDefinition>, u64, u32), AppError>;
    
    /// Get command status
    async fn get_command_status(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<CommandExecution, AppError>;
    
    /// Get command history
    async fn get_command_history(
        &self,
        user_id: &str,
        page: u32,
        limit: u32,
        status: Option<CommandStatus>,
        command: Option<&str>,
    ) -> Result<(Vec<CommandExecution>, u64, u32), AppError>;
    
    /// Cancel command execution
    async fn cancel_command(
        &self,
        user_id: &str,
        command_id: &str,
    ) -> Result<(), AppError>;
}
```

## MCP Integration

```rust
/// MCP command client interface
#[async_trait]
pub trait McpCommandClient: Send + Sync + 'static {
    /// Execute a command via MCP
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError>;
    
    /// Get command status
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError>;
    
    /// Cancel command
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError>;
    
    /// List available commands
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError>;
}
```

## Implementation Phases

### Phase 1: Data Models and Database Schema (Days 1-2)

1. **Define all data models**
   - Command definition
   - Command execution
   - Request/response types
   - WebSocket event models

2. **Create database schema**
   - Create migration for commands table
   - Create migration for command_executions table
   - Update schema.rs with new models

3. **Implement mock implementations for testing**
   - Create in-memory implementations of repositories
   - Add test data generation helpers

### Phase 2: Service Layer (Days 3-5)

1. **Implement command service**
   - Create CommandService interface
   - Implement database-backed implementation
   - Implement mock implementation for testing
   - Add unit tests for service methods

2. **Implement MCP integration**
   - Create McpCommandClient interface
   - Implement real client for MCP
   - Implement mock client for testing
   - Add unit tests for MCP client

### Phase 3: API Endpoints (Days 6-8)

1. **Implement API endpoints**
   - Create commands module in handlers
   - Implement create_command handler
   - Implement get_command_status handler
   - Implement list_available_commands handler
   - Implement get_command_history handler
   - Implement cancel_command handler

2. **Add authentication and validation**
   - Add authentication requirements
   - Implement request validation
   - Add role-based access checks
   - Add tests for authentication and validation

### Phase 4: WebSocket Integration (Days 9-10)

1. **Extend WebSocket with command events**
   - Add CommandStatus event type
   - Add CommandListUpdate event type
   - Implement command channel subscription
   - Add integration tests for WebSocket commands

2. **Implement real-time updates**
   - Add command status update notification
   - Create command list update notification
   - Implement error notification
   - Add tests for real-time updates

### Phase 5: Testing and Documentation (Days 11-14)

1. **Add comprehensive tests**
   - Unit tests for all components
   - Integration tests for API endpoints
   - WebSocket communication tests
   - Performance tests

2. **Create API documentation**
   - Add OpenAPI/Swagger specification
   - Document all endpoints
   - Add request/response examples
   - Document WebSocket API

## Risks and Mitigations

1. **MCP Integration Complexity**
   - **Risk**: The MCP protocol may have complex requirements
   - **Mitigation**: Start with a simplified integration and refine incrementally
   - **Fallback**: Implement a mock MCP client if real integration is delayed

2. **Performance Under Load**
   - **Risk**: Command execution may be resource intensive
   - **Mitigation**: Implement proper queuing and resource management
   - **Fallback**: Add rate limiting and graceful degradation

3. **Database Schema Evolution**
   - **Risk**: Schema may need to evolve as requirements change
   - **Mitigation**: Design flexible schema with future changes in mind
   - **Fallback**: Use feature flags to support multiple schema versions

4. **WebSocket Scalability**
   - **Risk**: Large number of WebSocket connections may impact performance
   - **Mitigation**: Implement connection pooling and efficient broadcasting
   - **Fallback**: Add connection limits and graceful degradation

## Acceptance Criteria

1. **Functional Requirements**
   - All API endpoints implemented and functional
   - WebSocket real-time updates working
   - MCP integration complete
   - Proper error handling in place

2. **Non-Functional Requirements**
   - API response time < 100ms (p95)
   - WebSocket message processing < 50ms (p95)
   - Test coverage > 80%
   - All endpoints documented in OpenAPI/Swagger

3. **Security Requirements**
   - All endpoints properly authenticate users
   - Role-based access control implemented
   - Input validation for all parameters
   - Secure error responses (no sensitive information) 