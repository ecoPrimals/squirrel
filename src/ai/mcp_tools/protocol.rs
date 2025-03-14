use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use flate2::Compression;
use flate2::write::{GzEncoder, GzDecoder};
use zstd::{encode_all, decode_all};
use lz4::{EncoderBuilder, Decoder};
use crate::ai::mcp_tools::{
    context::MachineContext,
    types::{MCPError, MCPCommand, MCPResponse, CompressionFormat, EncryptionFormat, MessageMetadata},
    security::{SecurityManager, AuthToken},
};

const CURRENT_PROTOCOL_VERSION: &str = "1.0.0";

/// Protocol implementation for MCP
pub struct MCPProtocol {
    context: Arc<Mutex<MachineContext>>,
    security: Arc<Mutex<SecurityManager>>,
    input: Box<dyn io::Read + Send>,
    output: Box<dyn io::Write + Send>,
    test_mode: bool,
    token: Option<AuthToken>,
    compression_format: CompressionFormat,
    encryption_format: EncryptionFormat,
}

impl MCPProtocol {
    /// Create a new protocol instance
    pub fn new(
        context: Arc<Mutex<MachineContext>>,
        security: Arc<Mutex<SecurityManager>>,
        input: Box<dyn io::Read + Send>,
        output: Box<dyn io::Write + Send>,
        test_mode: bool,
    ) -> Self {
        Self {
            context,
            security,
            input,
            output,
            test_mode,
            token: None,
            compression_format: CompressionFormat::None,
            encryption_format: EncryptionFormat::None,
        }
    }

    /// Set the compression format
    pub fn set_compression(&mut self, format: CompressionFormat) {
        self.compression_format = format;
    }

    /// Set the encryption format
    pub fn set_encryption(&mut self, format: EncryptionFormat) {
        self.encryption_format = format;
    }

    /// Run a single command and return the response
    pub fn run_single(&mut self) -> Result<MCPResponse, MCPError> {
        let mut buf = String::new();
        self.input.read_to_string(&mut buf)?;

        // Decrypt if needed
        let decrypted_data = match self.encryption_format {
            EncryptionFormat::None => buf.into_bytes(),
            EncryptionFormat::ChaCha20Poly1305 => {
                let security = self.security.lock().unwrap();
                security.decrypt(buf.as_bytes())?
            }
            EncryptionFormat::Aes256Gcm => {
                // TODO: Implement AES-256-GCM decryption
                return Err(MCPError::EncryptionError("AES-256-GCM not implemented".to_string()));
            }
        };

        // Decompress if needed
        let decompressed_data = match self.compression_format {
            CompressionFormat::None => decrypted_data,
            CompressionFormat::Gzip => {
                let mut decoder = GzDecoder::new(Vec::new());
                decoder.write_all(&decrypted_data)?;
                decoder.finish()?
            }
            CompressionFormat::Zstd => {
                decode_all(&decrypted_data[..])
                    .map_err(|e| MCPError::CompressionError(e.to_string()))?
            }
            CompressionFormat::Lz4 => {
                let mut decoder = Decoder::new(&decrypted_data[..])
                    .map_err(|e| MCPError::CompressionError(e.to_string()))?;
                let mut output = Vec::new();
                io::copy(&mut decoder, &mut output)?;
                output
            }
        };

        let command: MCPCommand = serde_json::from_slice(&decompressed_data)?;
        let mut response = self.handle_command(command)?;

        // Add metadata to response
        response.metadata = Some(MessageMetadata {
            compression: self.compression_format,
            encryption: self.encryption_format,
            version: CURRENT_PROTOCOL_VERSION.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        let response_data = serde_json::to_vec(&response)?;

        // Compress if needed
        let compressed_data = match self.compression_format {
            CompressionFormat::None => response_data,
            CompressionFormat::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&response_data)?;
                encoder.finish()?
            }
            CompressionFormat::Zstd => {
                encode_all(&response_data[..], 0)
                    .map_err(|e| MCPError::CompressionError(e.to_string()))?
            }
            CompressionFormat::Lz4 => {
                let mut output = Vec::new();
                let mut encoder = EncoderBuilder::new()
                    .level(4)
                    .build(&mut output)
                    .map_err(|e| MCPError::CompressionError(e.to_string()))?;
                encoder.write_all(&response_data)?;
                encoder.finish().1?;
                output
            }
        };

        // Encrypt if needed
        let encrypted_data = match self.encryption_format {
            EncryptionFormat::None => compressed_data,
            EncryptionFormat::ChaCha20Poly1305 => {
                let security = self.security.lock().unwrap();
                security.encrypt(&compressed_data)?
            }
            EncryptionFormat::Aes256Gcm => {
                // TODO: Implement AES-256-GCM encryption
                return Err(MCPError::EncryptionError("AES-256-GCM not implemented".to_string()));
            }
        };

        self.output.write_all(&encrypted_data)?;
        self.output.flush()?;

        Ok(response)
    }

