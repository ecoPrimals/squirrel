// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the SDKError-based error system

use super::*;
use universal_error::sdk::{
    ClientError, CommunicationError, InfrastructureError, SDKError,
};

macro_rules! sdk_error_test {
    ($name:ident, $body:block) => {
        #[cfg_attr(not(target_arch = "wasm32"), test)]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn $name() $body
    };
}

sdk_error_test!(test_sdk_error_variants, {
    let infra: SDKError = InfrastructureError::Configuration("bad".into()).into();
    assert!(matches!(infra, SDKError::Infrastructure(_)));

    let comm: SDKError = CommunicationError::MCP("proto".into()).into();
    assert!(matches!(comm, SDKError::Communication(_)));

    let client: SDKError = ClientError::Timeout(30).into();
    assert!(matches!(client, SDKError::Client(_)));

    let general = SDKError::General("oops".into());
    assert!(matches!(general, SDKError::General(_)));
});

sdk_error_test!(test_validation_helpers, {
    let params = serde_json::json!({
        "name": "test",
        "count": 42,
        "enabled": true,
        "items": [1, 2, 3],
        "config": {"key": "value"}
    });

    assert_eq!(
        validation::validate_required_string(&params, "name").expect("should succeed"),
        "test"
    );
    let count = validation::validate_required_number(&params, "count").expect("should succeed");
    assert!((count - 42.0).abs() < f64::EPSILON);
    assert!(validation::validate_boolean(&params, "enabled", false).expect("should succeed"));
    assert_eq!(
        validation::validate_array(&params, "items")
            .expect("should succeed")
            .len(),
        3
    );
    assert_eq!(
        validation::validate_object(&params, "config")
            .expect("should succeed")
            .len(),
        1
    );
});

sdk_error_test!(test_serde_json_to_sdk_error, {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_error.is_err());

    let sdk_error: SDKError = json_error.unwrap_err().into();
    assert!(matches!(
        sdk_error,
        SDKError::Communication(CommunicationError::Serialization(_))
    ));
});

sdk_error_test!(test_validation_error_to_sdk_error, {
    let ve = validation::ValidationError::RequiredField {
        field: "name".into(),
    };
    let sdk_error: SDKError = ve.into();
    assert!(matches!(
        sdk_error,
        SDKError::Infrastructure(InfrastructureError::Validation(_))
    ));
});

sdk_error_test!(test_sdk_error_recoverability, {
    use universal_error::ErrorContextTrait;

    let recoverable: SDKError = ClientError::Timeout(10).into();
    assert!(recoverable.is_recoverable());

    let not_recoverable = SDKError::General("permanent".into());
    assert!(!not_recoverable.is_recoverable());
});

sdk_error_test!(test_sdk_error_display, {
    let err: SDKError = CommunicationError::MCP("protocol error".into()).into();
    let display = err.to_string();
    assert!(display.contains("protocol error"));
});

sdk_error_test!(test_validation_helpers_extended, {
    assert!(validation::validate_url("https://example.com", "url").is_ok());
    assert!(validation::validate_url("invalid-url", "url").is_err());

    assert!(validation::validate_email("test@example.com", "email").is_ok());
    assert!(validation::validate_email("invalid-email", "email").is_err());

    assert!(validation::validate_non_empty_string("value", "field").is_ok());
    assert!(validation::validate_non_empty_string("", "field").is_err());

    let items = vec![
        serde_json::json!(1),
        serde_json::json!(2),
        serde_json::json!(3),
    ];
    assert!(validation::validate_array_length(&items, "items", 1, 5).is_ok());
    assert!(validation::validate_array_length(&items, "items", 10, 20).is_err());
});

sdk_error_test!(test_io_error_to_sdk_error, {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let sdk_err: SDKError = io_err.into();
    assert!(matches!(sdk_err, SDKError::General(_)));
    assert!(sdk_err.to_string().contains("IO:"));
});

sdk_error_test!(test_parse_error_to_sdk_error, {
    let parse_err: SDKError = "abc".parse::<i32>().unwrap_err().into();
    assert!(matches!(
        parse_err,
        SDKError::Infrastructure(InfrastructureError::Validation(_))
    ));
});

#[test]
#[expect(deprecated, reason = "testing bridge from deprecated PluginError")]
fn test_plugin_error_bridge() {
    let pe = core::PluginError::McpError {
        message: "bridge test".into(),
    };
    let sdk: SDKError = pe.into();
    assert!(matches!(
        sdk,
        SDKError::Communication(CommunicationError::MCP(_))
    ));
}
