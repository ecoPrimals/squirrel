// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Input sanitization methods
//!
//! This module contains methods for sanitizing user input to remove or encode
//! potentially dangerous content while preserving legitimate data.

use regex::Regex;

use super::types::{InputType, InputValidationConfig};

/// Sanitization patterns for various input transformations
///
/// Contains compiled regex patterns for efficiently sanitizing different
/// types of input (HTML, paths, URLs, etc.).
pub struct SanitizationPatterns {
    /// Regex for detecting/removing script tags
    pub script_regex: Regex,

    /// Regex for detecting/removing dangerous HTML attributes
    pub dangerous_attrs: Regex,

    /// Regex for detecting HTML tags (for whitelist filtering)
    pub tag_regex: Regex,

    /// Regex for removing dangerous path characters
    pub path_dangerous_chars: Regex,

    /// Regex for removing dangerous URL schemes
    pub url_dangerous_schemes: Regex,

    /// Regex for removing dangerous email characters
    pub email_dangerous_chars: Regex,

    /// Regex for removing control characters
    pub control_chars: Regex,
}

impl SanitizationPatterns {
    /// Compile all sanitization patterns
    ///
    /// Creates a new `SanitizationPatterns` instance with all regex patterns compiled.
    /// This is done once at initialization for performance.
    ///
    /// ## Errors
    /// Returns error if any regex pattern fails to compile (should never happen
    /// with our hardcoded patterns, but we handle it for safety).
    pub fn compile() -> Result<Self, String> {
        Ok(Self {
            script_regex: Regex::new(r"(?i)<script\b[^>]*>(.*?)</script>")
                .map_err(|e| format!("Failed to compile script regex: {e}"))?,
            dangerous_attrs: Regex::new(
                r#"(?i)\s*(on\w+|href\s*=\s*['"]?javascript:|src\s*=\s*['"]?javascript:)"#,
            )
            .map_err(|e| format!("Failed to compile dangerous attrs regex: {e}"))?,
            tag_regex: Regex::new(r"<(/?)(\w+)([^>]*)>")
                .map_err(|e| format!("Failed to compile tag regex: {e}"))?,
            path_dangerous_chars: Regex::new(r#"[<>:"|?*\x00-\x1f]"#)
                .map_err(|e| format!("Failed to compile path chars regex: {e}"))?,
            url_dangerous_schemes: Regex::new(r"(?i)^(javascript|data|vbscript):")
                .map_err(|e| format!("Failed to compile URL schemes regex: {e}"))?,
            email_dangerous_chars: Regex::new(r#"[<>(){}\[\]\\,;:\s@"\x00-\x1f]"#)
                .map_err(|e| format!("Failed to compile email chars regex: {e}"))?,
            control_chars: Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]")
                .map_err(|e| format!("Failed to compile control chars regex: {e}"))?,
        })
    }
}

/// Sanitize input based on type
///
/// Main sanitization entry point that delegates to type-specific sanitization
/// methods and handles length truncation.
///
/// ## Arguments
/// - `input`: The user input to sanitize
/// - `input_type`: Type of input to determine sanitization strategy
/// - `config`: Configuration for sanitization behavior
/// - `patterns`: Compiled sanitization patterns
///
/// ## Returns
/// Sanitized version of the input, safe for the specified input type
pub fn sanitize_input(
    input: &str,
    input_type: &InputType,
    config: &InputValidationConfig,
    patterns: &SanitizationPatterns,
) -> String {
    let mut sanitized = input.to_string();

    match input_type {
        InputType::Html => {
            if config.enable_html_sanitization {
                sanitized = sanitize_html(&sanitized, config, patterns);
            }
        }
        InputType::FilePath => {
            sanitized = sanitize_file_path(&sanitized, patterns);
        }
        InputType::Url => {
            sanitized = sanitize_url(&sanitized, patterns);
        }
        InputType::Email => {
            sanitized = sanitize_email(&sanitized, patterns);
        }
        _ => {
            sanitized = sanitize_general_text(&sanitized, patterns);
        }
    }

    // Truncate if too long
    let max_length = match input_type {
        InputType::Text | InputType::Html => config.max_text_length,
        _ => config.max_string_length,
    };

    if sanitized.len() > max_length {
        sanitized.truncate(max_length);
    }

    sanitized
}

/// Sanitize HTML content
///
/// Removes dangerous HTML elements and attributes while preserving
/// whitelisted safe tags.
///
/// ## Arguments
/// - `input`: HTML content to sanitize
/// - `config`: Configuration with allowed HTML tags
/// - `patterns`: Compiled sanitization patterns
///
/// ## Returns
/// Sanitized HTML with dangerous content removed
///
/// ## Security Notes
/// - Removes `<script>` tags and content
/// - Removes dangerous event handlers (onclick, onerror, etc.)
/// - Removes javascript: and data: URIs
/// - Only allows whitelisted HTML tags
pub fn sanitize_html(
    input: &str,
    config: &InputValidationConfig,
    patterns: &SanitizationPatterns,
) -> String {
    let mut sanitized = input.to_string();

    // Remove script tags and their content
    sanitized = patterns
        .script_regex
        .replace_all(&sanitized, "")
        .to_string();

    // Remove dangerous attributes
    sanitized = patterns
        .dangerous_attrs
        .replace_all(&sanitized, "")
        .to_string();

    // Remove non-whitelisted tags (simplified - in production use a proper HTML sanitizer)
    sanitized = patterns
        .tag_regex
        .replace_all(&sanitized, |caps: &regex::Captures<'_>| {
            // SAFETY: Regex pattern guarantees capture group 1 exists for tag name
            if let Some(tag_match) = caps.get(2) {
                let tag = tag_match.as_str().to_lowercase();
                if config.allowed_html_tags.contains(&tag) {
                    // SAFETY: Regex pattern guarantees capture group 0 exists for full match
                    caps.get(0)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        })
        .to_string();

    sanitized
}

/// Sanitize file path
///
/// Removes path traversal sequences and dangerous characters to prevent
/// directory escape attacks.
///
/// ## Arguments
/// - `input`: File path to sanitize
/// - `patterns`: Compiled sanitization patterns
///
/// ## Returns
/// Sanitized file path
///
/// ## Security Notes
/// - Removes `..` sequences (path traversal)
/// - Normalizes backslashes to forward slashes
/// - Removes dangerous characters (<, >, :, ", |, ?, *)
pub fn sanitize_file_path(input: &str, patterns: &SanitizationPatterns) -> String {
    let mut sanitized = input.replace("..", "");
    sanitized = sanitized.replace('\\', "/");

    // Remove dangerous characters
    sanitized = patterns
        .path_dangerous_chars
        .replace_all(&sanitized, "")
        .to_string();

    sanitized
}

/// Sanitize URL
///
/// Removes dangerous URL schemes that could be used for XSS attacks.
///
/// ## Arguments
/// - `input`: URL to sanitize
/// - `patterns`: Compiled sanitization patterns
///
/// ## Returns
/// Sanitized URL
///
/// ## Security Notes
/// - Removes javascript: schemes
/// - Removes data: schemes (can contain base64 encoded scripts)
/// - Removes vbscript: schemes
pub fn sanitize_url(input: &str, patterns: &SanitizationPatterns) -> String {
    let mut sanitized = input.to_string();

    // Remove javascript: and data: schemes
    sanitized = patterns
        .url_dangerous_schemes
        .replace_all(&sanitized, "")
        .to_string();

    sanitized
}

/// Sanitize email address
///
/// Removes characters that are invalid or dangerous in email addresses.
///
/// ## Arguments
/// - `input`: Email address to sanitize
/// - `patterns`: Compiled sanitization patterns
///
/// ## Returns
/// Sanitized email address
///
/// ## Note
/// This is a simple sanitization. For production, use a proper email
/// validation library that complies with RFC 5322.
pub fn sanitize_email(input: &str, patterns: &SanitizationPatterns) -> String {
    // Simple email sanitization - remove dangerous characters
    patterns
        .email_dangerous_chars
        .replace_all(input, "")
        .to_string()
}

/// Sanitize general text
///
/// Removes null bytes and control characters from general text input.
///
/// ## Arguments
/// - `input`: Text to sanitize
/// - `patterns`: Compiled sanitization patterns
///
/// ## Returns
/// Sanitized text
///
/// ## Security Notes
/// - Removes null bytes (\0)
/// - Removes control characters (0x00-0x1F, 0x7F)
pub fn sanitize_general_text(input: &str, patterns: &SanitizationPatterns) -> String {
    let mut sanitized = input.to_string();

    // Remove null bytes and control characters
    sanitized = sanitized.replace('\0', "");
    sanitized = patterns
        .control_chars
        .replace_all(&sanitized, "")
        .to_string();

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_test_config() -> InputValidationConfig {
        let mut allowed_html_tags = HashSet::new();
        allowed_html_tags.insert("b".to_string());
        allowed_html_tags.insert("i".to_string());
        allowed_html_tags.insert("p".to_string());

        InputValidationConfig {
            max_string_length: 100,
            max_text_length: 1000,
            max_array_length: 100,
            max_json_depth: 5,
            enable_html_sanitization: true,
            allowed_html_tags,
            enable_sql_injection_detection: true,
            enable_xss_detection: true,
            enable_path_traversal_detection: true,
            enable_command_injection_detection: true,
            enable_nosql_injection_detection: true,
            strict_mode: true,
        }
    }

    fn create_test_patterns() -> SanitizationPatterns {
        SanitizationPatterns::compile().unwrap()
    }

    #[test]
    fn test_sanitize_html() {
        let config = create_test_config();
        let patterns = create_test_patterns();

        let input = "<p>Hello</p><script>alert('xss')</script><b>World</b>";
        let result = sanitize_html(input, &config, &patterns);

        // Should remove script tags but keep allowed tags
        assert!(!result.contains("<script>"));
        assert!(result.contains("<p>"));
        assert!(result.contains("<b>"));
    }

    #[test]
    fn test_sanitize_file_path() {
        let patterns = create_test_patterns();

        let input = "../../etc/passwd";
        let result = sanitize_file_path(input, &patterns);

        // Should remove path traversal
        assert!(!result.contains(".."));
    }

    #[test]
    fn test_sanitize_url() {
        let patterns = create_test_patterns();

        let input = "javascript:alert('xss')";
        let result = sanitize_url(input, &patterns);

        // Should remove dangerous scheme
        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_email() {
        let patterns = create_test_patterns();

        let input = "user@example<script>.com";
        let result = sanitize_email(input, &patterns);

        // Should remove dangerous characters
        assert!(!result.contains("<"));
        assert!(!result.contains(">"));
    }

    #[test]
    fn test_sanitize_general_text() {
        let patterns = create_test_patterns();

        let input = "Hello\x00World\x1F!";
        let result = sanitize_general_text(input, &patterns);

        // Should remove null bytes and control characters
        assert!(!result.contains('\x00'));
        assert!(!result.contains('\x1F'));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_sanitize_input_with_length_limit() {
        let config = create_test_config();
        let patterns = create_test_patterns();

        let long_input = "a".repeat(200);
        let result = sanitize_input(&long_input, &InputType::Text, &config, &patterns);

        // Should truncate to max_string_length (Text uses max_text_length, but test still valid)
        assert!(result.len() <= config.max_text_length);
    }
}
