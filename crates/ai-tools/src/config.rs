use directories::ProjectDirs;
use secrecy::{DebugSecret, ExposeSecret, Secret, SerializableSecret};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use zeroize::Zeroize;
use std::env;
use base64::{Engine as _, engine::general_purpose};

/// A wrapper type for strings that should be kept secret.
/// This type implements `Zeroize` and `DebugSecret` to ensure proper handling of sensitive data.
#[derive(Clone)]
pub struct SecretString(pub String);

// Custom Serialize implementation to encode the secret value
impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Encode with base64 to avoid storing plaintext
        let encoded = general_purpose::STANDARD.encode(&self.0);
        let prefix = "_SECRET_";
        serializer.serialize_str(&format!("{}{}", prefix, encoded))
    }
}

// Custom Deserialize implementation to decode the secret value
impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        
        // Check if this is an encoded secret
        let prefix = "_SECRET_";
        if s.starts_with(prefix) {
            let encoded = &s[prefix.len()..];
            match general_purpose::STANDARD.decode(encoded) {
                Ok(bytes) => {
                    match String::from_utf8(bytes) {
                        Ok(decoded) => Ok(SecretString(decoded)),
                        Err(_) => Ok(SecretString(s))
                    }
                },
                Err(_) => Ok(SecretString(s))
            }
        } else {
            // Old format or plaintext - will be re-encoded on next save
            Ok(SecretString(s))
        }
    }
}

impl Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl DebugSecret for SecretString {}

impl Zeroize for SecretString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl SerializableSecret for SecretString {}

impl ExposeSecret<String> for SecretString {
    fn expose_secret(&self) -> &String {
        &self.0
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        SecretString(s)
    }
}

impl From<SecretString> for String {
    fn from(s: SecretString) -> Self {
        s.0
    }
}

// Thread-local storage for the test directory path
#[cfg(test)]
thread_local! {
    static TEST_CONFIG_DIR: RefCell<Option<String>> = RefCell::new(None);
}

// Helper functions for tests
#[cfg(test)]
pub fn setup_test_dir() -> String {
    let temp_dir = TempDir::new().unwrap();
    let test_dir_path = temp_dir.path().canonicalize().unwrap().to_string_lossy().to_string();
    
    // Don't keep a reference to the TempDir - we need it to stay alive for the duration of the test
    std::mem::forget(temp_dir);
    
    // Set the thread-local storage
    TEST_CONFIG_DIR.with(|dir| {
        *dir.borrow_mut() = Some(test_dir_path.clone());
    });
    
    println!("Set up test directory: {}", test_dir_path);
    test_dir_path
}

