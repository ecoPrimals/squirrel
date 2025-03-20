use std::collections::HashMap;
use uuid::Uuid;
use std::path::PathBuf;

/// Generate a unique test identifier
pub fn generate_test_id() -> String {
    format!("test_{}", Uuid::new_v4())
}

/// Generate a test file path
pub fn generate_test_file_path() -> PathBuf {
    std::env::temp_dir().join(format!("test_file_{}.txt", Uuid::new_v4()))
}

/// Generate test user data
pub fn generate_test_user() -> HashMap<String, String> {
    let mut user = HashMap::new();
    user.insert("id".to_string(), Uuid::new_v4().to_string());
    user.insert("username".to_string(), format!("user_{}", Uuid::new_v4().simple()));
    user.insert("email".to_string(), format!("user_{}@example.com", Uuid::new_v4().simple()));
    user
}

/// Generate a test configuration map
pub fn generate_test_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("test_mode".to_string(), "true".to_string());
    config.insert("log_level".to_string(), "debug".to_string());
    config.insert("timeout".to_string(), "30".to_string());
    config.insert("data_dir".to_string(), std::env::temp_dir().to_string_lossy().to_string());
    config
}

/// Generate test message data
pub fn generate_test_message(msg_type: &str) -> HashMap<String, String> {
    let mut message = HashMap::new();
    message.insert("id".to_string(), Uuid::new_v4().to_string());
    message.insert("type".to_string(), msg_type.to_string());
    message.insert("timestamp".to_string(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string());
    message.insert("payload".to_string(), format!("payload_{}", Uuid::new_v4().simple()));
    message
}

/// Generate random test data of specified size
pub fn generate_random_data(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rand::random::<u8>());
    }
    data
}

/// Generate a test JSON string
pub fn generate_test_json() -> String {
    let user = generate_test_user();
    serde_json::to_string(&user).unwrap_or_else(|_| "{}".to_string())
}

/// Generate test metrics data
pub fn generate_test_metrics() -> HashMap<String, f64> {
    let mut metrics = HashMap::new();
    metrics.insert("cpu_usage".to_string(), rand::random::<f64>() * 100.0);
    metrics.insert("memory_usage".to_string(), rand::random::<f64>() * 1024.0);
    metrics.insert("disk_usage".to_string(), rand::random::<f64>() * 100.0);
    metrics.insert("network_in".to_string(), rand::random::<f64>() * 1000.0);
    metrics.insert("network_out".to_string(), rand::random::<f64>() * 1000.0);
    metrics
} 