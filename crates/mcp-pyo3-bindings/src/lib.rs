// Import the Python information generated at build time
include!(concat!(env!("OUT_DIR"), "/python_lib_info.rs"));

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Once;
use std::path::Path;
use std::env;
use tracing::{debug, info, warn};

pub mod task;
pub mod types;

static PYTHON_INIT: Once = Once::new();

/// Set up the Python environment paths for dynamic loading
pub fn set_python_paths() {
    static INIT: Once = Once::new();
    
    INIT.call_once(|| {
        let python_executable = get_python_executable();
        let python_sys_prefix = get_python_sys_prefix();
        let library_dirs = get_python_library_dirs();
        
        debug!("Python executable: {}", python_executable);
        debug!("Python sys.prefix: {}", python_sys_prefix);
        debug!("Python library dirs: {:?}", library_dirs);
        
        // Set PYTHONHOME to the sys.prefix directory
        env::set_var("PYTHONHOME", &python_sys_prefix);
        
        // Add Python executable directory to PATH
        if let Some(python_dir) = Path::new(&python_executable).parent() {
            let current_path = env::var("PATH").unwrap_or_default();
            let path_separator = if cfg!(windows) { ";" } else { ":" };
            let new_path = format!("{}{}{}", python_dir.display(), path_separator, current_path);
            env::set_var("PATH", new_path);
        }
        
        // Set library path environment variables
        if !library_dirs.is_empty() {
            let lib_paths = library_dirs.join(if cfg!(windows) { ";" } else { ":" });
            
            if cfg!(target_os = "macos") {
                // On macOS, set DYLD_LIBRARY_PATH
                let current_dyld_path = env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
                if !current_dyld_path.is_empty() {
                    env::set_var("DYLD_LIBRARY_PATH", format!("{}:{}", lib_paths, current_dyld_path));
                } else {
                    env::set_var("DYLD_LIBRARY_PATH", &lib_paths);
                }
            } else if cfg!(target_os = "linux") {
                // On Linux, set LD_LIBRARY_PATH
                let current_ld_path = env::var("LD_LIBRARY_PATH").unwrap_or_default();
                if !current_ld_path.is_empty() {
                    env::set_var("LD_LIBRARY_PATH", format!("{}:{}", lib_paths, current_ld_path));
                } else {
                    env::set_var("LD_LIBRARY_PATH", &lib_paths);
                }
            } else if cfg!(target_os = "windows") {
                // On Windows, append to PATH
                let current_path = env::var("PATH").unwrap_or_default();
                let new_path = format!("{};{}", lib_paths, current_path);
                env::set_var("PATH", new_path);
            }
            
            // Set PYTHONPATH if needed
            let current_python_path = env::var("PYTHONPATH").unwrap_or_default();
            if !current_python_path.is_empty() {
                env::set_var("PYTHONPATH", format!("{}:{}", python_sys_prefix, current_python_path));
            } else {
                env::set_var("PYTHONPATH", &python_sys_prefix);
            }
        }
        
        debug!("Environment variables set for Python integration");
    });
}

/// Initialize Python with the correct library paths
pub fn init_python() -> PyResult<()> {
    PYTHON_INIT.call_once(|| {
        // Set up the Python environment variables
        set_python_paths();
        
        // Log the environment variables for debugging
        debug!("PYTHONHOME: {}", env::var("PYTHONHOME").unwrap_or_default());
        debug!("PYTHONPATH: {}", env::var("PYTHONPATH").unwrap_or_default());
        debug!("LD_LIBRARY_PATH: {}", env::var("LD_LIBRARY_PATH").unwrap_or_default());
        debug!("PATH: {}", env::var("PATH").unwrap_or_default());
    });
    
    Ok(())
}

/// Main entry point for initializing the Python interpreter
/// Should be called before any Python functionality is used
pub fn initialize() -> PyResult<()> {
    // Initialize Python environment
    init_python()?;
    
    // Use Python GIL to ensure thread safety
    Python::with_gil(|py| {
        // Initialize any Python modules you need
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;
        info!("Python version: {}", version);
        
        // Add additional setup code here
        
        Ok(())
    })
}

// Example function to create a Python interpreter instance with custom paths
pub fn create_interpreter_with_env(_py: Python<'_>, python_path: Option<String>, runtime_dir: Option<String>) -> PyResult<PyObject> {
    // Use our Python path setup
    set_python_paths();
    
    // Any custom path overrides
    if let Some(path) = python_path {
        env::set_var("PYTHONPATH", path);
    }
    
    if let Some(dir) = runtime_dir {
        if Path::new(&dir).exists() {
            debug!("Setting runtime directory to: {}", dir);
            // Set runtime directory (customize as needed)
        }
    }
    
    // Create and return your interpreter object
    Python::with_gil(|py| {
        // Just return None as a placeholder
        // In reality, you would create and return your interpreter object
        Ok(py.None())
    })
}

// Example function to get Python version info
pub fn get_python_version_info() -> PyResult<String> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;
        Ok(version)
    })
}

// Example function to execute Python code
pub fn execute_python_code(code: &str) -> PyResult<PyObject> {
    init_python()?;
    
    Python::with_gil(|py| {
        // Local variables module
        let locals = PyDict::new(py);
        
        // Execute the code and return the result
        match py.eval(code, None, Some(locals)) {
            Ok(result) => Ok(result.to_object(py)),
            Err(e) => {
                warn!("Error executing Python code: {:?}", e);
                Err(e)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_python_version() {
        // Initialize Python
        init_python().expect("Failed to initialize Python");
        
        // Check that Python version info can be retrieved
        let version = get_python_version_info().expect("Failed to get Python version");
        println!("Python version: {}", version);
        assert!(!version.is_empty(), "Python version should not be empty");
    }
}

/// PyO3 module declaration for Python import
#[pymodule]
fn mcp_pyo3_bindings(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Initialize Python with proper paths
    init_python()?;
    
    // Add module functions
    m.add_function(wrap_pyfunction!(get_python_version_py, m)?)?;
    m.add_function(wrap_pyfunction!(execute_python_code_py, m)?)?;
    
    // Add the task module
    task::task(py, m)?;
    
    Ok(())
}

/// Python-facing wrapper for get_python_version_info
#[pyfunction]
fn get_python_version_py() -> PyResult<String> {
    get_python_version_info()
}

/// Python-facing wrapper for execute_python_code
#[pyfunction]
fn execute_python_code_py(code: &str) -> PyResult<PyObject> {
    execute_python_code(code)
}
