---
title: Web API Implementation Pattern
version: 1.0.0
date: 2024-03-22
status: approved
category: implementation
---

# Web API Implementation Pattern

## Context

This pattern applies when:
- Implementing new endpoints in the Web Interface
- Extending existing API functionality
- Implementing WebSocket connections
- Integrating the Web Interface with other Squirrel components
- Implementing security features for the API

## Implementation Guide

### Endpoint Structure

All HTTP endpoints in the Web Interface should follow a consistent structure:

1. **Routing**:
   - Group related endpoints in the same router module
   - Use consistent route naming patterns
   - Follow RESTful conventions for resource operations

```rust
pub fn user_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_user))
        .route("/", get(list_users))
        .route("/:id", get(get_user))
        .route("/:id", patch(update_user))
        .route("/:id", delete(delete_user))
}
```

2. **Handler Signature**:
   - Use consistent parameter ordering: `State`, authentication, path params, query params, request body
   - Return `Result<Json<ApiResponse<T>>, AppError>` for all handlers
   - Use appropriate types for request/response bodies

```rust
pub async fn create_user(
    State(state): State<Arc<AppState>>,     // State always first
    claims: Claims,                        // Authentication second
    Json(req): Json<CreateUserRequest>,    // Request body last
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    // Implementation
}
```

3. **Response Format**:
   - Use standardized `ApiResponse` wrapper for all responses
   - Include appropriate metadata
   - Use pagination helpers for list endpoints

```rust
// Success response with single item
Ok(api_success(user_response))

// Success response with pagination
Ok(api_success_paginated(
    user_responses,
    page,
    limit,
    total_items,
    total_pages
))
```

### Authentication & Authorization

All protected endpoints should implement authentication and authorization:

1. **Authentication**:
   - Use `Claims` extractor for JWT authentication
   - Use appropriate authentication middleware based on endpoint requirements
   - Handle authentication failures with proper error responses

2. **Role-Based Access**:
   - Check user roles for authorization
   - Use role-specific middleware when appropriate
   - Return 403 Forbidden for unauthorized access attempts

```rust
// Admin-only endpoint
pub async fn admin_operation(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<ApiResponse<AdminResponse>>, AppError> {
    // Verify admin role
    if !claims.roles.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    
    // Implementation for admin operation
}
```

3. **API Key Authentication**:
   - Use API key authentication for service-to-service communication
   - Validate API keys against stored keys
   - Implement rate limiting for API key usage

### Error Handling

Follow consistent error handling patterns:

