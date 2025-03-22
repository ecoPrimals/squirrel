pub use sqlx::SqlitePool;

/// Database options
#[derive(Debug, Clone)]
pub struct DatabaseOptions {
    /// Database URL
    pub url: String,
    /// Max connections in the pool
    pub max_connections: u32,
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 5,
        }
    }
} 