#[cfg(test)]
pub fn get_test_dir() -> Option<String> {
    TEST_CONFIG_DIR.with(|dir| dir.borrow().clone())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub openai_api_key: Secret<SecretString>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        
        #[cfg(test)]
        println!("Loading config from: {:?}", config_path);
        
        if !config_path.exists() {
            #[cfg(test)]
            println!("Config file does not exist, returning empty config");
            
            return Ok(Self {
                openai_api_key: Secret::new(SecretString(String::new())),
            });
        }
        
        #[cfg(test)]
        println!("Reading config file content");
        
        let contents = fs::read_to_string(&config_path)?;
        
        #[cfg(test)]
        println!("Parsing config file");
        
        let config: Config = toml::from_str(&contents)?;
        
        #[cfg(test)]
        println!("Config loaded successfully");
        
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        
        #[cfg(test)]
        println!("Saving config to: {:?}", config_path);
        
        // Ensure the parent directory exists
        if let Some(parent) = config_path.parent() {
            #[cfg(test)]
            println!("Creating directory: {:?}", parent);
            
            fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string(self)?;
        fs::write(&config_path, contents)?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&config_path, perms)?;
        }
        
        #[cfg(test)]
        println!("Config saved successfully");
        
        Ok(())
    }

    pub fn set_openai_api_key(&mut self, key: impl Into<String>) {
        self.openai_api_key = Secret::new(SecretString(key.into()));
    }

    // Regular config_path implementation for non-test code
    #[cfg(not(test))]
    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // First check if the environment variable is set
        if let Ok(config_dir) = env::var("SQUIRREL_CONFIG_DIR") {
            let config_path = PathBuf::from(config_dir).join(".config").join("squirrel").join("ai-tools");
            fs::create_dir_all(&config_path)?;
            return Ok(config_path.join("config.toml"));
        }
        
        // Use standard config directory for normal operation
        let proj_dirs = ProjectDirs::from("com", "squirrel", "ai-tools")
            .ok_or("Failed to determine config directory")?;
            
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    #[cfg(test)]
    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // First, try to get the test directory from thread-local storage
        if let Some(test_dir) = get_test_dir() {
            println!("Using test directory from thread-local: {}", test_dir);
            let config_path = PathBuf::from(test_dir).join(".config").join("squirrel").join("ai-tools");
            fs::create_dir_all(&config_path)?;
            return Ok(config_path.join("config.toml"));
        }
        
        // Second, check if the environment variable is set
        if let Ok(config_dir) = env::var("SQUIRREL_CONFIG_DIR") {
            println!("Using config directory from environment: {}", config_dir);
            let config_path = PathBuf::from(config_dir).join(".config").join("squirrel").join("ai-tools");
            fs::create_dir_all(&config_path)?;
            return Ok(config_path.join("config.toml"));
        }
        
        // Fall back to normal behavior (shouldn't happen in tests)
        println!("Warning: No test directory found in thread-local storage or environment!");
        let proj_dirs = ProjectDirs::from("com", "squirrel", "ai-tools")
            .ok_or("Failed to determine config directory")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    static INIT: Once = Once::new();
    
    #[test]
    fn test_config_save_load() {
        let _test_dir = setup_test_dir();
        let test_key = "test-key-123".to_string();
        
        // Create and save config
        let mut config = Config {
            openai_api_key: Secret::new(SecretString(String::new())),
        };
        config.set_openai_api_key(&test_key);
        config.save().unwrap();
        
        // Load config and verify
        let loaded_config = Config::load().unwrap();
        assert_eq!(
            loaded_config.openai_api_key.expose_secret().0,
            test_key
        );
    }
    
    #[test]
    fn test_config_update() {
        let _test_dir = setup_test_dir();
        
        // Initial config with initial key
        let mut config = Config {
            openai_api_key: Secret::new(SecretString(String::new())),
        };
        config.set_openai_api_key("initial_key");
        config.save().unwrap();
        
        // Load and verify initial
        let loaded_initial = Config::load().unwrap();
        assert_eq!(
            loaded_initial.openai_api_key.expose_secret().0,
            "initial_key"
        );
        
        // Update key
        let mut config = Config::load().unwrap();
        config.set_openai_api_key("updated_key");
        config.save().unwrap();
        
        // Load and verify updated
        let loaded_updated = Config::load().unwrap();
        assert_eq!(
            loaded_updated.openai_api_key.expose_secret().0,
            "updated_key"
        );
    }
    
    #[test]
    fn test_config_file_permissions() {
        let _test_dir = setup_test_dir();
        
        // Create and save an empty config
        let config = Config {
            openai_api_key: Secret::new(SecretString(String::new())),
        };
        config.save().unwrap();
        
        // Get config path and check permissions
        let config_path = Config::config_path().unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&config_path).unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }
    }
    
    #[test]
    fn test_empty_config_creation() {
        let _test_dir = setup_test_dir();
        
        let config = Config {
            openai_api_key: Secret::new(SecretString(String::new())),
        };
        assert!(config.openai_api_key.expose_secret().0.is_empty());
    }
    
    #[test]
    fn test_secret_not_in_debug() {
        let secret = SecretString("sensitive_data".to_string());
        let debug_str = format!("{:?}", secret);
        assert!(!debug_str.contains("sensitive_data"));
    }
    
    #[test]
    fn test_config_not_in_debug() {
        let mut config = Config {
            openai_api_key: Secret::new(SecretString(String::new())),
        };
        config.set_openai_api_key("sensitive_key");
        let debug_str = format!("{:?}", config);
        assert!(!debug_str.contains("sensitive_key"));
    }
    
    #[test]
    fn test_secret_string_zeroize() {
        let mut secret = SecretString("sensitive_data".to_string());
        let original_len = secret.0.len();
        secret.zeroize();
        assert!(secret.0.is_empty());
        assert_eq!(secret.0.capacity(), original_len);
    }
    
    #[test]
    fn test_secret_string_serialization() {
        let secret_val = "test-key-123".to_string();
        let config = Config {
            openai_api_key: Secret::new(SecretString(secret_val.clone())),
        };
        
        // Serialize the config
        let serialized = toml::to_string(&config).unwrap();
        
        // Check that the plain text key is not present
        assert!(!serialized.contains(&secret_val));
        
        // Deserialize the config and check that the key is properly decoded
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.openai_api_key.expose_secret().0, secret_val);
    }
} 