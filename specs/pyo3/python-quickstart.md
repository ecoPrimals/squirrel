---
version: 1.1.0
last_updated: 2025-04-15
status: implemented
priority: high
crossRefs:
  - pyo3-integration-plan.md
  - python-env-management.md
---

# MCP PyO3 Bindings Quick Start Guide

## Overview

This guide provides a quick introduction to using the MCP PyO3 Bindings for integrating Python code with the Rust-based MCP system. The bindings allow direct access to Rust functionality, context management, and task coordination from Python code.

## Installation

### Prerequisites

- Python 3.7 or newer
- pip (for package installation)
- For source builds: Rust compiler and a C compiler

### Install From PyPI

```bash
pip install mcp-pyo3-bindings
```

### Install From Wheel

```bash
pip install mcp_pyo3_bindings-0.1.0-cp310-none-linux_x86_64.whl
```

### Install Additional AI Dependencies

```bash
pip install openai transformers torch numpy
```

## Basic Usage

### Import the Module

```python
import mcp_pyo3_bindings as mcp
```

### Get Python Version

```python
version = mcp.get_python_version_py()
print(f"Python version: {version}")
```

### Execute Python Code

```python
result = mcp.execute_python_code_py("""
import math
result = math.sqrt(16)
print(f"The square root is {result}")
result
""")
print(f"Result: {result}")  # Result: 4.0
```

## Context Management

The Context API allows you to create, retrieve, update, and delete contexts that can be shared between Python and Rust components.

### Create a Context

```python
# Create with initial data
context_id = mcp.context.create_context({
    "user": "test_user",
    "session": "new_session",
    "metadata": {
        "source": "example_script",
        "version": "1.0"
    }
})
print(f"Created context with ID: {context_id}")
```

### Get Context

```python
# Retrieve context by ID
context = mcp.context.get_context(context_id)
print(f"User: {context.data['user']}")
print(f"Session: {context.data['session']}")
print(f"Created at: {context.created_at}")
```

### Update Context

```python
# Update context data
context.data["updated"] = True
context.data["metadata"]["last_access"] = "2025-04-15"
mcp.context.update_context(context_id, context.data)
```

### Convert to Dictionary

```python
# Convert context to dictionary
context_dict = context.to_dict()
print(context_dict)
```

### Delete Context

```python
# Delete context when no longer needed
mcp.context.delete_context(context_id)
```

### List All Contexts

```python
# List all available contexts
contexts = mcp.context.list_contexts()
for ctx_id in contexts:
    print(f"Available context: {ctx_id}")
```

## Task Management

The Task API allows you to create, track, and manage tasks within the MCP system.

### Constants

```python
# Task status constants
print(f"Available statuses: {mcp.task.STATUS_CREATED}, {mcp.task.STATUS_RUNNING}, {mcp.task.STATUS_COMPLETED}")

# Priority constants
print(f"Priority levels: {mcp.task.PRIORITY_LOW}, {mcp.task.PRIORITY_MEDIUM}, {mcp.task.PRIORITY_HIGH}")

# Agent type constants
print(f"Agent types: {mcp.task.AGENT_TYPE_HUMAN}, {mcp.task.AGENT_TYPE_AI}, {mcp.task.AGENT_TYPE_SYSTEM}")
```

### Create a Task

```python
# Create a new task
task_id = mcp.task.create_task(
    context_id=context_id,
    task_type="data_processing",
    priority=mcp.task.PRIORITY_HIGH,
    agent_type=mcp.task.AGENT_TYPE_AI,
    input_data={"source": "test_data.csv", "format": "json"}
)
print(f"Created task with ID: {task_id}")
```

### Get Task

```python
# Get task details
task = mcp.task.get_task(task_id)
print(f"Task type: {task.task_type}")
print(f"Task status: {task.status}")
print(f"Task priority: {task.priority}")
```

### Update Task

```python
# Update task status
mcp.task.update_task_status(task_id, mcp.task.STATUS_RUNNING)

# Update task output
mcp.task.update_task_output(task_id, {"processed_records": 1000, "success": True})
```

### Complete Task

```python
# Mark task as complete
mcp.task.complete_task(
    task_id=task_id,
    output_data={"processed_records": 1000, "errors": 0},
    status=mcp.task.STATUS_COMPLETED
)
```

