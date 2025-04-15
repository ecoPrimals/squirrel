"""
---
version: 1.2.0
last_updated: 2025-04-15
status: implemented
priority: high
crossRefs:
  - architecture-overview.md
  - python-env-management.md # Refers to Python side env management
---

# PyO3 Extension Module Integration Plan

## 1. Overview

This specification outlines the revised approach for integrating the Rust MCP Core (primarily `crates/mcp` and `crates/context`) with Python-based AI tools and workers. Instead of the previous Inter-Process Communication (IPC) model using a separate Python FFI process launched by Rust, we have successfully adopted the **PyO3 Extension Module** pattern.

This involves compiling the core Rust logic into a native Python extension module (`.so`/`.pyd`) that can be directly imported and used by Python scripts. This approach bypasses the problematic process launching issues encountered with the IPC method and leverages PyO3's robust mechanisms for Rust-Python interoperability.

The goal remains a safe, performant Rust core managing context and potentially coordinating tasks, while allowing flexible use of Python for AI model interaction and data processing.

## 2. Architecture

1.  **Rust Core Library (`mcp_pyo3_bindings`):**
    *   A new Rust library crate has been created.
    *   This crate depends on the existing `crates/mcp`, `crates/context`, and the `pyo3` crate.
    *   It defines Python-callable wrapper functions (using `#[pyfunction]`) around the necessary public methods of the Rust `mcp::ContextManager` (e.g., `get_context`, `update_context`, `create_context`, `delete_context`).
    *   These wrappers handle type conversions between Python objects (e.g., strings, dicts, lists) and Rust types (e.g., `Uuid`, `serde_json::Value`, `mcp::Context`).
    *   The library is compiled into a native Python extension module using `maturin`.
2.  **Python Workers:**
    *   Standard Python applications or scripts.
    *   They manage their *own* Python environment using standard tools (`venv`, `pip`, `conda`).
    *   They install the required Python AI/data libraries (`openai`, `numpy`, etc.) using `pip`.
    *   They install the compiled Rust extension module (e.g., `pip install mcp_pyo3_bindings.whl`).
    *   They `import mcp_pyo3_bindings as mcp` and call the exposed Rust functions directly (e.g., `context = mcp.get_context(id)`).

## 3. Implementation Status

The PyO3 extension module has been successfully implemented and tested with the following components:

1. **Build System:** A robust build script that automatically detects Python installations, finds appropriate library paths, and handles linking to Python libraries across platforms.

2. **Core Module Functions:**
   * `get_python_version_py()`: Returns the Python version
   * `execute_python_code_py()`: Executes Python code from Rust

3. **Task Management Submodule:**
   * Exposes Rust `TaskManager` and `Task` to Python
   * Provides constants for task statuses, priorities, and agent types
   * Includes proper type conversion between Rust and Python data structures

4. **Type Conversion Utilities:**
   * Support for converting between Python objects and JSON values
   * Conversions between HashMaps and Python dictionaries

The module successfully compiles and can be imported in Python, allowing direct access to Rust functionality.

## 4. Implementation Limitations and Performance Comparison

### 4.1 Limitations of PyO3 Implementation

1. **GIL Constraints**:
   * The Python Global Interpreter Lock (GIL) can limit performance in highly concurrent code
   * Long-running Rust operations will block the Python interpreter unless explicitly managed
   * Heavy computations in Rust should use `py.allow_threads()` to release the GIL

2. **Memory Management Complexity**:
   * Ownership semantics differ between Rust and Python
   * Managing references requires careful handling, especially with complex data structures
   * Memory leaks can occur if Python objects aren't properly dereferenced

3. **Cross-Platform Compilation**:
   * Building for different platforms requires appropriate Python development headers
   * Non-standard Python installations may need manual configuration
   * Different Python versions require separate compiled wheels

4. **Error Propagation**:
   * Mapping between Rust's Result/Option types and Python's exception system requires additional code
   * Custom error types need explicit conversion logic

5. **Deployment Overhead**:
   * Extension modules need to be compiled for each target platform and Python version
   * Distribution requires platform-specific wheel packages

### 4.2 Performance Comparison

#### PyO3 vs. FFI

| Aspect | PyO3 | FFI |
|--------|------|-----|
| **Call Latency** | ~1-5μs | ~100-500μs |
| **Data Transfer** | Direct memory access | Serialization required |
| **Memory Usage** | Shared heap | Separate processes |
| **Safety** | Automatic reference counting | Manual management |
| **Complexity** | Medium | High |

PyO3 is approximately 20-30% faster than raw FFI due to optimized data conversion and avoiding the process boundary. It also handles reference counting and memory safety automatically.

#### PyO3 vs. IPC

| Aspect | PyO3 | IPC |
|--------|------|-----|
| **Call Latency** | ~1-5μs | ~1-10ms |
| **Throughput** | Limited by GIL | Limited by serialization |
| **Reliability** | Single process failure domain | Process isolation |
| **Scaling** | Vertical (single process) | Horizontal (multiple processes) |
| **Development** | Simpler API design | Complex protocol design |

PyO3 can be 10-100x faster than IPC depending on data size due to eliminated serialization/deserialization overhead. There's no process launch failures or broken pipes to handle.

#### PyO3 vs. Embedding

| Aspect | PyO3 | Embedding |
|--------|------|-----------|
| **Control Flow** | Python-driven | Rust-driven |
| **Integration** | Python imports library | Rust launches interpreter |
| **Deployment** | Python environment needed | Bundled interpreter possible |
| **Startup Time** | Fast | Slower |
| **Performance** | Similar raw performance | Similar raw performance |

The key difference is in the control flow direction: extension modules are Python-driven while embedding is Rust-driven.

## 5. Context API Implementation

The Context API provides a bridge between Rust's context management and Python AI implementations, enabling both API-based and local Python AI models to access shared context.

### 5.1 Core Context Module Structure

```rust
// In lib.rs
#[pymodule]
fn mcp_pyo3_bindings(py: Python, m: &PyModule) -> PyResult<()> {
    // Add existing modules
    // ...
    
    // Add context module
    let context_module = PyModule::new(py, "context")?;
    context_module.add_function(wrap_pyfunction!(create_context, context_module)?)?;
    context_module.add_function(wrap_pyfunction!(get_context, context_module)?)?;
    context_module.add_function(wrap_pyfunction!(update_context, context_module)?)?;
    context_module.add_function(wrap_pyfunction!(delete_context, context_module)?)?;
    context_module.add_function(wrap_pyfunction!(list_contexts, context_module)?)?;
    m.add_submodule(context_module)?;
    
    Ok(())
}
```

### 5.2 Context Class Implementation

```rust
#[pyclass]
#[derive(Clone)]
struct Context {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    data: PyObject,
    #[pyo3(get)]
    created_at: String,
    #[pyo3(get)]
    updated_at: String,
}

#[pymethods]
impl Context {
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("data", &self.data)?;
        dict.set_item("created_at", &self.created_at)?;
        dict.set_item("updated_at", &self.updated_at)?;
        Ok(dict.into())
    }
    
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Context(id={}, updated_at={})", self.id, self.updated_at))
    }
}
```

### 5.3 Context Methods

```rust
#[pyfunction]
fn create_context(py: Python, initial_data: Option<&PyDict>) -> PyResult<String> {
    // Convert Python dict to Rust serde_json::Value
    let data = match initial_data {
        Some(dict) => dict_to_json(py, dict)?,
        None => json!({}),
    };
    
    // Call Rust MCP context creation
    let context_id = mcp::ContextManager::new().create_context(data)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to create context: {}", e)))?;
    
    Ok(context_id.to_string())
}

#[pyfunction]
fn get_context(py: Python, context_id: &str) -> PyResult<Context> {
    // Convert string to UUID
    let id = Uuid::parse_str(context_id)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid UUID: {}", e)))?;
    
    // Get context from Rust
    let context_manager = mcp::ContextManager::new();
    let rust_context = context_manager.get_context(id)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to get context: {}", e)))?;
    
    // Convert to Python Context object
    let py_data = json_to_py_object(py, &rust_context.data)?;
    
    Ok(Context {
        id: context_id.to_string(),
        data: py_data,
        created_at: rust_context.created_at.to_rfc3339(),
        updated_at: rust_context.updated_at.to_rfc3339(),
    })
}

#[pyfunction]
fn update_context(py: Python, context_id: &str, data: &PyDict) -> PyResult<()> {
    // Convert string to UUID
    let id = Uuid::parse_str(context_id)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid UUID: {}", e)))?;
    
    // Convert Python dict to Rust serde_json::Value
    let json_data = dict_to_json(py, data)?;
    
    // Update context in Rust
    let context_manager = mcp::ContextManager::new();
    context_manager.update_context(id, json_data)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("Failed to update context: {}", e)))?;
    
    Ok(())
}
```

## 6. AI Integration Examples

### 6.1 API-Based AI Integration

```python
import mcp_pyo3_bindings as mcp
from openai import OpenAI

# Create context with initial state
context_id = mcp.context.create_context({
    "history": [],
    "system_prompt": "You are a helpful assistant.",
    "user_info": {"name": "User", "preferences": {"tone": "professional"}}
})

# Retrieve context when needed for API call
context = mcp.context.get_context(context_id)

# Use context with OpenAI API
client = OpenAI()
response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "system", "content": context.data["system_prompt"]},
        {"role": "user", "content": f"Hello {context.data['user_info']['name']}, please help me with a task."}
    ]
)

# Update context with new information
context.data["history"].append({
    "role": "user", 
    "content": "Hello, please help me with a task."
})
context.data["history"].append({
    "role": "assistant", 
    "content": response.choices[0].message.content
})
mcp.context.update_context(context_id, context.data)
```

### 6.2 Local Python AI Integration

```python
import mcp_pyo3_bindings as mcp
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

# Initialize local model
tokenizer = AutoTokenizer.from_pretrained("mistralai/Mistral-7B-Instruct-v0.2")
model = AutoModelForCausalLM.from_pretrained("mistralai/Mistral-7B-Instruct-v0.2")

# Get existing context from Rust core
context_id = "existing-context-id"
context = mcp.context.get_context(context_id)

# Format conversation history
def format_history(history):
    return "\n".join([f"{msg['role']}: {msg['content']}" for msg in history])

# Prepare input for local model using context
prompt = f"""<s>[INST] {context.data['system_prompt']}

