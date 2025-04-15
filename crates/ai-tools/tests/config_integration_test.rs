use std::process::Command;
use tempfile::TempDir;
use std::env;
use std::fs;
use std::path::PathBuf;
use secrecy::ExposeSecret;
use walkdir::WalkDir;

#[test]
fn test_config_cli() {
    let temp_dir = TempDir::new().unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    let config_dir = temp_dir.path().join(".config").join("squirrel").join("ai-tools");
    fs::create_dir_all(&config_dir).unwrap();

    // Set environment variable to use test config directory
    env::set_var("SQUIRREL_CONFIG_DIR", &temp_dir_path);

    // Test setting API key
    let output = Command::new(env!("CARGO_BIN_EXE_ai-config"))
        .args(&["set-key", "test-key-123"])
        .env("SQUIRREL_CONFIG_DIR", &temp_dir_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    // Verify config file exists and has correct permissions
    let config_file = config_dir.join("config.toml");
    assert!(config_file.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&config_file).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    // Verify config file doesn't contain plaintext key
    let contents = fs::read_to_string(&config_file).unwrap();
    assert!(!contents.contains("test-key-123"));

    // Test getting API key status
    let output = Command::new(env!("CARGO_BIN_EXE_ai-config"))
        .arg("status")
        .env("SQUIRREL_CONFIG_DIR", &temp_dir_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output_str = String::from_utf8(output.stdout).unwrap();
    assert!(output_str.contains("API key is configured"));
}

#[test]
fn test_config_cli_error_handling() {
    // Test with invalid key
    let output = Command::new(env!("CARGO_BIN_EXE_ai-config"))
        .args(&["set-key", ""])
        .output()
        .unwrap();
    assert!(!output.status.success());

    // Test with missing arguments
    let output = Command::new(env!("CARGO_BIN_EXE_ai-config"))
        .args(&["set-key"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_config_file_isolation() {
    let temp_dir = TempDir::new().unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy().to_string();
    env::set_var("SQUIRREL_CONFIG_DIR", &temp_dir_path);

    // Set a key
    Command::new(env!("CARGO_BIN_EXE_ai-config"))
        .args(&["set-key", "test-key-456"])
        .env("SQUIRREL_CONFIG_DIR", &temp_dir_path)
        .output()
        .unwrap();

    // Verify no other files in directory contain the key
    let walker = WalkDir::new(temp_dir.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());

    for entry in walker {
        let contents = fs::read_to_string(entry.path()).unwrap();
        if entry.path().ends_with("config.toml") {
            continue; // Skip the actual config file
        }
        assert!(!contents.contains("test-key-456"));
    }
} 