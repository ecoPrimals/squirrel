//! Utility functions for the universal configuration system
//!
//! This module contains helper functions for parsing, validation, and other
//! utilities used throughout the configuration system.

use crate::universal::types::ConfigError;
use std::time::Duration;
use url::Url;

/// Parse duration from string
///
/// Supports formats:
/// - "1000ms" - milliseconds
/// - "30s" - seconds
/// - "5m" - minutes
/// - "2h" - hours
/// - "60" - default to seconds
pub fn parse_duration(s: &str) -> Result<Duration, std::num::ParseIntError> {
    if s.ends_with("ms") {
        let ms = s.trim_end_matches("ms").parse::<u64>()?;
        Ok(Duration::from_millis(ms))
    } else if s.ends_with("s") {
        let secs = s.trim_end_matches("s").parse::<u64>()?;
        Ok(Duration::from_secs(secs))
    } else if s.ends_with("m") {
        let mins = s.trim_end_matches("m").parse::<u64>()?;
        Ok(Duration::from_secs(mins * 60))
    } else if s.ends_with("h") {
        let hours = s.trim_end_matches("h").parse::<u64>()?;
        Ok(Duration::from_secs(hours * 3600))
    } else {
        // Default to seconds
        let secs = s.parse::<u64>()?;
        Ok(Duration::from_secs(secs))
    }
}

/// Validate URL format
///
/// Checks if the provided string is a valid URL format.
/// Supports http, https, ws, and wss protocols.
pub fn validate_url(url: &str) -> Result<(), ConfigError> {
    if url.is_empty() {
        return Err(ConfigError::InvalidUrl("URL cannot be empty".to_string()));
    }

    Url::parse(url).map_err(|e| ConfigError::InvalidUrl(format!("Invalid URL '{url}': {e}")))?;

    Ok(())
}

/// Parse comma-separated values
///
/// Splits a string by commas and trims whitespace from each value.
pub fn parse_comma_separated(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse key-value pairs from string
///
/// Parses a string in the format "key1=value1,key2=value2" into a HashMap.
pub fn parse_key_value_pairs(
    s: &str,
) -> Result<std::collections::HashMap<String, String>, ConfigError> {
    let mut map = std::collections::HashMap::new();

    if s.is_empty() {
        return Ok(map);
    }

    for pair in s.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() != 2 {
            return Err(ConfigError::InvalidServiceConfig(format!(
                "Invalid key-value pair: {pair}"
            )));
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        if key.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Key cannot be empty".to_string(),
            ));
        }

        map.insert(key, value);
    }

    Ok(map)
}

/// Parse HTTP status codes from string
///
/// Parses a comma-separated list of HTTP status codes.
pub fn parse_http_status_codes(s: &str) -> Result<Vec<u16>, ConfigError> {
    if s.is_empty() {
        return Ok(vec![200]); // Default to 200 OK
    }

    s.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u16>().map_err(|_| {
                ConfigError::InvalidServiceConfig(format!("Invalid HTTP status code: {s}"))
            })
        })
        .collect()
}

/// Validate weight value
///
/// Ensures weight is between 0.0 and 1.0.
pub fn validate_weight(weight: f32) -> Result<(), ConfigError> {
    if !(0.0..=1.0).contains(&weight) {
        return Err(ConfigError::InvalidServiceConfig(format!(
            "Weight must be between 0.0 and 1.0, got: {weight}"
        )));
    }
    Ok(())
}

/// Validate port number
///
/// Ensures port is within valid range.
pub fn validate_port(port: u16) -> Result<(), ConfigError> {
    if port == 0 {
        return Err(ConfigError::InvalidPort(port));
    }
    Ok(())
}

/// Format duration for display
///
/// Formats a duration in a human-readable way.
pub fn format_duration(duration: &Duration) -> String {
    let secs = duration.as_secs();
    let ms = duration.subsec_millis();

    if secs == 0 {
        format!("{ms}ms")
    } else if secs < 60 {
        if ms > 0 {
            format!("{secs}.{ms:03}s")
        } else {
            format!("{secs}s")
        }
    } else if secs < 3600 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        if remaining_secs > 0 {
            format!("{mins}m{remaining_secs}s")
        } else {
            format!("{mins}m")
        }
    } else {
        let hours = secs / 3600;
        let remaining_mins = (secs % 3600) / 60;
        let remaining_secs = secs % 60;

        if remaining_secs > 0 {
            format!("{hours}h{remaining_mins}m{remaining_secs}s")
        } else if remaining_mins > 0 {
            format!("{hours}h{remaining_mins}m")
        } else {
            format!("{hours}h")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(
            parse_duration("1000ms").unwrap(),
            Duration::from_millis(1000)
        );
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("https://api.example.com").is_ok());
        assert!(validate_url("ws://localhost:8080").is_ok());
        assert!(validate_url("wss://secure.example.com").is_ok());

        assert!(validate_url("").is_err());
        assert!(validate_url("invalid-url").is_err());
    }

    #[test]
    fn test_parse_comma_separated() {
        assert_eq!(parse_comma_separated("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(parse_comma_separated("a, b , c "), vec!["a", "b", "c"]);
        assert_eq!(parse_comma_separated(""), Vec::<String>::new());
    }

    #[test]
    fn test_parse_key_value_pairs() {
        let result = parse_key_value_pairs("key1=value1,key2=value2").unwrap();
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));

        let result = parse_key_value_pairs("").unwrap();
        assert!(result.is_empty());

        assert!(parse_key_value_pairs("invalid").is_err());
        assert!(parse_key_value_pairs("=value").is_err());
    }

    #[test]
    fn test_parse_http_status_codes() {
        assert_eq!(
            parse_http_status_codes("200,201,204").unwrap(),
            vec![200, 201, 204]
        );
        assert_eq!(parse_http_status_codes("").unwrap(), vec![200]);
        assert!(parse_http_status_codes("invalid").is_err());
    }

    #[test]
    fn test_validate_weight() {
        assert!(validate_weight(0.0).is_ok());
        assert!(validate_weight(0.5).is_ok());
        assert!(validate_weight(1.0).is_ok());
        assert!(validate_weight(-0.1).is_err());
        assert!(validate_weight(1.1).is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(65535).is_ok());
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(&Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(&Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(&Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(&Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(&Duration::from_secs(3661)), "1h1m1s");
    }
}
