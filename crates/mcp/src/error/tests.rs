#[cfg(test)]
mod tests {
    use crate::error::transport::TransportError as CanonicalTransportError;
    use crate::error::MCPError;
    use std::fmt::Debug;

    #[test]
    fn test_transport_error_conversions() {
        // Test canonical to simplified conversion
        let canonical_error = CanonicalTransportError::ConnectionFailed("Failed connection".to_string());
        let simplified_error: crate::error::TransportError = canonical_error.clone().into();
        
        // Verify the variant matches
        match simplified_error {
            crate::error::TransportError::ConnectionFailed(msg) => {
                assert!(msg.contains("Failed connection"));
                println!("Successfully converted canonical->simplified ConnectionFailed");
            },
            _ => panic!("Expected ConnectionFailed variant")
        }
        
        // Test simplified to canonical conversion
        let simplified_error = crate::error::TransportError::Timeout("Connection timeout".to_string());
        let canonical_error: CanonicalTransportError = simplified_error.into();
        
        // Verify the variant matches
        match canonical_error {
            CanonicalTransportError::Timeout(msg) => {
                assert!(msg.contains("Connection timeout"));
                println!("Successfully converted simplified->canonical Timeout");
            },
            _ => panic!("Expected Timeout variant")
        }
        
        // Test MCPError wrapping of simplified error
        let simplified_error = crate::error::TransportError::IoError("IO failure".to_string());
        let mcp_error = MCPError::Transport(simplified_error);
        
        // Verify error type extraction
        match &mcp_error {
            MCPError::Transport(err) => {
                match err {
                    crate::error::TransportError::IoError(msg) => {
                        assert!(msg.contains("IO failure"));
                        println!("Successfully extracted IoError from MCPError");
                    },
                    _ => panic!("Expected IoError variant")
                }
            },
            _ => panic!("Expected MCPError::Transport")
        }
        
        println!("All transport error conversion tests passed!");
    }
} 