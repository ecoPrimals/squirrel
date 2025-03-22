---
title: "Galaxy API to MCP Mapping Specification"
description: "Detailed mapping between Galaxy API endpoints and MCP functionality"
version: "0.1.0"
last_updated: "2025-03-25"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "mcp-team"]
  reviewers: ["core-team", "integration-team"]
---

# Galaxy API to MCP Mapping

## 1. Overview

This specification defines the mapping between Galaxy API endpoints and Machine Context Protocol (MCP) functionality. It provides detailed information on how Galaxy's RESTful API is translated into MCP tool capabilities, messages, and contexts.

## 2. API Endpoint Mapping

### 2.1 Authentication and Session Management

| MCP Capability | Galaxy API Endpoint | HTTP Method | Parameters | Return Value |
|----------------|---------------------|-------------|------------|--------------|
| `authenticate` | `/api/authenticate/baseauth` | GET | username, password | API key |
| `validate_key` | `/api/users/current` | GET | key | User info |
| `create_session` | `/api/users/{user_id}/api_key` | POST | user_id | New API key |
| `logout` | `/api/logout` | GET | key | Success status |

### 2.2 Tool Management

| MCP Capability | Galaxy API Endpoint | HTTP Method | Parameters | Return Value |
|----------------|---------------------|-------------|------------|--------------|
| `list_tools` | `/api/tools` | GET | key | Tool list |
| `get_tool_details` | `/api/tools/{tool_id}` | GET | key, tool_id | Tool details |
| `search_tools` | `/api/tools?q={query}` | GET | key, query | Filtered tool list |
| `get_tool_versions` | `/api/tools/{tool_id}/versions` | GET | key, tool_id | Version list |
| `get_tool_help` | `/api/tools/{tool_id}/help` | GET | key, tool_id | Tool help text |

### 2.3 Data Management

| MCP Capability | Galaxy API Endpoint | HTTP Method | Parameters | Return Value |
|----------------|---------------------|-------------|------------|--------------|
| `list_histories` | `/api/histories` | GET | key | History list |
| `create_history` | `/api/histories` | POST | key, name | New history |
| `get_history_details` | `/api/histories/{history_id}` | GET | key, history_id | History details |
| `upload_dataset` | `/api/tools/fetch` | POST | key, history_id, url | Dataset info |
| `get_dataset` | `/api/datasets/{dataset_id}` | GET | key, dataset_id | Dataset content |
| `delete_dataset` | `/api/datasets/{dataset_id}` | DELETE | key, dataset_id | Success status |

### 2.4 Tool Execution

| MCP Capability | Galaxy API Endpoint | HTTP Method | Parameters | Return Value |
|----------------|---------------------|-------------|------------|--------------|
| `execute_tool` | `/api/tools` | POST | key, tool_id, history_id, inputs | Job info |
| `get_job_status` | `/api/jobs/{job_id}` | GET | key, job_id | Job status |
| `cancel_job` | `/api/jobs/{job_id}` | DELETE | key, job_id | Success status |
| `rerun_job` | `/api/jobs/{job_id}/rerun` | POST | key, job_id | New job info |

### 2.5 Workflow Management

| MCP Capability | Galaxy API Endpoint | HTTP Method | Parameters | Return Value |
|----------------|---------------------|-------------|------------|--------------|
| `list_workflows` | `/api/workflows` | GET | key | Workflow list |
| `get_workflow_details` | `/api/workflows/{workflow_id}` | GET | key, workflow_id | Workflow details |
| `create_workflow` | `/api/workflows` | POST | key, workflow_data | New workflow |
| `execute_workflow` | `/api/workflows/{workflow_id}/invocations` | POST | key, workflow_id, inputs | Invocation info |
| `get_workflow_invocation` | `/api/workflows/{workflow_id}/invocations/{invocation_id}` | GET | key, workflow_id, invocation_id | Invocation status |

## 3. Request/Response Transformations

### 3.1 Galaxy API Request → MCP Message

```json
// Galaxy API Request (execute_tool)
{
  "tool_id": "fastqc",
  "history_id": "f2db41e1fa331b3e",
  "inputs": {
    "input_file": {"src": "hda", "id": "12345"}
  }
}

// Transformed to MCP Message
{
  "header": {
    "messageId": "uuid-string",
    "timestamp": "2025-03-25T12:34:56Z",
    "messageType": "request",
    "source": "mcp-client",
    "destination": "galaxy-adapter"
  },
  "payload": {
    "tool": {
      "id": "galaxy-tool-fastqc",
      "capability": "execute",
      "parameters": {
        "inputs": {
          "input_file": "12345"
        },
        "history_id": "f2db41e1fa331b3e"
      }
    },
    "context": {
      "session_id": "session-uuid",
      "request_id": "request-uuid"
    },
    "security": {
      "token": "galaxy-api-key"
    }
  }
}
```

### 3.2 Galaxy API Response → MCP Message

