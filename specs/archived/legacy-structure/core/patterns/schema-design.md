# Schema Design Pattern

## Context

In the Squirrel codebase, consistent schema design is essential for data validation, serialization, and API contracts. This pattern outlines the approach to designing and implementing schemas across different components of the system.

### When to Use This Pattern

- When defining data structures that will be serialized/deserialized
- When creating API contracts between components or with external systems
- When validating user input or configuration
- When migrating data between different schema versions

## Implementation

### 1. Schema Definition

Use Rust's strong type system combined with serialization libraries for schema definition:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default)]
    pub preferences: UserPreferences,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub notifications_enabled: bool,
}
```

### 2. Validation

Implement validation using either custom validation methods or validation libraries:

```rust
impl UserProfile {
    // Custom validation method
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::new("id cannot be empty"));
        }
        
        if self.display_name.is_empty() {
            return Err(ValidationError::new("display_name cannot be empty"));
        }
        
        if let Some(email) = &self.email {
            if !is_valid_email(email) {
                return Err(ValidationError::new("invalid email format"));
            }
        }
        
        Ok(())
    }
}

// Using validator crate
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct ConfigSchema {
    #[validate(length(min = 1))]
    pub name: String,
    
    #[validate(range(min = 1, max = 100))]
    pub max_connections: u32,
    
    #[validate(email)]
    pub admin_email: String,
}
```

### 3. Schema Versioning

For schemas that may evolve over time, include versioning:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum UserProfileSchema {
    #[serde(rename = "v1")]
    V1(UserProfileV1),
    
    #[serde(rename = "v2")]
    V2(UserProfileV2),
}

impl UserProfileSchema {
    // Convert to latest version
    pub fn to_latest(self) -> UserProfileV2 {
        match self {
            UserProfileSchema::V1(v1) => v1.into(),
            UserProfileSchema::V2(v2) => v2,
        }
    }
}

impl From<UserProfileV1> for UserProfileV2 {
    fn from(v1: UserProfileV1) -> Self {
        UserProfileV2 {
            id: v1.id,
            display_name: v1.display_name,
            email: v1.email,
            preferences: v1.preferences,
            created_at: v1.created_at,
            // New fields in V2
            updated_at: time::OffsetDateTime::now_utc(),
            account_type: AccountType::Standard,
        }
    }
}
```

### 4. API Contracts

When using schemas for API contracts, document them clearly:

```rust
/// API request for user creation
/// 
/// # Example
/// 
/// ```json
/// {
///   "displayName": "John Doe",
///   "email": "john@example.com",
///   "preferences": {
///     "theme": "dark",
///     "notificationsEnabled": true
///   }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[serde(default)]
    pub preferences: UserPreferences,
}

/// API response for user creation
/// 
/// # Example
/// 
/// ```json
/// {
///   "id": "user_123",
///   "displayName": "John Doe",
///   "createdAt": "2023-01-01T00:00:00Z"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserResponse {
    pub id: String,
    pub display_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
}
```

## Benefits

1. **Type Safety**: Leverages Rust's type system to ensure schema correctness
2. **Validation**: Enforces data integrity through validation
3. **Documentation**: Self-documenting through Rust types and documentation
4. **Serialization**: Consistent handling of serialization/deserialization
5. **Evolution**: Supports schema evolution over time
6. **API Contracts**: Clearly defines API contracts between components

## Tradeoffs

1. **Boilerplate**: Requires more code than dynamic schemas
2. **Flexibility**: Less flexible than dynamic schemas for certain use cases
3. **Compilation**: Changes require recompilation
4. **Learning Curve**: Requires understanding of Rust's type system and serde
5. **Size**: Generated code may be larger than dynamic validation

## When to Use

- For all data structures that cross component boundaries
- When defining public APIs
- When data persistence is involved
- When validation is critical for correctness
- When evolving data formats over time

## When to Avoid

- For internal, ephemeral data structures that don't cross boundaries
- When extreme flexibility is required (consider dynamic validation)
- When schemas need to be defined at runtime

## Related Patterns

- **Error Handling Pattern**: For handling validation errors
- **API Response Pattern**: For structuring API responses
- **Resource Management Pattern**: For managing resources with validated schemas
- **Serialization Pattern**: For consistent serialization/deserialization

## Examples in Codebase

While we don't currently have a dedicated schema crate, we use schema patterns in several places:

- `commands` crate for command payloads
- `context` crate for context data structures
- `mcp` crate for protocol messages

## Testing Approach

Schemas should be thoroughly tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_user_profile() {
        let profile = UserProfile {
            id: "user_123".to_string(),
            display_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            preferences: UserPreferences::default(),
            created_at: time::OffsetDateTime::now_utc(),
        };
        
        assert!(profile.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_user_profile() {
        let profile = UserProfile {
            id: "".to_string(), // Invalid: empty ID
            display_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            preferences: UserPreferences::default(),
            created_at: time::OffsetDateTime::now_utc(),
        };
        
        assert!(profile.validate().is_err());
    }
    
    #[test]
    fn test_serialization_deserialization() {
        let profile = UserProfile {
            id: "user_123".to_string(),
            display_name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            preferences: UserPreferences::default(),
            created_at: time::OffsetDateTime::now_utc(),
        };
        
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: UserProfile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(profile.id, deserialized.id);
        assert_eq!(profile.display_name, deserialized.display_name);
    }
}
```

## Security Considerations

1. **Input Validation**: Always validate external input against schemas
2. **Sensitive Data**: Be careful with serializing sensitive data
3. **Denial of Service**: Consider limits on collection sizes
4. **Schema Poisoning**: Guard against maliciously crafted inputs
5. **Default Values**: Use secure defaults when deserializing

## Performance Characteristics

1. **Compile-time Checks**: Most validation occurs at compile time
2. **Runtime Validation**: Some validation occurs at runtime
3. **Serialization Cost**: Consider the cost of serialization/deserialization
4. **Size**: Consider the size impact of generated serialization code

## Future Work: Schema Crate

In the future, we should consider implementing a dedicated `schema` crate that provides:

1. **Standard Types**: Common schema definitions
2. **Validation Framework**: Consistent validation approach
3. **Migration Utilities**: Tools for schema evolution
4. **Documentation Generation**: Generate OpenAPI/JSON Schema docs

This would centralize schema management and ensure consistency across components.

## Migration Guide

When implementing a new schema or updating an existing one:

1. Define the schema using Rust types with serde attributes
2. Implement validation logic (custom or using a validation library)
3. Add version information if the schema may evolve
4. Document the schema with examples
5. Write tests to verify validation and serialization
6. If updating, provide a migration path from the old schema

## Version History

<version>1.0.0</version> 