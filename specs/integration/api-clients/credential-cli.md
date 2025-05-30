---
version: 1.0.0
last_updated: 2024-03-21
status: draft
priority: high
crossRefs:
  - 1001-rust-safety.mdc
  - 1008-rust-error-handling.mdc
  - 006-cli-standards.mdc
---

# Credential Management CLI Specification

## Overview
This specification details the command-line interface for securely managing API keys, tokens, and other credentials used by the Squirrel API Client module. It provides a user-friendly way to add, update, list, and remove credentials without exposing sensitive information in plain text.

## Architecture

### Component Structure
```rust
crates/api_client/src/credential_cli/
├── main.rs          # CLI entry point
├── commands/        # CLI command implementations
│   ├── add.rs       # Add credential command
│   ├── list.rs      # List credentials command
│   ├── remove.rs    # Remove credential command
│   ├── update.rs    # Update credential command
│   ├── export.rs    # Export credentials command
│   ├── import.rs    # Import credentials command
│   └── mod.rs       # Commands entry point
├── input/           # Secure input handling
│   ├── prompt.rs    # Interactive prompts
│   ├── mask.rs      # Input masking
│   └── mod.rs       # Input entry point
├── output/          # Secure output handling
│   ├── table.rs     # Tabular output
│   ├── redact.rs    # Output redaction
│   └── mod.rs       # Output entry point
├── crypto/          # Cryptographic operations
│   ├── key.rs       # Master key management
│   ├── encrypt.rs   # Encryption utilities
│   └── mod.rs       # Crypto entry point
└── config.rs        # CLI configuration
```

## Implementation Details

### CLI Command Structure
```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "credman")]
#[command(about = "Secure credential management for Squirrel API Client")]
pub struct CredentialCliArgs {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Storage type (encrypted, environment, memory)
    #[arg(long)]
    pub storage: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new credential
    Add {
        /// Service name (e.g., github, openai)
        #[arg(required = true)]
        service: String,
        
        /// Credential type (apikey, oauth2, basic)
        #[arg(long, short)]
        type_: Option<String>,
        
        /// Non-interactive mode (read from stdin)
        #[arg(long)]
        non_interactive: bool,
    },
    
    /// List available credentials
    List {
        /// Service name filter
        #[arg(required = false)]
        service: Option<String>,
        
        /// Show only service names
        #[arg(long)]
        names_only: bool,
        
        /// Show credential details (redacted by default)
        #[arg(long)]
        show_details: bool,
    },
    
    /// Update an existing credential
    Update {
        /// Service name
        #[arg(required = true)]
        service: String,
        
        /// Non-interactive mode (read from stdin)
        #[arg(long)]
        non_interactive: bool,
    },
    
    /// Remove a credential
    Remove {
        /// Service name
        #[arg(required = true)]
        service: String,
        
        /// Force removal without confirmation
        #[arg(long, short)]
        force: bool,
    },
    
    /// Export credentials (encrypted)
    Export {
        /// Output file path
        #[arg(short, long, required = true)]
        output: PathBuf,
        
        /// Services to export (all if not specified)
        #[arg(short, long)]
        services: Option<Vec<String>>,
        
        /// Export format (json, yaml)
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Import credentials
    Import {
        /// Input file path
        #[arg(short, long, required = true)]
        input: PathBuf,
        
        /// Merge strategy (replace, skip, prompt)
        #[arg(long, default_value = "prompt")]
        merge: String,
    },
    
    /// Rotate encryption key
    RotateKey {
        /// Force rotation without backup
        #[arg(long)]
        force: bool,
    },
}
```

### Secure Input Handling
```rust
pub struct SecurePrompt {
    mask_char: Option<char>,
    allow_empty: bool,
    confirm: bool,
}

impl SecurePrompt {
    pub fn new() -> Self;
    pub fn with_mask(mut self, mask_char: char) -> Self;
    pub fn allow_empty(mut self, allow: bool) -> Self;
    pub fn with_confirmation(mut self, confirm: bool) -> Self;
    
    pub fn prompt_password(&self, prompt: &str) -> Result<SecretString, InputError>;
    pub fn prompt_api_key(&self, prompt: &str) -> Result<SecretString, InputError>;
    pub fn prompt_secret(&self, prompt: &str) -> Result<SecretString, InputError>;
}
```

### Credential Storage Interface
```rust
pub struct CliCredentialStorage {
    storage: Arc<dyn CredentialStorage>,
    config: CliConfig,
}

impl CliCredentialStorage {
    pub async fn new(config: CliConfig) -> Result<Self, CliError>;
    
    pub async fn get_credentials(&self, service: &str) -> Result<Credentials, CliError>;
    pub async fn store_credentials(&self, service: &str, credentials: Credentials) -> Result<(), CliError>;
    pub async fn delete_credentials(&self, service: &str) -> Result<(), CliError>;
    pub async fn list_services(&self) -> Result<Vec<String>, CliError>;
    pub async fn export_credentials(&self, services: Option<&[String]>) -> Result<CredentialExport, CliError>;
    pub async fn import_credentials(&self, export: CredentialExport, merge_strategy: MergeStrategy) -> Result<ImportResult, CliError>;
}
```