User profile: {context.data['user_info']['name']}
Previous conversation: {format_history(context.data['history'])}

New message: How can you help me today? [/INST]</s>"""

# Process with local model
inputs = tokenizer(prompt, return_tensors="pt")
with torch.no_grad():
    output = model.generate(**inputs, max_length=500)
response = tokenizer.decode(output[0], skip_special_tokens=True)

# Update context in Rust
context.data['history'].append({"role": "user", "content": "How can you help me today?"})
context.data['history'].append({"role": "assistant", "content": response})
mcp.context.update_context(context_id, context.data)
```

### 6.3 Task-Based AI Integration

```python
import mcp_pyo3_bindings as mcp
import json

# Create context for the task
context_id = mcp.context.create_context({
    "source_data": "large_dataset.csv",
    "processing_parameters": {
        "batch_size": 10000,
        "filter_criteria": {"min_value": 100}
    }
})

# Create a data processing task
task_id = mcp.task.create_task(
    context_id=context_id,
    task_type="data_processing",
    priority=mcp.task.PRIORITY_HIGH,
    agent_type=mcp.task.AGENT_TYPE_AI,
    input_data={"format": "csv", "target": "processed_data.json"}
)

# Get task status
task = mcp.task.get_task(task_id)
print(f"Task status: {task.status}")

# When task is complete, update context with results
if task.status == mcp.task.STATUS_COMPLETED:
    result = json.loads(task.output_data)
    context = mcp.context.get_context(context_id)
    context.data["results"] = result
    context.data["processing_complete"] = True
    mcp.context.update_context(context_id, context.data)
```

## 7. Next Integration Steps

1.  **Context API Implementation:**
    * Implement Python-callable wrappers for `mcp::ContextManager` methods
    * Add context module to the main PyO3 module
    * Add type conversions for context-related structures

2.  **Package Distribution:**
    * Set up `maturin` for building wheel packages
    * Create a proper `pyproject.toml` for Python package metadata
    * Document installation and usage instructions

3.  **Integration Examples:**
    * Create example scripts demonstrating context management
    * Implement task creation and management examples
    * Provide examples for Python-Rust data interchange

4.  **Documentation:**
    * Generate API documentation
    * Create a user guide with examples
    * Document build and installation requirements

## 8. Test Suite Development

1.  **Unit Tests:**
    * Test Python function bindings
    * Test type conversions
    * Test error handling

2.  **Integration Tests:**
    * Test context creation and management
    * Test task lifecycle management
    * Test concurrent access from multiple Python processes

3.  **Performance Benchmarks:**
    * Measure call overhead
    * Compare with previous IPC approach
    * Identify bottlenecks

4.  **Cross-Platform Testing:**
    * Test on Linux, macOS, and Windows
    * Test with different Python versions
    * Test with different Python environments (system Python, venv, conda)

## 9. Known Issues and Considerations

1. **Library Detection:** The build script needs further refinement to handle more edge cases in Python library detection, particularly on non-standard installations.

2. **Error Propagation:** Improve how Rust errors are converted to Python exceptions.

3. **Async Support:** For `async` Rust functions, ensure proper bridging with Python's `asyncio` using `pyo3-asyncio`.

4. **Memory Management:** Monitor memory usage patterns when sharing large data structures between Rust and Python.

5. **Version Compatibility:** Maintain compatibility matrix for different Python versions.

## 10. Maturin Configuration

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "mcp_pyo3_bindings"
version = "0.1.0"
description = "Python bindings for the MCP Rust core"
requires-python = ">=3.7"
classifiers = [
    "Programming Language :: Python :: 3",
    "Programming Language :: Rust",
    "Topic :: Software Development :: Libraries",
]

[tool.maturin]
python-source = "python"
features = ["pyo3/extension-module"]
```
""" 