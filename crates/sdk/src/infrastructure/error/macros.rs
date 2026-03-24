// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Helper macros for error creation in the Squirrel Plugin SDK

/// Helper macro for creating parameter validation errors
#[macro_export]
macro_rules! param_error {
    ($name:expr, $reason:expr) => {
        $crate::infrastructure::error::core::PluginError::InvalidParameter {
            name: $name.to_string(),
            reason: $reason.to_string(),
        }
    };
}

/// Helper macro for creating missing parameter errors
#[macro_export]
macro_rules! missing_param {
    ($name:expr) => {
        $crate::infrastructure::error::core::PluginError::MissingParameter {
            parameter: $name.to_string(),
        }
    };
}

/// Helper macro for creating enhanced errors with context
#[macro_export]
macro_rules! error_with_context {
    ($error:expr, $operation:expr) => {
        $error.with_context($crate::infrastructure::error::context::ErrorContext::new(
            $operation,
        ))
    };
    ($error:expr, $operation:expr, $module:expr) => {
        $error.with_context(
            $crate::infrastructure::error::context::ErrorContext::new($operation)
                .with_module($module),
        )
    };
    ($error:expr, $operation:expr, $module:expr, $function:expr) => {
        $error.with_context(
            $crate::infrastructure::error::context::ErrorContext::new($operation)
                .with_module($module)
                .with_function($function),
        )
    };
}

/// Helper macro for creating network errors
#[macro_export]
macro_rules! network_error {
    ($operation:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::NetworkError {
            operation: $operation.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating file system errors
#[macro_export]
macro_rules! fs_error {
    ($operation:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::FileSystemError {
            operation: $operation.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating configuration errors
#[macro_export]
macro_rules! config_error {
    ($message:expr) => {
        $crate::infrastructure::error::core::PluginError::ConfigurationError {
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating timeout errors
#[macro_export]
macro_rules! timeout_error {
    ($operation:expr, $seconds:expr) => {
        $crate::infrastructure::error::core::PluginError::TimeoutError {
            operation: $operation.to_string(),
            seconds: $seconds,
        }
    };
}

/// Helper macro for creating security violation errors
#[macro_export]
macro_rules! security_error {
    ($violation:expr) => {
        $crate::infrastructure::error::core::PluginError::SecurityViolation {
            violation: $violation.to_string(),
        }
    };
}

/// Helper macro for creating permission denied errors
#[macro_export]
macro_rules! permission_denied {
    ($operation:expr, $reason:expr) => {
        $crate::infrastructure::error::core::PluginError::PermissionDenied {
            operation: $operation.to_string(),
            reason: $reason.to_string(),
        }
    };
}

/// Helper macro for creating resource limit exceeded errors
#[macro_export]
macro_rules! resource_limit_exceeded {
    ($resource:expr, $limit:expr) => {
        $crate::infrastructure::error::core::PluginError::ResourceLimitExceeded {
            resource: $resource.to_string(),
            limit: $limit.to_string(),
        }
    };
}

/// Helper macro for creating plugin not found errors
#[macro_export]
macro_rules! plugin_not_found {
    ($plugin_id:expr) => {
        $crate::infrastructure::error::core::PluginError::PluginNotFound {
            plugin_id: $plugin_id.to_string(),
        }
    };
}

/// Helper macro for creating internal errors
#[macro_export]
macro_rules! internal_error {
    ($message:expr) => {
        $crate::infrastructure::error::core::PluginError::InternalError {
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating command execution errors
#[macro_export]
macro_rules! command_error {
    ($command:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::CommandExecutionError {
            command: $command.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating not implemented errors
#[macro_export]
macro_rules! not_implemented {
    ($feature:expr) => {
        $crate::infrastructure::error::core::PluginError::NotImplemented {
            feature: $feature.to_string(),
        }
    };
}

/// Helper macro for creating not supported errors
#[macro_export]
macro_rules! not_supported {
    ($feature:expr) => {
        $crate::infrastructure::error::core::PluginError::NotSupported {
            feature: $feature.to_string(),
        }
    };
}

/// Helper macro for creating external service errors
#[macro_export]
macro_rules! external_service_error {
    ($service:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::ExternalServiceError {
            service: $service.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating MCP protocol errors
#[macro_export]
macro_rules! mcp_error {
    ($message:expr) => {
        $crate::infrastructure::error::core::PluginError::McpError {
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating serialization errors
#[macro_export]
macro_rules! serialization_error {
    ($message:expr) => {
        $crate::infrastructure::error::core::PluginError::SerializationError {
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating validation errors
#[macro_export]
macro_rules! validation_error {
    ($field:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::ValidationError {
            field: $field.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating authentication errors
#[macro_export]
macro_rules! auth_error {
    ($message:expr) => {
        $crate::infrastructure::error::core::PluginError::AuthenticationError {
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating authorization errors
#[macro_export]
macro_rules! authz_error {
    ($resource:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::AuthorizationError {
            resource: $resource.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating rate limit errors
#[macro_export]
macro_rules! rate_limit_error {
    ($resource:expr, $retry_after:expr) => {
        $crate::infrastructure::error::core::PluginError::RateLimitError {
            resource: $resource.to_string(),
            retry_after: $retry_after,
        }
    };
}

/// Helper macro for creating context errors
#[macro_export]
macro_rules! context_error {
    ($context:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::ContextError {
            context: $context.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating temporary failure errors
#[macro_export]
macro_rules! temporary_failure {
    ($operation:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::TemporaryFailure {
            operation: $operation.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating permanent failure errors
#[macro_export]
macro_rules! permanent_failure {
    ($operation:expr, $message:expr) => {
        $crate::infrastructure::error::core::PluginError::PermanentFailure {
            operation: $operation.to_string(),
            message: $message.to_string(),
        }
    };
}

/// Helper macro for creating deprecated feature errors
#[macro_export]
macro_rules! deprecated_error {
    ($feature:expr, $alternative:expr) => {
        $crate::infrastructure::error::core::PluginError::Deprecated {
            feature: $feature.to_string(),
            alternative: $alternative.to_string(),
        }
    };
}
