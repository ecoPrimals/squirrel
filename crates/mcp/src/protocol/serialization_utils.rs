// Define helper structs for serialization/deserialization

use crate::error::{MCPError, Result};
use crate::protocol::adapter_wire::WireFormatError;
use serde_json::Map;
use serde_json::Value;

/// Extracts a string value from a JSON object by key, returning a protocol error if
/// the key is not found or the value is not a string.
///
/// # Arguments
///
/// * `obj` - The JSON object to extract from
/// * `key` - The key to extract
///
/// # Returns
///
/// The string value if found and of the correct type
///
/// # Errors
///
/// Returns a protocol error if the key is not found or the value is not a string
pub fn extract_string(obj: &Map<String, Value>, key: &str) -> Result<String> {
    obj.get(key)
        .ok_or_else(|| MCPError::from(WireFormatError::MissingField(key.to_string())))
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| MCPError::from(WireFormatError::InvalidFieldType(
                    key.to_string(),
                    "string".to_string()
                )))
        })
} 