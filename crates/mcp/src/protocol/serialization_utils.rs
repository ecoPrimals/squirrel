// Define helper structs for serialization/deserialization

pub mod serialization_utils {
    use crate::error::{MCPError, Result};
    use crate::protocol::adapter_wire::WireFormatError;
    use serde_json::Value;
    
    /// Helper function to extract a string field from a JSON object
    pub fn extract_string(obj: &serde_json::Map<String, Value>, field: &str) -> Result<String> {
        Ok(obj.get(field)
            .ok_or_else(|| MCPError::from(WireFormatError::MissingField(field.to_string())))?
            .as_str()
            .ok_or_else(|| MCPError::from(WireFormatError::InvalidFieldValue(field.to_string(), "not a string".to_string())))?
            .to_string())
    }
} 