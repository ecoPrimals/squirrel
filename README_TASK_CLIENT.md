# MCP Task Client

This package provides Python and Rust clients for interacting with the MCP Task Service.

## Overview

The MCP Task Service is a gRPC-based service for managing tasks in the Squirrel system. It allows you to:

- Create tasks with input data, metadata, and prerequisites
- Assign tasks to agents
- Report progress on tasks
- Complete tasks with output data
- Cancel tasks
- List and filter tasks

## Python Client

### Prerequisites

- Python 3.6+
- gRPC Python tools

Install the required packages:

```bash
./install_python_deps.sh
```

### Usage

The Python client is a simple script that demonstrates how to use the task service. Run it:

```bash
./task_client_minimal.py
```

This will:
1. Connect to the task service
2. Create a new task
3. Assign it to an agent
4. Report progress
5. Complete the task
6. List tasks in the context

### API Reference

The `TaskClient` class provides the following methods:

- `connect()`: Connect to the task service
- `create_task()`: Create a new task
- `get_task()`: Get a task by ID
- `assign_task()`: Assign a task to an agent
- `report_progress()`: Report progress on a task
- `complete_task()`: Complete a task with results
- `cancel_task()`: Cancel a task
- `list_tasks()`: List tasks matching filter criteria
- `close()`: Close the connection

## Starting the Task Server

Before using the client, make sure the task server is running:

```bash
./start_task_server.sh
```

This will build and start the task server on `[::1]:50052`.

## Task Data Structure

Tasks have the following structure:

- `id`: Unique task identifier
- `name`: Human-readable name
- `description`: Task description
- `status`: Current status
- `priority`: Task priority
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp
- `started_at`: When the task started processing
- `completed_at`: When the task completed or failed
- `agent_id`: ID of assigned agent
- `agent_type`: Type of assigned agent
- `input_data`: Input data for the task (JSON)
- `output_data`: Output data from the task (JSON)
- `metadata`: Metadata about the task (JSON)
- `error_message`: Error message if the task failed
- `prerequisite_task_ids`: Tasks that must complete before this one
- `dependent_task_ids`: Tasks that depend on this one
- `progress_percent`: 0-100 percent complete
- `progress_message`: Human-readable progress message
- `context_id`: Associated context ID

## Status Values

- `CREATED = 1`: Task created but not assigned
- `ASSIGNED = 2`: Task assigned to an agent
- `RUNNING = 3`: Task is currently being processed
- `COMPLETED = 4`: Task completed successfully
- `FAILED = 5`: Task failed to complete
- `CANCELLED = 6`: Task was cancelled
- `PENDING = 7`: Task is waiting for prerequisites

## Priority Values

- `LOW = 1`: Low priority
- `MEDIUM = 2`: Medium priority
- `HIGH = 3`: High priority
- `CRITICAL = 4`: Critical priority

## Agent Types

- `LOCAL_PYTHON = 1`: Local Python process
- `REMOTE_API = 2`: Remote API service
- `UI = 3`: UI component
- `SYSTEM = 4`: System agent
- `CUSTOM = 5`: Custom agent type

## Next Steps

1. Add more robust error handling
2. Implement authentication
3. Add more examples
4. Create a stable Rust client library 