1. **AppError Definition**:
   - Define specific error variants for different error conditions
   - Map internal errors to appropriate HTTP status codes
   - Provide meaningful error messages for clients

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Forbidden(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Invalid request: {0}")]
    BadRequest(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}
```

2. **Error Response Mapping**:
   - Implement `IntoResponse` for converting errors to API responses
   - Include appropriate HTTP status codes
   - Provide error codes and messages in the response

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Authentication(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let error_code = match self {
            Self::Authentication(_) => "AUTHENTICATION_ERROR",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        };
        
        let body = Json(ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code: error_code.to_string(),
                message: self.to_string(),
                details: None,
            }),
            meta: ApiMeta {
                request_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().to_rfc3339(),
                pagination: None,
            },
        });
        
        (status, body).into_response()
    }
}
```

3. **Error Logging**:
   - Log errors with appropriate context
   - Include request ID for correlation
   - Use appropriate log levels based on error severity

```rust
// Log error with context
tracing::error!(
    request_id = %request_id,
    user_id = %user_id,
    endpoint = "create_user",
    error = %err,
    "Failed to create user"
);
```

### WebSocket Implementation

WebSocket endpoints should follow these patterns:

1. **Connection Establishment**:
   - Validate authentication during WebSocket upgrade
   - Establish connection with proper error handling
   - Track active connections with an appropriate data structure

```rust
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_connection(socket, state, claims))
}
```

2. **Message Handling**:
   - Define a consistent message format
   - Handle different message types with pattern matching
   - Implement proper error responses

```rust
async fn handle_message(
    msg: Message,
    state: Arc<AppState>,
    claims: Claims,
) -> Result<Option<Message>, websocket::Error> {
    match msg {
        Message::Text(text) => {
            let payload: WebSocketCommand = serde_json::from_str(&text)?;
            match payload.command.as_str() {
                "subscribe" => handle_subscribe(&payload, &state, &claims).await,
                "unsubscribe" => handle_unsubscribe(&payload, &state, &claims).await,
                _ => Err(websocket::Error::UnknownCommand),
            }
        },
        Message::Close(_) => Ok(None),
        _ => Err(websocket::Error::UnsupportedMessageType),
    }
}
```

3. **Connection Management**:
   - Implement proper connection lifecycle management
   - Handle disconnections gracefully
   - Clean up resources when connections close

```rust
async fn websocket_connection(
    socket: WebSocket,
    state: Arc<AppState>,
    claims: Claims,
) {
    // Register connection
    let connection_id = Uuid::new_v4();
    state.connection_manager.register(connection_id, claims.sub.clone()).await;
    
    // Split socket for concurrent reading and writing
    let (tx, mut rx) = socket.split();
    
    // Handle messages and cleanup on disconnect
    let result = handle_socket_messages(tx, &mut rx, state.clone(), claims, connection_id).await;
    
    // Clean up on disconnect
    state.connection_manager.unregister(connection_id).await;
    
    if let Err(e) = result {
        tracing::error!(connection_id = %connection_id, error = %e, "WebSocket error");
    }
}
```

### Integration with MCP

Integrate with the Machine Context Protocol using these patterns:

1. **Message Transformation**:
   - Convert between API requests and MCP messages
   - Maintain context information across protocol boundaries
   - Handle serialization differences

```rust
async fn execute_command(
    state: &AppState,
    command: &Command,
    user_id: &str,
) -> Result<CommandResult, AppError> {
    // Convert API command to MCP message
    let mcp_message = McpMessage {
        type_: "command.execute".to_string(),
        payload: serde_json::to_value(command)?,
        context: McpContext {
            user_id: user_id.to_string(),
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        },
    };
    
    // Send to MCP and receive response
    let response = state.mcp_client
        .send_message(&serde_json::to_string(&mcp_message)?)
        .await?;
    
    // Convert MCP response to API response
    let command_result: CommandResult = serde_json::from_str(&response)?;
    
    Ok(command_result)
}
```

2. **Error Propagation**:
   - Translate MCP errors to API errors
   - Preserve context information for debugging
   - Maintain proper security boundaries

3. **Event Handling**:
   - Subscribe to relevant MCP events
   - Forward events to WebSocket clients
   - Implement proper filtering based on user context

### Database Access

Use these patterns for database access:

1. **Feature Flag Conditional Compilation**:
   - Support both `db` and `mock-db` modes
   - Use conditional compilation for different implementations
   - Share common interfaces

```rust
#[cfg(feature = "db")]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<ApiResponse<Vec<UserResponse>>>, AppError> {
    // Database implementation
    let users = sqlx::query_as!(/* ... */)
        .fetch_all(&state.db)
        .await?;
    
    // Transform and return
}

#[cfg(feature = "mock-db")]
pub async fn list_users(
    _state: State<Arc<AppState>>,
    _claims: Claims,
) -> Result<Json<ApiResponse<Vec<UserResponse>>>, AppError> {
    // Mock implementation
    let users = vec![/* ... */];
    
    // Transform and return
}
```

2. **Repository Pattern**:
   - Separate database access from business logic
   - Use repository traits for abstraction
   - Implement mock repositories for testing

3. **Migrations and Schema Management**:
   - Use SQLx migrations for schema management
   - Keep migrations idempotent
   - Include both up and down migration scripts

## Benefits & Tradeoffs

### Benefits

- **Consistency**: Standardized approach to API implementation
- **Maintainability**: Clear patterns for handling common tasks
- **Testability**: Abstractions allow for easier testing
- **Security**: Consistent authentication and authorization
- **Error Handling**: Standardized error responses
- **Flexibility**: Dual-mode architecture supports different development scenarios

### Tradeoffs

- **Boilerplate**: Some patterns introduce additional code
- **Learning Curve**: Developers need to understand the patterns
- **Complexity**: Dual-mode support adds conditional compilation complexity
- **Feature Flags**: Requires careful management of feature flags

## Examples

### Complete Endpoint Implementation

```rust
// User creation endpoint
#[cfg(feature = "db")]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    // Verify admin role for user creation
    if !claims.roles.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    
    // Validate input
    if req.username.len() < 3 {
        return Err(AppError::BadRequest("Username must be at least 3 characters".to_string()));
    }
    
    // Check if username exists
    let existing = sqlx::query!(
        "SELECT id FROM users WHERE username = ?",
        req.username
    )
    .fetch_optional(&state.db)
    .await?;
    
    if existing.is_some() {
        return Err(AppError::BadRequest("Username already exists".to_string()));
    }
    
    // Hash password
    let password_hash = bcrypt::hash(req.password, bcrypt::DEFAULT_COST)?;
    
    // Insert user
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    sqlx::query!(
        r#"
        INSERT INTO users (
            id, username, password_hash, email, roles, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user_id.to_string(),
        req.username,
        password_hash,
        req.email,
        serde_json::to_string(&req.roles)?,
        now,
        now
    )
    .execute(&state.db)
    .await?;
    
    // Return response
    let response = UserResponse {
        id: user_id,
        username: req.username,
        email: req.email,
        roles: req.roles,
        created_at: now,
        updated_at: now,
    };
    
    Ok(api_success(response))
}
```

### WebSocket Implementation Example

```rust
// WebSocket subscription handler
async fn handle_subscribe(
    payload: &WebSocketCommand,
    state: &Arc<AppState>,
    claims: &Claims,
) -> Result<Option<Message>, websocket::Error> {
    let channel = payload.params.get("channel")
        .ok_or(websocket::Error::MissingParameter("channel"))?
        .as_str()
        .ok_or(websocket::Error::InvalidParameterType("channel"))?;
    
    // Check authorization for channel
    if !is_authorized_for_channel(channel, claims) {
        return Err(websocket::Error::Unauthorized(
            format!("Not authorized for channel: {}", channel)
        ));
    }
    
    // Subscribe to channel
    state.subscription_manager
        .subscribe(claims.sub.clone(), channel.to_string())
        .await?;
    
    // Return success message
    Ok(Some(Message::Text(serde_json::to_string(&WebSocketResponse {
        success: true,
        event: "subscribed".to_string(),
        data: json!({
            "channel": channel
        }),
        error: None,
    })?)))
}
```

## Testing Approach

### Unit Testing

Test each endpoint with:
- Valid inputs
- Invalid inputs
- Authentication failures
- Authorization failures
- Database errors

Use mock implementations:
- Create mock `AppState` with mock services
- Test with mock database in `mock-db` mode
- Use test fixtures for common scenarios

```rust
#[tokio::test]
async fn test_create_user_success() {
    // Arrange
    let app_state = test_utils::create_test_state().await;
    let claims = test_utils::create_admin_claims();
    let request = CreateUserRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
        email: "test@example.com".to_string(),
        roles: vec!["user".to_string()],
    };
    
    // Act
    let result = create_user(
        State(Arc::new(app_state)),
        claims,
        Json(request)
    ).await;
    
    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    let user = response.0.data.unwrap();
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.roles, vec!["user"]);
}
```

### Integration Testing

Test complete workflows:
- API endpoint chains
- WebSocket connections
- Authentication flows
- Database integration

```rust
#[tokio::test]
async fn test_user_workflow() {
    // Setup test server
    let app = test_utils::create_test_app().await;
    let client = axum_test::TestClient::new(app);
    
    // Login as admin
    let login_response = client.post("/api/auth/login")
        .json(&LoginRequest {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        })
        .send()
        .await;
    
    assert_eq!(login_response.status(), StatusCode::OK);
    let token = login_response.json::<ApiResponse<LoginResponse>>().await
        .data.unwrap().token;
    
    // Create user
    let create_response = client.post("/api/users")
        .header("Authorization", format!("Bearer {}", token))
        .json(&CreateUserRequest {
            username: "newuser".to_string(),
            password: "password123".to_string(),
            email: "new@example.com".to_string(),
            roles: vec!["user".to_string()],
        })
        .send()
        .await;
    
    assert_eq!(create_response.status(), StatusCode::OK);
    
    // Get created user
    let user_id = create_response.json::<ApiResponse<UserResponse>>().await
        .data.unwrap().id;
    
    let get_response = client.get(&format!("/api/users/{}", user_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;
    
    assert_eq!(get_response.status(), StatusCode::OK);
    let user = get_response.json::<ApiResponse<UserResponse>>().await.data.unwrap();
    assert_eq!(user.username, "newuser");
}
```

## Security & Performance

### Security Considerations

- Use JWT tokens with appropriate expiration
- Implement refresh token rotation for security
- Validate all input parameters
- Use parameterized queries to prevent SQL injection
- Implement rate limiting for public endpoints
- Use secure password hashing with bcrypt
- Add security headers to all responses
- Implement proper CORS configuration
- Audit log security-critical operations

### Performance Considerations

- Use connection pooling for database access
- Implement caching for frequently accessed data
- Use asynchronous processing for long-running operations
- Implement pagination for list endpoints
- Optimize database queries with appropriate indexes
- Use efficient serialization/deserialization
- Implement timeout handling for external services
- Monitor performance with metrics collection

## Migration Guide

### Converting Existing Endpoints

1. Update endpoint signature to match pattern:

```rust
// Before
async fn get_data(state: &AppState, user_id: &str) -> Result<Data, Error> {
    // ...
}

// After
async fn get_data(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Data>>, AppError> {
    // ...
}
```

2. Update response format to use ApiResponse:

```rust
// Before
Ok(Json(data))

// After
Ok(api_success(data))
```

3. Update error handling to use AppError:

```rust
// Before
Err(Error::NotFound)

// After
Err(AppError::NotFound("Resource not found".to_string()))
```

4. Add feature flags for different implementations:

```rust
// Add feature flags
#[cfg(feature = "db")]
async fn handler() {
    // Database implementation
}

#[cfg(feature = "mock-db")]
async fn handler() {
    // Mock implementation
}
```

<version>1.0.0</version> 