use serde::{Deserialize, Serialize};

/// Configuration for the web server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Base URL for the API
    pub api_base_url: String,
    /// Timeout for API requests
    pub request_timeout_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:8000".to_string(),
            request_timeout_secs: 30,
        }
    }
} 