### Security Features
```rust
pub struct MasterKeyManager {
    key_path: PathBuf,
}

impl MasterKeyManager {
    pub fn new(config: &CliConfig) -> Result<Self, CliError>;
    
    pub fn get_master_key(&self) -> Result<SecretKey, CliError>;
    pub fn rotate_master_key(&self, backup: bool) -> Result<(), CliError>;
    pub fn generate_master_key(&self) -> Result<SecretKey, CliError>;
}

pub struct CredentialEncryption {
    master_key: SecretKey,
}

impl CredentialEncryption {
    pub fn new(master_key: SecretKey) -> Self;
    
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CliError>;
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CliError>;
}
```

## Command Workflows

### Add Credential Workflow
1. Parse command-line arguments
2. Determine credential type from arguments or prompt user
3. Collect credential details:
   - For API keys: prompt for key with masked input
   - For OAuth2: prompt for client ID, client secret
   - For Basic: prompt for username, password
4. Encrypt and store credentials
5. Display success message

### List Credentials Workflow
1. Parse command-line arguments
2. Retrieve credentials from storage
3. Apply service name filter if provided
4. Redact sensitive values in output
5. Display formatted credential list:
   - Service names only, or
   - Service names with credential types, or
   - Service names with masked credential details

### Update Credential Workflow
1. Parse command-line arguments
2. Check if credential exists
3. Retrieve existing credential
4. Prompt for updated values
5. Encrypt and store updated credential
6. Display success message

### Remove Credential Workflow
1. Parse command-line arguments
2. Check if credential exists
3. Prompt for confirmation (unless --force)
4. Remove credential from storage
5. Display success message

## Security Requirements

### Input Protection
1. Mask sensitive input with asterisks or other mask character
2. Support terminal echo disabling for password input
3. Clear input buffers after processing
4. Support non-interactive mode for scripting

### Output Protection
1. Redact sensitive information in output
2. Provide clear feedback without exposing secrets
3. Support different verbosity levels
4. Enable safe logging of operations

### Storage Security
1. Use secure storage backed by auth-management module
2. Support master key rotation
3. Implement secure memory handling
4. Encrypt exported credentials

## Error Handling
```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Input error: {0}")]
    InputError(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Command error: {0}")]
    CommandError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Testing Requirements

### Unit Tests
1. Test input masking and validation
2. Test credential encryption/decryption
3. Test command parsing and execution
4. Test output formatting and redaction
5. Test error handling

### Integration Tests
1. Test end-to-end credential management
2. Test master key rotation
3. Test import/export functionality
4. Test different storage backends
5. Test command-line interface

### Security Tests
1. Test protection against memory dumping
2. Test credential file security
3. Test input sanitation
4. Test error message information leakage
5. Test master key handling

## Metrics

### Usage Metrics
1. Command success/failure rate
2. Command execution time
3. Number of credentials managed
4. Storage type usage
5. Import/export usage

### Security Metrics
1. Key rotation frequency
2. Storage access patterns
3. Error frequency
4. Command usage patterns
5. Non-interactive usage frequency

## Implementation Steps

### Phase 1: Core Functionality
1. Implement command structure
2. Add secure input handling
3. Implement credential storage interface
4. Set up basic command handlers
5. Add help and documentation

### Phase 2: Enhanced Security
1. Implement master key management
2. Add credential encryption
3. Implement secure memory handling
4. Add input/output protection
5. Enhance error handling

### Phase 3: Advanced Features
1. Implement import/export functionality
2. Add key rotation
3. Support different storage backends
4. Add batch operations
5. Improve testing and security

## Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }
thiserror = "1.0"
secrecy = "0.8"
zeroize = "1.5"
ring = "0.16"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rpassword = "7.0"
tracing = "0.1"
console = "0.15"
dialoguer = "0.10"
directories = "4.0"
```

## Configuration
```toml
[credential_cli]
storage_type = "encrypted"
storage_path = "~/.config/squirrel/credentials"
master_key_path = "~/.config/squirrel/master.key"
timeout_seconds = 60
history_file_enabled = false

[credential_cli.display]
mask_char = "*"
table_style = "psql"
redact_values = true
```

## Usage Examples

### Adding a GitHub API Token
```
$ credman add github --type apikey
Enter API Key for service 'github': ********************
Credential for 'github' added successfully.
```

### Listing Available Credentials
```
$ credman list
SERVICE    TYPE      ADDED                   EXPIRES
github     ApiKey    2024-03-15 14:30:22     Never
openai     ApiKey    2024-03-20 09:15:45     Never
```

### Updating a Credential
```
$ credman update openai
Current credential type: ApiKey
Enter new API Key for service 'openai': ********************
Credential for 'openai' updated successfully.
```

### Exporting Credentials
```
$ credman export --output credentials.json
Exporting credentials for 2 services.
Credentials exported to credentials.json (encrypted)
```

## Notes
- All sensitive input must be masked or hidden
- Master key should be stored securely
- Support integration with system keyrings when available
- Implement timeouts for sensitive data in memory
- Enable scripting for automated credential management
- Provide clear documentation on security practices 