```json
// Galaxy API Response (execute_tool)
{
  "outputs": [
    {
      "id": "67890",
      "name": "FastQC HTML report",
      "history_id": "f2db41e1fa331b3e",
      "visible": true,
      "type": "file",
      "url": "/datasets/67890/display"
    },
    {
      "id": "67891",
      "name": "FastQC Text report",
      "history_id": "f2db41e1fa331b3e",
      "visible": true,
      "type": "file",
      "url": "/datasets/67891/display"
    }
  ],
  "jobs": [
    {
      "id": "abcdef1234567890",
      "tool_id": "fastqc",
      "state": "ok",
      "exit_code": 0,
      "create_time": "2025-03-25T12:34:56Z",
      "update_time": "2025-03-25T12:45:06Z"
    }
  ],
  "implicit_collections": []
}

// Transformed to MCP Message
{
  "header": {
    "messageId": "response-uuid",
    "timestamp": "2025-03-25T12:45:10Z",
    "messageType": "response",
    "requestId": "uuid-string",
    "source": "galaxy-adapter",
    "destination": "mcp-client"
  },
  "payload": {
    "tool": {
      "id": "galaxy-tool-fastqc",
      "capability": "execute"
    },
    "execution": {
      "status": "success",
      "result": {
        "outputs": {
          "html_file": {
            "data_id": "67890",
            "name": "FastQC HTML report",
            "url": "/datasets/67890/display",
            "format": "html"
          },
          "text_file": {
            "data_id": "67891",
            "name": "FastQC Text report",
            "url": "/datasets/67891/display",
            "format": "txt"
          }
        },
        "job_info": {
          "job_id": "abcdef1234567890",
          "state": "ok",
          "exit_code": 0,
          "start_time": "2025-03-25T12:34:56Z",
          "end_time": "2025-03-25T12:45:06Z"
        }
      },
      "execution_time_ms": 611000
    }
  }
}
```

## 4. Error Handling and Status Codes

### 4.1 HTTP Status Code Mapping

| Galaxy HTTP Status | MCP Error Type | Description |
|--------------------|----------------|-------------|
| 400 (Bad Request) | `ValidationError` | Invalid parameters or input data |
| 401 (Unauthorized) | `AuthenticationError` | Invalid or missing API key |
| 403 (Forbidden) | `AuthorizationError` | Insufficient permissions |
| 404 (Not Found) | `NotFoundError` | Resource not found (tool, dataset, etc.) |
| 408 (Request Timeout) | `TimeoutError` | Request processing timeout |
| 500 (Server Error) | `SystemError` | Server-side error in Galaxy |
| 503 (Service Unavailable) | `ServiceUnavailableError` | Galaxy service unavailable |

### 4.2 Galaxy Error Response → MCP Error Message

```json
// Galaxy API Error Response
{
  "err_msg": "Tool with id 'invalid_tool' not found",
  "err_code": 404
}

// Transformed to MCP Error Message
{
  "header": {
    "messageId": "error-uuid",
    "timestamp": "2025-03-25T12:34:56Z",
    "messageType": "error",
    "requestId": "original-request-uuid",
    "source": "galaxy-adapter",
    "destination": "mcp-client"
  },
  "payload": {
    "error": {
      "code": "TOOL_NOT_FOUND",
      "message": "Tool with id 'invalid_tool' not found",
      "details": {
        "tool_id": "invalid_tool",
        "galaxy_error_code": 404,
        "suggestion": "Check the tool ID or query available tools with list_tools capability"
      }
    }
  }
}
```

## 5. Context Management

The Galaxy MCP Adapter maintains the following context information:

### 5.1 Session Context

```json
{
  "session": {
    "galaxy_api_key": "your-api-key",
    "galaxy_user_id": "user-id",
    "galaxy_base_url": "https://usegalaxy.org",
    "session_start_time": "2025-03-25T12:00:00Z",
    "default_history_id": "default-history-id"
  }
}
```

### 5.2 Tool Execution Context

```json
{
  "execution": {
    "tool_id": "galaxy-tool-id",
    "job_id": "galaxy-job-id",
    "history_id": "history-id",
    "input_datasets": ["dataset-id-1", "dataset-id-2"],
    "output_datasets": ["output-id-1", "output-id-2"],
    "execution_start_time": "2025-03-25T12:34:56Z",
    "status_check_interval_ms": 5000
  }
}
```

### 5.3 Workflow Context

```json
{
  "workflow": {
    "workflow_id": "workflow-id",
    "invocation_id": "invocation-id",
    "history_id": "history-id",
    "steps": [
      {
        "step_id": "step-1",
        "tool_id": "tool-id-1",
        "status": "complete"
      },
      {
        "step_id": "step-2",
        "tool_id": "tool-id-2",
        "status": "running"
      }
    ],
    "workflow_start_time": "2025-03-25T12:34:56Z"
  }
}
```

## 6. Implementation Requirements

### 6.1 API Client Requirements

1. **Error Handling**: Implement robust error handling for all Galaxy API requests
2. **Rate Limiting**: Respect Galaxy API rate limits (backoff exponentially on 429 responses)
3. **Connection Management**: Maintain persistent connections where appropriate
4. **Response Parsing**: Handle different response formats (JSON, binary data for datasets)
5. **Authentication**: Support API key rotation and session management

### 6.2 Message Transformation Requirements

1. **Schema Validation**: Validate all messages against MCP and Galaxy schemas
2. **Type Conversion**: Handle data type conversions between systems
3. **Context Awareness**: Use context information to enrich messages
4. **Stateful Operations**: Maintain state for long-running operations
5. **Error Transformation**: Transform Galaxy errors to MCP error format

## 7. Performance Considerations

1. **Caching**:
   - Cache tool definitions and metadata (invalidate after 24 hours)
   - Cache dataset metadata (invalidate on job completion)
   - Do not cache authentication tokens or sensitive data

2. **Batching**:
   - Batch job status checks when possible
   - Combine multiple dataset operations where appropriate

3. **Asynchronous Operations**:
   - Use Galaxy's async APIs where available
   - Implement polling strategies for long-running jobs
   - Provide webhooks for job completion notifications

## 8. Related Specifications

- [Galaxy API Documentation](https://docs.galaxyproject.org/en/master/api_doc.html)
- [MCP Protocol Specification](../mcp/protocol.md)
- [MCP Tool Definition Specification](../mcp/protocol/tool-definition.md)
- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)

<version>0.1.0</version> 