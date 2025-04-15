use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use pyo3::prelude::*;
use anyhow::{Result, Context};

/// Configuration for a Python interpreter
#[derive(Debug, Clone)]
pub struct InterpreterConfig {
    /// Path to base Python executable (e.g., system Python)
    pub python_path: Option<String>,
    
    /// Directory for runtime files
    pub runtime_dir: Option<String>,
    
    /// Working directory for Python processes
    pub working_dir: Option<String>,
    
    /// Path to requirements.txt file
    pub requirements_path: Option<String>,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            python_path: None,
            runtime_dir: None,
            working_dir: None,
            requirements_path: None,
        }
    }
}

/// Creates a new Python virtual environment
pub fn create_venv(config: &InterpreterConfig) -> Result<PathBuf> {
    // Determine runtime directory
    let runtime_dir = match &config.runtime_dir {
        Some(dir) => PathBuf::from(dir),
        None => std::env::temp_dir().join("mcp_python_runtime"),
    };
    
    // Create runtime directory if it doesn't exist
    fs::create_dir_all(&runtime_dir)
        .context("Failed to create runtime directory")?;
    
    // Determine Python executable
    let python_exe = match &config.python_path {
        Some(path) => path.clone(),
        None => {
            if cfg!(windows) {
                "python".to_string()
            } else {
                "/usr/bin/python3".to_string()
            }
        }
    };
    
    // Create virtual environment
    let venv_dir = runtime_dir.join("venv");
    
    // Skip if venv already exists
    if venv_dir.exists() {
        return get_venv_python(&venv_dir);
    }
    
    // Create venv
    let status = Command::new(&python_exe)
        .arg("-m")
        .arg("venv")
        .arg(&venv_dir)
        .status()
        .context("Failed to create virtual environment")?;
    
    if !status.success() {
        anyhow::bail!("Failed to create virtual environment, exit code: {:?}", status.code());
    }
    
    // Install requirements if specified
    if let Some(requirements) = &config.requirements_path {
        let venv_python = get_venv_python(&venv_dir)?;
        
        let status = Command::new(&venv_python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("-r")
            .arg(requirements)
            .status()
            .context("Failed to install requirements")?;
        
        if !status.success() {
            anyhow::bail!("Failed to install requirements, exit code: {:?}", status.code());
        }
    }
    
    get_venv_python(&venv_dir)
}

/// Get the Python executable path from a virtual environment
fn get_venv_python(venv_dir: &Path) -> Result<PathBuf> {
    let bin_dir = if cfg!(windows) {
        venv_dir.join("Scripts")
    } else {
        venv_dir.join("bin")
    };
    
    let python_exe = if cfg!(windows) {
        bin_dir.join("python.exe")
    } else {
        bin_dir.join("python")
    };
    
    if !python_exe.exists() {
        anyhow::bail!("Python executable not found in venv: {:?}", python_exe);
    }
    
    Ok(python_exe)
}

/// Evaluate Python code in the interpreter
pub fn eval_python(code: &str) -> Result<String> {
    Python::with_gil(|py| {
        let result = py.eval(code, None, None)
            .context("Failed to evaluate Python code")?;
        
        let result_str = result.extract::<String>()
            .context("Failed to extract Python result as string")?;
        
        Ok(result_str)
    })
}

/// Execute a Python script in a separate process
pub fn run_script(script_path: &Path, args: &[&str], env: Option<&[(String, String)]>) -> Result<String> {
    // Find Python interpreter
    let python_exe = if cfg!(windows) {
        "python"
    } else {
        "/usr/bin/python3"
    };
    
    // Build command
    let mut cmd = Command::new(python_exe);
    cmd.arg(script_path);
    
    // Add arguments
    for arg in args {
        cmd.arg(arg);
    }
    
    // Add environment variables
    if let Some(env_vars) = env {
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
    }
    
    // Execute command
    let output = cmd.output()
        .context("Failed to execute Python script")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Python script execution failed: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
} 