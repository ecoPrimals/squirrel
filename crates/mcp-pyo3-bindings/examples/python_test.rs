use std::error::Error;
use pyo3::prelude::*;
use mcp_pyo3_bindings as pyo3_bindings;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize Python interpreter with our custom setup
    println!("Initializing Python interpreter...");
    pyo3_bindings::initialize()?;
    
    // Run Python code
    Python::with_gil(|py| -> PyResult<()> {
        println!("Python version info:");
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;
        let version_info = sys.getattr("version_info")?;
        
        println!("Python version: {}", version);
        println!("Python path: {:?}", sys.getattr("path")?.extract::<Vec<String>>()?);
        
        // Run a simple Python script
        println!("\nRunning Python code:");
        let result = py.eval("'Python integration ' + 'is working!'", None, None)?;
        println!("Result: {}", result);
        
        // Test importing numpy if available
        println!("\nTesting NumPy import:");
        match py.import("numpy") {
            Ok(numpy) => {
                let version = numpy.getattr("__version__")?.extract::<String>()?;
                println!("NumPy version: {}", version);
                
                let array = numpy.call_method1("array", (vec![1, 2, 3, 4, 5],))?;
                println!("NumPy array: {}", array);
            },
            Err(e) => {
                println!("NumPy not available: {}", e);
            }
        }
        
        // Test importing common data science packages
        println!("\nChecking for other common packages:");
        for &package_name in &["pandas", "matplotlib", "torch", "tensorflow"] {
            match py.import(package_name) {
                Ok(module) => {
                    if let Ok(version) = module.getattr("__version__") {
                        println!("{} version: {}", package_name, version.extract::<String>()?);
                    } else {
                        println!("{} is available but version not found", package_name);
                    }
                },
                Err(_) => {
                    println!("{} is not available", package_name);
                }
            }
        }
        
        Ok(())
    })?;
    
    println!("\nPython integration test completed successfully!");
    Ok(())
} 