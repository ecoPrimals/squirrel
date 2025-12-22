//! Tests for authentication error types
//!
//! Comprehensive test coverage for AuthError and AuthResult types.

use super::*;

#[cfg(test)]
mod auth_error_tests {
    use super::*;

    #[test]
    fn test_authentication_failed_error() {
        let error = AuthError::authentication_failed("Invalid credentials");

        assert!(matches!(error, AuthError::AuthenticationFailed { .. }));
        assert_eq!(
            error.to_string(),
            "Authentication failed: Invalid credentials"
        );
    }

    #[test]
    fn test_token_error() {
        let error = AuthError::token_error("validate", "Token expired");

        assert!(matches!(error, AuthError::Token { .. }));
        assert_eq!(error.to_string(), "Token error in validate: Token expired");
    }

    #[test]
    fn test_authorization_error() {
        let error = AuthError::authorization_error("Insufficient permissions");

        assert!(matches!(error, AuthError::Authorization { .. }));
        assert_eq!(
            error.to_string(),
            "Authorization error: Insufficient permissions"
        );
    }

    #[test]
    fn test_session_error() {
        let error = AuthError::session_error("Session expired");

        assert!(matches!(error, AuthError::Session { .. }));
        assert_eq!(error.to_string(), "Session error: Session expired");
    }

    #[test]
    fn test_configuration_error() {
        let error = AuthError::configuration_error("Missing required config");

        assert!(matches!(error, AuthError::Configuration { .. }));
        assert_eq!(
            error.to_string(),
            "Auth configuration error: Missing required config"
        );
    }

    #[test]
    fn test_network_error() {
        let error = AuthError::network_error("login", "Connection timeout");

        assert!(matches!(error, AuthError::Network { .. }));
        assert_eq!(
            error.to_string(),
            "Network error during login: Connection timeout"
        );
    }

    #[test]
    fn test_beardog_error() {
        let error = AuthError::beardog_error("Integration failed");

        assert!(matches!(error, AuthError::BeardogIntegration { .. }));
        assert_eq!(
            error.to_string(),
            "Beardog integration error: Integration failed"
        );
    }

    #[test]
    fn test_internal_error() {
        let error = AuthError::internal_error("Database connection failed");

        assert!(matches!(error, AuthError::Internal { .. }));
        assert_eq!(
            error.to_string(),
            "Internal auth system error: Database connection failed"
        );
    }

    #[tokio::test]
    async fn test_from_reqwest_error() {
        // Create a mock reqwest error (using invalid URL)
        let reqwest_err = reqwest::get("not_a_valid_url").await.unwrap_err();

        let auth_error: AuthError = reqwest_err.into();

        assert!(matches!(auth_error, AuthError::Network { .. }));
        assert!(auth_error.to_string().contains("http_request"));
    }

    #[test]
    fn test_from_serde_json_error() {
        // Create a serde_json error
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();

        let auth_error: AuthError = json_err.into();

        assert!(matches!(auth_error, AuthError::Internal { .. }));
        assert!(auth_error.to_string().contains("JSON serialization error"));
    }

    #[test]
    fn test_from_uuid_error() {
        // Create a UUID error
        let uuid_err = uuid::Uuid::parse_str("not-a-valid-uuid").unwrap_err();

        let auth_error: AuthError = uuid_err.into();

        assert!(matches!(auth_error, AuthError::Internal { .. }));
        assert!(auth_error.to_string().contains("UUID error"));
    }

    #[test]
    fn test_error_with_string() {
        let error = AuthError::authentication_failed(String::from("Dynamic message"));
        assert_eq!(error.to_string(), "Authentication failed: Dynamic message");
    }

    #[test]
    fn test_error_with_str_slice() {
        let message = "Static message";
        let error = AuthError::authentication_failed(message);
        assert_eq!(error.to_string(), "Authentication failed: Static message");
    }

    #[test]
    fn test_auth_result_ok() {
        let result: AuthResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_auth_result_err() {
        let result: AuthResult<String> = Err(AuthError::authentication_failed("test"));
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::AuthenticationFailed { .. }));
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = AuthError::authentication_failed("test");
        let debug_str = format!("{:?}", error);

        assert!(debug_str.contains("AuthenticationFailed"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_multiple_error_variants() {
        let errors = vec![
            AuthError::authentication_failed("auth"),
            AuthError::token_error("op", "msg"),
            AuthError::authorization_error("authz"),
            AuthError::session_error("session"),
            AuthError::configuration_error("config"),
            AuthError::network_error("net", "err"),
            AuthError::beardog_error("beardog"),
            AuthError::internal_error("internal"),
        ];

        // Verify all errors have unique display messages
        let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        assert_eq!(messages.len(), 8);

        // All should be distinct
        for (i, msg1) in messages.iter().enumerate() {
            for (j, msg2) in messages.iter().enumerate() {
                if i != j {
                    assert_ne!(msg1, msg2, "Error messages should be distinct");
                }
            }
        }
    }

    #[test]
    fn test_error_chaining_with_question_mark() {
        fn inner_function() -> AuthResult<i32> {
            Err(AuthError::authentication_failed("inner error"))
        }

        fn outer_function() -> AuthResult<i32> {
            let value = inner_function()?;
            Ok(value + 1)
        }

        let result = outer_function();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::AuthenticationFailed { .. }
        ));
    }

    #[test]
    fn test_error_with_empty_message() {
        let error = AuthError::authentication_failed("");
        assert_eq!(error.to_string(), "Authentication failed: ");
    }

    #[test]
    fn test_error_with_special_characters() {
        let error = AuthError::authentication_failed("Error with symbols: @#$%^&*()");
        assert!(error.to_string().contains("@#$%^&*()"));
    }

    #[test]
    fn test_error_with_unicode() {
        let error = AuthError::authentication_failed("用户认证失败");
        assert!(error.to_string().contains("用户认证失败"));
    }

    #[test]
    fn test_error_with_newlines() {
        let error = AuthError::authentication_failed("Line 1\nLine 2\nLine 3");
        assert!(error.to_string().contains("Line 1\nLine 2\nLine 3"));
    }
}
