# MCP PyO3 Bindings

This crate provides Python bindings for the MCP (Machine Context Protocol) using PyO3, enabling Python code to interact with the Rust-based MCP framework.

## Features

- Python bindings for the MCP ContextManager
- Task management interfaces for Python
- Runtime Python interpreter creation and management
- Cross-platform compatibility (Linux, macOS, Windows)

## Setup and Installation

The bindings require a compatible Python installation (Python 3.7 or newer). The build system will automatically detect Python installations on your system.

### Environment Variables

You can control the Python detection and linking through these environment variables:

- `PYTHON_SYS_EXECUTABLE`: Point to a specific Python executable to use
- `PYTHONHOME`: Set the Python installation directory
- `VIRTUAL_ENV` or `CONDA_PREFIX`: Indicate a virtual environment or Conda environment

### Building on Different Platforms

#### Linux

On Linux, the build script will:
1. Search for Python installations in common locations and environment variables
2. Add the appropriate library search paths
3. Set `LD_LIBRARY_PATH` for runtime loading

```sh
# Building with the system Python
cargo build -p mcp-pyo3-bindings

# Building with a specific Python executable
PYTHON_SYS_EXECUTABLE=/path/to/python cargo build -p mcp-pyo3-bindings

# Building with a virtual environment
source /path/to/venv/bin/activate
cargo build -p mcp-pyo3-bindings
```

#### macOS

On macOS, the build process is similar to Linux but uses `DYLD_LIBRARY_PATH` for runtime loading:

```sh
# Building with the system Python 
cargo build -p mcp-pyo3-bindings

# Using a specific Python installation, like Homebrew's Python
PYTHON_SYS_EXECUTABLE=/opt/homebrew/bin/python3 cargo build -p mcp-pyo3-bindings

# Using a Conda environment
conda activate my_env
cargo build -p mcp-pyo3-bindings
```

#### Windows

On Windows, the build script will look for Python in standard locations and modify the `PATH` for runtime loading:

```powershell
# Building with system Python
cargo build -p mcp-pyo3-bindings

# Using a specific Python installation
$env:PYTHON_SYS_EXECUTABLE = "C:\Python310\python.exe"
cargo build -p mcp-pyo3-bindings

# Using a virtual environment
.\venv\Scripts\activate
cargo build -p mcp-pyo3-bindings
```

### Creating a Python Environment for Development

For consistent development, you can create a dedicated Python environment:

```sh
# Linux/macOS
python -m venv pyo3-test-env
source pyo3-test-env/bin/activate
pip install -r requirements.txt

# Windows
python -m venv pyo3-test-env
.\pyo3-test-env\Scripts\activate
pip install -r requirements.txt
```

The build script will automatically detect this environment if it exists in the project root.

## Usage

### Loading the Module in Python

Once built, you can use the bindings from Python:

```python
import mcp_pyo3_bindings as mcp

# Initialize the MCP context manager
mcp.initialize_manager_py()

# Create a context
context_id = mcp.create_context_py("test_context", {"key": "value"}, None, None)

# Get a context
context = mcp.get_context_py(context_id)

# Update a context
mcp.update_context_py(context_id, {"updated": True}, None)

# Create a task
task_manager = mcp.task.PyTaskManager()
task = task_manager.create_task("test_task", {"input": "data"})

# Execute a task
result = task.execute()
```

### Creating Python Interpreters 

You can create and manage Python virtual environments:

```python
# Create a virtual environment
venv_path = mcp.create_virtual_env_py(None, None, "requirements.txt")

# Evaluate Python code
result = mcp.eval_python_code_py("1 + 1")
```

## Troubleshooting

### Library Not Found Errors

If you encounter "library not found" errors:

1. Check if Python is correctly installed and detected:
   ```sh
   cargo clean
   RUST_LOG=debug cargo build -vv -p mcp-pyo3-bindings
   ```

2. Manually set the library path:
   - Linux: `export LD_LIBRARY_PATH=/path/to/python/lib:$LD_LIBRARY_PATH`
   - macOS: `export DYLD_LIBRARY_PATH=/path/to/python/lib:$DYLD_LIBRARY_PATH`
   - Windows: Add Python's DLL directory to the `PATH`

3. Use the rpath linker flag:
   ```sh
   RUSTFLAGS="-C link-args=-Wl,-rpath,/path/to/python/lib" cargo build -p mcp-pyo3-bindings
   ```

### Python Version Mismatches

If you have multiple Python versions installed, specify the correct one:

```sh
PYTHON_SYS_EXECUTABLE=/path/to/specific/python cargo build -p mcp-pyo3-bindings
```

### Common Issues by Platform

#### Linux
- Missing development packages: Install `python3-dev` or `python3-devel`
- SELinux restrictions: May need to adjust policies for dynamic loading

#### macOS
- System Integrity Protection: May affect library loading from non-standard locations
- Homebrew Python: Ensure it's in your `PATH`

#### Windows
- Missing Visual C++ Redistributable: Install the appropriate version
- PATH issues: Ensure Python and its DLLs are in the system PATH

## Development Notes

### Adding New Bindings

To add new bindings to MCP functionality:

1. Implement a new function in `lib.rs` or a module
2. Add `#[pyfunction]` or `#[pyclass]` attribute
3. Register the function or class in the module initialization

Example:

```rust
#[pyfunction]
fn new_function_py(py: Python<'_>, arg: PyObject) -> PyResult<&PyAny> {
    // Implementation
}

// In module initialization
m.add_function(wrap_pyfunction!(new_function_py, m)?)?;
```

### Testing

Run the tests to ensure cross-platform compatibility:

```sh
cargo test -p mcp-pyo3-bindings
```

The tests include a subprocess test that verifies Python can be correctly loaded and executed. 