## Integration with AI Libraries

### OpenAI Integration

```python
import mcp_pyo3_bindings as mcp
from openai import OpenAI

# Create context with conversation history
context_id = mcp.context.create_context({
    "history": [],
    "system_prompt": "You are a helpful assistant.",
    "user_info": {"name": "User", "preferences": {"tone": "professional"}}
})

# Get context
context = mcp.context.get_context(context_id)

# Set up OpenAI client
client = OpenAI()

# Create messages from context
messages = [
    {"role": "system", "content": context.data["system_prompt"]},
]

# Add any conversation history
for msg in context.data["history"]:
    messages.append({"role": msg["role"], "content": msg["content"]})

# Add new user message
user_message = f"Hello {context.data['user_info']['name']}, please help me with a task."
messages.append({"role": "user", "content": user_message})

# Call OpenAI API
response = client.chat.completions.create(
    model="gpt-4",
    messages=messages
)

# Get response content
assistant_message = response.choices[0].message.content

# Update context with new messages
context.data["history"].append({"role": "user", "content": user_message})
context.data["history"].append({"role": "assistant", "content": assistant_message})
mcp.context.update_context(context_id, context.data)

print(f"Assistant: {assistant_message}")
```

### Local Transformers Integration

```python
import mcp_pyo3_bindings as mcp
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

# Initialize local model
tokenizer = AutoTokenizer.from_pretrained("mistralai/Mistral-7B-Instruct-v0.2")
model = AutoModelForCausalLM.from_pretrained("mistralai/Mistral-7B-Instruct-v0.2")

# Get existing context from Rust core
context_id = mcp.context.create_context({
    "history": [],
    "system_prompt": "You are a helpful assistant.",
    "user_info": {"name": "User"}
})
context = mcp.context.get_context(context_id)

# Format conversation history
def format_history(history):
    if not history:
        return "No previous conversation."
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

print(f"Model response: {response}")
```

## Error Handling

```python
import mcp_pyo3_bindings as mcp

try:
    # Try to get a context that doesn't exist
    context = mcp.context.get_context("00000000-0000-0000-0000-000000000000")
except Exception as e:
    print(f"Error: {e}")
    
    # Create a new context instead
    context_id = mcp.context.create_context({"new": True})
    print(f"Created new context: {context_id}")
```

## Resource Cleanup

The PyO3 bindings automatically handle resource cleanup when Python objects go out of scope. However, for explicit cleanup:

```python
# Delete contexts when no longer needed
mcp.context.delete_context(context_id)

# For long-running applications, ensure all objects are properly dereferenced
context = None
```

## Advanced Topics

### Asynchronous Operations

PyO3 async support requires the `pyo3-asyncio` feature, which allows Python's asyncio to work with Rust futures:

```python
import asyncio
import mcp_pyo3_bindings as mcp

async def process_task():
    # Create a context
    context_id = await mcp.context.create_context_async({"async": True})
    
    # Process in the background
    task_id = await mcp.task.create_task_async(
        context_id=context_id,
        task_type="background_processing",
        priority=mcp.task.PRIORITY_MEDIUM,
        agent_type=mcp.task.AGENT_TYPE_SYSTEM,
        input_data={"background": True}
    )
    
    # Wait for completion
    while True:
        task = await mcp.task.get_task_async(task_id)
        if task.status == mcp.task.STATUS_COMPLETED:
            break
        await asyncio.sleep(0.1)
    
    return task.output_data

# Run the async function
result = asyncio.run(process_task())
print(f"Result: {result}")
```

### Custom Types

Creating custom Python wrappers for Rust types:

```python
# Create a custom task processor
processor = mcp.task.TaskProcessor(
    name="custom_processor",
    config={"threads": 4, "batch_size": 100}
)

# Process multiple tasks
result = processor.process_batch([task_id1, task_id2, task_id3])
print(f"Processed {len(result)} tasks")
```

## Performance Considerations

1. **GIL Limitations**: Be aware of Python's Global Interpreter Lock when doing computation-heavy work
2. **Memory Management**: Large data structures are most efficient when processed on one side (Rust or Python)
3. **Type Conversions**: Minimize conversions between Rust and Python types for performance-critical code
4. **Async Coordination**: Use async features for I/O bound operations to avoid blocking

<version>1.1.0</version> 