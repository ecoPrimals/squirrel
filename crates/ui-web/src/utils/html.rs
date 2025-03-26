/// Utilities for HTML escape and manipulation

/// Encode a string for HTML, escaping special characters
pub fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

/// Encode a string for HTML attributes
pub fn escape_attribute(s: &str) -> String {
    escape_html(s)
}

/// Create a module called htmlescape for compatibility with existing code
pub mod htmlescape {
    use super::*;
    
    /// Encode the minimal set of characters needed for HTML safety
    pub fn encode_minimal(s: &str) -> String {
        escape_html(s)
    }
    
    /// Encode for safe HTML attribute usage
    pub fn encode_attribute(s: &str) -> String {
        escape_attribute(s)
    }
} 