    /// Run the protocol loop
    pub fn run(&mut self) -> Result<(), MCPError> {
        loop {
            let response = self.run_single()?;
            
            if self.test_mode {
                break;
            }

            if response.success && response.data.is_none() {
                break;
            }
        }
        Ok(())
    }

    /// Handle a command
    fn handle_command(&mut self, command: MCPCommand) -> Result<MCPResponse, MCPError> {
        // Check if the command requires authentication
        if self.requires_authentication(&command) && self.token.is_none() {
            return Err(MCPError::AuthenticationError("Authentication required".to_string()));
        }

        // Check if the user has permission to execute the command
        if let Some(token) = &self.token {
            let security = self.security.lock().unwrap();
            if !security.check_permission(&token.token, &command.name)? {
                return Err(MCPError::AuthorizationError("Permission denied".to_string()));
            }
        }

        match command.name.as_str() {
            // Authentication commands
            "authenticate" => self.handle_authenticate(&command),
            "logout" => self.handle_logout(),

            // Context commands
            "get_context" => self.handle_get_context(),
            "list_commands" => self.handle_list_commands(),
            "get_command" => self.handle_get_command(&command),
            "get_env_var" => self.handle_get_env_var(&command),
            "get_working_dir" => self.handle_get_working_dir(),
            "get_system_info" => self.handle_get_system_info(),

            // File operations
            "read_file" => self.handle_read_file(&command),
            "write_file" => self.handle_write_file(&command),
            "delete_file" => self.handle_delete_file(&command),

            // Directory operations
            "list_dir" => self.handle_list_dir(&command),
            "create_dir" => self.handle_create_dir(&command),
            "delete_dir" => self.handle_delete_dir(&command),

            // Process operations
            "run_process" => self.handle_run_process(&command),
            "stop_process" => self.handle_stop_process(&command),
            "process_status" => self.handle_process_status(&command),

            // Git operations
            "git_status" => self.handle_git_status(),
            "git_commit" => self.handle_git_commit(&command),
            "git_push" => self.handle_git_push(),
            "git_pull" => self.handle_git_pull(),

            _ => Err(MCPError::CommandNotFound(command.name)),
        }
    }

    /// Handle the authenticate command
    fn handle_authenticate(&mut self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        if command.args.len() != 2 {
            return Err(MCPError::InvalidArguments("User ID and password required".to_string()));
        }

        let user_id = &command.args[0];
        let password = &command.args[1];

        let security = self.security.lock().unwrap();
        let token = security.authenticate(user_id, password)?;
        self.token = Some(token.clone());

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "token": token.token,
                "expires_at": token.expires_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })),
            error: None,
        })
    }

    /// Handle the logout command
    fn handle_logout(&mut self) -> Result<MCPResponse, MCPError> {
        if let Some(token) = &self.token {
            let security = self.security.lock().unwrap();
            if !security.validate_token(&token.token)? {
                return Err(MCPError::TokenError("Invalid token".to_string()));
            }
            self.token = None;
        }

        Ok(MCPResponse {
            success: true,
            data: None,
            error: None,
        })
    }

    /// Handle the get_context command
    fn handle_get_context(&self) -> Result<MCPResponse, MCPError> {
        let context = self.context.lock().unwrap();
        let context_data = json!({
            "working_dir": context.working_dir().to_string_lossy(),
            "system_info": {
                "os_type": context.system_info().os_type,
                "os_version": context.system_info().os_version,
                "cpu_count": context.system_info().cpu_count,
                "memory_total": context.system_info().memory_total,
            }
        });

        Ok(MCPResponse {
            success: true,
            data: Some(context_data),
            error: None,
        })
    }

    /// Handle the list_commands command
    fn handle_list_commands(&self) -> Result<MCPResponse, MCPError> {
        let context = self.context.lock().unwrap();
        let commands: Vec<Value> = context.list_commands()
            .iter()
            .map(|cmd| json!({
                "name": cmd.name,
                "description": cmd.description,
                "args": cmd.args,
            }))
            .collect();

        Ok(MCPResponse {
            success: true,
            data: Some(json!({ "commands": commands })),
            error: None,
        })
    }

    /// Handle the get_command command
    fn handle_get_command(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let cmd_name = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Command name required".to_string()))?;

        let context = self.context.lock().unwrap();
        let cmd_info = context.get_command(cmd_name)
            .ok_or_else(|| MCPError::CommandNotFound(cmd_name.clone()))?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "name": cmd_info.name,
                "description": cmd_info.description,
                "args": cmd_info.args,
            })),
            error: None,
        })
    }

    /// Handle the get_env_var command
    fn handle_get_env_var(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let var_name = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Environment variable name required".to_string()))?;

        let context = self.context.lock().unwrap();
        let value = context.get_env_var(var_name)
            .ok_or_else(|| MCPError::InvalidArguments(format!("Environment variable not found: {}", var_name)))?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "name": var_name,
                "value": value,
            })),
            error: None,
        })
    }

    /// Handle the get_working_dir command
    fn handle_get_working_dir(&self) -> Result<MCPResponse, MCPError> {
        let context = self.context.lock().unwrap();
        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "working_dir": context.working_dir().to_string_lossy(),
            })),
            error: None,
        })
    }

    /// Handle the get_system_info command
    fn handle_get_system_info(&self) -> Result<MCPResponse, MCPError> {
        let context = self.context.lock().unwrap();
        let info = context.system_info();
        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "os_type": info.os_type,
                "os_version": info.os_version,
                "cpu_count": info.cpu_count,
                "memory_total": info.memory_total,
            })),
            error: None,
        })
    }

    /// Handle the read_file command
    fn handle_read_file(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let path = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("File path required".to_string()))?;

        let content = std::fs::read_to_string(path)
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "path": path,
                "content": content,
            })),
            error: None,
        })
    }

    /// Handle the write_file command
    fn handle_write_file(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        if command.args.len() < 2 {
            return Err(MCPError::InvalidArguments("File path and content required".to_string()));
        }

        let path = &command.args[0];
        let content = &command.args[1];

        std::fs::write(path, content)
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "path": path,
                "status": "written",
            })),
            error: None,
        })
    }

    /// Handle the delete_file command
    fn handle_delete_file(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let path = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("File path required".to_string()))?;

        std::fs::remove_file(path)
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "path": path,
                "status": "deleted",
            })),
            error: None,
        })
    }

    /// Handle the list_dir command
    fn handle_list_dir(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let context = self.context.lock().unwrap();
        let default_path = context.working_dir().to_string_lossy().to_string();
        let path = command.args.first()
            .unwrap_or(&default_path);

        let entries = std::fs::read_dir(path)
            .map_err(MCPError::IoError)?
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let path = entry.path();
                json!({
                    "name": path.file_name().unwrap_or_default().to_string_lossy(),
                    "path": path.to_string_lossy(),
                    "is_dir": path.is_dir(),
                    "is_file": path.is_file(),
                })
            })
            .collect::<Vec<Value>>();

        Ok(MCPResponse {
            success: true,
            data: Some(json!(entries)),
            error: None,
        })
    }

    /// Handle the create_dir command
    fn handle_create_dir(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let path = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Directory path required".to_string()))?;

        std::fs::create_dir_all(path)
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "path": path,
                "status": "created",
            })),
            error: None,
        })
    }

    /// Handle the delete_dir command
    fn handle_delete_dir(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let path = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Directory path required".to_string()))?;

        std::fs::remove_dir_all(path)
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: true,
            data: Some(json!({
                "path": path,
                "status": "deleted",
            })),
            error: None,
        })
    }

    /// Handle the run_process command
    fn handle_run_process(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        if command.args.is_empty() {
            return Err(MCPError::InvalidArguments("Command and arguments required".to_string()));
        }

        let output = Command::new(&command.args[0])
            .args(&command.args[1..])
            .output()
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: output.status.success(),
            data: Some(json!({
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
                "status": output.status.code(),
            })),
            error: if output.status.success() { None } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Handle the stop_process command
    fn handle_stop_process(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let pid = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Process ID required".to_string()))?
            .parse::<u32>()
            .map_err(|_| MCPError::InvalidArguments("Invalid process ID".to_string()))?;

        #[cfg(target_os = "windows")]
        let output = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()
            .map_err(MCPError::IoError)?;

        #[cfg(not(target_os = "windows"))]
        let output = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: output.status.success(),
            data: Some(json!({
                "pid": pid,
                "status": output.status.code(),
            })),
            error: if output.status.success() { None } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Handle the process_status command
    fn handle_process_status(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let pid = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Process ID required".to_string()))?
            .parse::<u32>()
            .map_err(|_| MCPError::InvalidArguments("Invalid process ID".to_string()))?;

        #[cfg(target_os = "windows")]
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map_err(MCPError::IoError)?;

        #[cfg(not(target_os = "windows"))]
        let output = Command::new("ps")
            .args(["-p", &pid.to_string()])
            .output()
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: output.status.success(),
            data: Some(json!({
                "pid": pid,
                "status": output.status.code(),
                "output": String::from_utf8_lossy(&output.stdout),
            })),
            error: if output.status.success() { None } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Handle the git_status command
    fn handle_git_status(&self) -> Result<MCPResponse, MCPError> {
        let output = Command::new("git")
            .arg("status")
            .output()
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: output.status.success(),
            data: Some(json!({
                "status": output.status.code(),
                "output": String::from_utf8_lossy(&output.stdout),
            })),
            error: if output.status.success() { None } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Handle the git_commit command
    fn handle_git_commit(&self, command: &MCPCommand) -> Result<MCPResponse, MCPError> {
        let message = command.args.first()
            .ok_or_else(|| MCPError::InvalidArguments("Commit message required".to_string()))?;

        let output = Command::new("git")
            .args(["commit", "-m", message])
            .output()
            .map_err(MCPError::IoError)?;

        // If commit was successful, attempt to push
        if output.status.success() {
            let push_output = Command::new("git")
                .arg("push")
                .output()
                .map_err(MCPError::IoError)?;

            // Return combined result
            Ok(MCPResponse {
                success: push_output.status.success(),
                data: Some(json!({
                    "commit": {
                        "status": output.status.code(),
                        "output": String::from_utf8_lossy(&output.stdout),
                    },
                    "push": {
                        "status": push_output.status.code(),
                        "output": String::from_utf8_lossy(&push_output.stdout),
                    }
                })),
                error: if push_output.status.success() { None } else {
                    Some(String::from_utf8_lossy(&push_output.stderr).to_string())
                },
            })
        } else {
            // Return commit error
            Ok(MCPResponse {
                success: false,
                data: Some(json!({
                    "commit": {
                        "status": output.status.code(),
                        "output": String::from_utf8_lossy(&output.stdout),
                    }
                })),
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            })
        }
    }

    /// Handle the git_push command
    fn handle_git_push(&self) -> Result<MCPResponse, MCPError> {
        let output = Command::new("git")
            .arg("push")
            .output()
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: output.status.success(),
            data: Some(json!({
                "status": output.status.code(),
                "output": String::from_utf8_lossy(&output.stdout),
            })),
            error: if output.status.success() { None } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Handle the git_pull command
    fn handle_git_pull(&self) -> Result<MCPResponse, MCPError> {
        let output = Command::new("git")
            .arg("pull")
            .output()
            .map_err(MCPError::IoError)?;

        Ok(MCPResponse {
            success: output.status.success(),
            data: Some(json!({
                "status": output.status.code(),
                "output": String::from_utf8_lossy(&output.stdout),
            })),
            error: if output.status.success() { None } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
        })
    }

    /// Check if a command requires authentication
    fn requires_authentication(&self, command: &MCPCommand) -> bool {
        !matches!(command.name.as_str(), "authenticate" | "get_system_info")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::collections::HashMap;

    fn create_test_protocol(input: String) -> MCPProtocol {
        let context = Arc::new(Mutex::new(MachineContext::new().unwrap()));
        let security = Arc::new(Mutex::new(SecurityManager::new().unwrap()));
        
        // Grant test permissions
        let security_guard = security.lock().unwrap();
        let mut role_permissions = HashMap::new();
        role_permissions.insert("admin".to_string(), vec![
            "read_file".to_string(),
            "write_file".to_string(),
            "delete_file".to_string(),
            "create_dir".to_string(),
            "list_dir".to_string(),
            "delete_dir".to_string(),
            "run_process".to_string(),
            "get_env_var".to_string(),
            "set_env_var".to_string(),
            "get_working_dir".to_string(),
            "set_working_dir".to_string(),
            "get_system_info".to_string(),
            "get_context".to_string(),
        ]);

        security_guard.grant_permissions(
            "test_user",
            vec!["admin".to_string()],
            role_permissions,
        ).unwrap();
        let token = security_guard.authenticate("test_user", "test_password").unwrap();
        drop(security_guard);

        let input = Box::new(Cursor::new(input));
        let output = Box::new(Vec::new());

        let mut protocol = MCPProtocol::new(context, security, input, output, true);
        protocol.token = Some(token);
        protocol
    }

    #[test]
    fn test_get_context() {
        let mut protocol = create_test_protocol(
            r#"{"name": "get_context", "args": [], "metadata": null}"#.to_string()
        );
        let response = protocol.run_single().unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_compression_gzip() {
        let mut protocol = create_test_protocol(String::new());
        protocol.set_compression(CompressionFormat::Gzip);

        let command = MCPCommand {
            name: "test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            metadata: None,
        };

        let data = serde_json::to_vec(&command).unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(&compressed).unwrap();
        let decompressed = decoder.finish().unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_zstd() {
        let mut protocol = create_test_protocol(String::new());
        protocol.set_compression(CompressionFormat::Zstd);

        let command = MCPCommand {
            name: "test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            metadata: None,
        };

        let data = serde_json::to_vec(&command).unwrap();
        let compressed = encode_all(&data[..], 0).unwrap();
        let decompressed = decode_all(&compressed[..]).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_compression_lz4() {
        let mut protocol = create_test_protocol(String::new());
        protocol.set_compression(CompressionFormat::Lz4);

        let command = MCPCommand {
            name: "test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            metadata: None,
        };

        let data = serde_json::to_vec(&command).unwrap();
        let mut output = Vec::new();
        let mut encoder = EncoderBuilder::new()
            .level(4)
            .build(&mut output)
            .unwrap();
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().1.unwrap();

        let mut decoder = Decoder::new(&compressed[..]).unwrap();
        let mut decompressed = Vec::new();
        io::copy(&mut decoder, &mut decompressed).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_encryption() {
        let mut protocol = create_test_protocol(String::new());
        protocol.set_encryption(EncryptionFormat::ChaCha20Poly1305);

        let command = MCPCommand {
            name: "test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            metadata: None,
        };

        let data = serde_json::to_vec(&command).unwrap();
        let security = protocol.security.lock().unwrap();
        let encrypted = security.encrypt(&data).unwrap();
        let decrypted = security.decrypt(&encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_compression_and_encryption() {
        let mut protocol = create_test_protocol(String::new());
        protocol.set_compression(CompressionFormat::Gzip);
        protocol.set_encryption(EncryptionFormat::ChaCha20Poly1305);

        let command = MCPCommand {
            name: "test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            metadata: None,
        };

        let data = serde_json::to_vec(&command).unwrap();

        // Compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&data).unwrap();
        let compressed = encoder.finish().unwrap();

        // Encrypt
        let security = protocol.security.lock().unwrap();
        let encrypted = security.encrypt(&compressed).unwrap();

        // Decrypt
        let decrypted = security.decrypt(&encrypted).unwrap();

        // Decompress
        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(&decrypted).unwrap();
        let decompressed = decoder.finish().unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_protocol_version() {
        let mut protocol = create_test_protocol(String::new());
        let command = MCPCommand {
            name: "test".to_string(),
            args: vec![],
            metadata: Some(MessageMetadata {
                compression: CompressionFormat::None,
                encryption: EncryptionFormat::None,
                version: CURRENT_PROTOCOL_VERSION.to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }),
        };

        let data = serde_json::to_vec(&command).unwrap();
        let input = String::from_utf8(data).unwrap();
        let mut protocol = create_test_protocol(input);
        let response = protocol.run_single().unwrap();

        assert!(response.metadata.is_some());
        let metadata = response.metadata.unwrap();
        assert_eq!(metadata.version, CURRENT_PROTOCOL_VERSION);
    }
} 