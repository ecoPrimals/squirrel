---
title: Web Interface API Specification
version: 1.0.0
date: 2025-03-21
status: draft
---

# Web Interface API Specification

## Overview

This document details the HTTP and WebSocket API endpoints exposed by the Squirrel Web Interface. It covers endpoint definitions, request/response formats, authentication requirements, and error handling.

## API Conventions

### Base URL

All HTTP endpoints are relative to the base URL:

```
https://{host}:{port}/api/v1
```

### Authentication

Most endpoints require authentication using one of the following methods:

1. **Bearer Token Authentication**
   - Include the JWT in the `Authorization` header:
   ```
   Authorization: Bearer <token>
   ```

2. **API Key Authentication** (for service-to-service communication)
   - Include the API key in the `X-API-Key` header:
   ```
   X-API-Key: <api-key>
   ```

### Request Format

- All request bodies should be in JSON format
- Content-Type should be set to `application/json`
- UTF-8 encoding is required

### Response Format

All responses follow a standard envelope format:

```json
{
  "success": true|false,
  "data": { ... } | null,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": { ... } | null
  } | null,
  "meta": {
    "requestId": "unique-request-id",
    "timestamp": "ISO-8601 timestamp"
  }
}
```

### Error Codes

Standard HTTP status codes are used:
- 200: Success
- 201: Created
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 409: Conflict
- 422: Unprocessable Entity
- 429: Too Many Requests
- 500: Internal Server Error
- 503: Service Unavailable

Application-specific error codes are included in the `error.code` field.

### Pagination

List endpoints support pagination with the following query parameters:
- `page`: Page number (1-based)
- `limit`: Items per page

Pagination metadata is included in the response:

```json
{
  "success": true,
  "data": [...],
  "meta": {
    "pagination": {
      "page": 1,
      "limit": 10,
      "totalItems": 100,
      "totalPages": 10
    }
  }
}
```

## API Endpoints

### Health & Status

#### GET /health

Check the health status of the service.

**Authentication**: None

**Query Parameters**: None

**Response**:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "1.0.0",
    "uptime": 1234567
  },
  "meta": {
    "timestamp": "2025-03-21T15:30:45Z"
  }
}
```

#### GET /status

Get detailed status of the system components.

**Authentication**: Required (Admin role)

**Query Parameters**: None

**Response**:
```json
{
  "success": true,
  "data": {
    "components": [
      {
        "name": "database",
        "status": "healthy",
        "details": {
          "connectionPool": "10/20 active"
        }
      },
      {
        "name": "mcp",
        "status": "healthy",
        "details": {
          "connectionStatus": "connected",
          "messageQueue": "12 pending"
        }
      },
      {
        "name": "command-system",
        "status": "healthy",
        "details": {
          "registeredCommands": 42,
          "activeExecutions": 3
        }
      }
    ]
  },
  "meta": {
    "timestamp": "2025-03-21T15:31:45Z"
  }
}
```

#### GET /metrics

Get performance metrics of the system.

**Authentication**: Required

**Query Parameters**:
- `from`: Start timestamp (ISO-8601)
- `to`: End timestamp (ISO-8601)
- `metrics`: Comma-separated list of metric names

**Response**:
```json
{
  "success": true,
  "data": {
    "timeframe": {
      "from": "2025-03-21T14:00:00Z",
      "to": "2025-03-21T15:00:00Z"
    },
    "metrics": {
      "request_count": [
        {
          "timestamp": "2025-03-21T14:05:00Z",
          "value": 120
        },
        {
          "timestamp": "2025-03-21T14:10:00Z",
          "value": 145
        }
      ],
      "response_time_ms": [
        {
          "timestamp": "2025-03-21T14:05:00Z",
          "value": 45.3
        },
        {
          "timestamp": "2025-03-21T14:10:00Z",
          "value": 42.1
        }
      ]
    }
  },
  "meta": {
    "timestamp": "2025-03-21T15:32:45Z"
  }
}
```

### Authentication

#### POST /auth/login

Authenticate a user and get an access token.

**Authentication**: None

**Request Body**:
```json
{
  "username": "string",
  "password": "string",
  "mfaCode": "string (optional)"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "accessToken": "JWT_TOKEN",
    "refreshToken": "REFRESH_TOKEN",
    "expiresIn": 3600,
    "user": {
      "id": "user-id",
      "username": "username",
      "email": "email@example.com",
      "roles": ["user", "admin"]
    }
  },
  "meta": {
    "timestamp": "2025-03-21T15:33:45Z"
  }
}
```

#### POST /auth/logout

End a user session.

**Authentication**: Required

**Request Body**: None

**Response**:
```json
{
  "success": true,
  "data": {
    "message": "Successfully logged out"
  },
  "meta": {
    "timestamp": "2025-03-21T15:34:45Z"
  }
}
```

#### POST /auth/refresh

Refresh an access token.

**Authentication**: None

**Request Body**:
```json
{
  "refreshToken": "REFRESH_TOKEN"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "accessToken": "NEW_JWT_TOKEN",
    "expiresIn": 3600
  },
  "meta": {
    "timestamp": "2025-03-21T15:35:45Z"
  }
}
```

#### GET /auth/me

Get information about the currently authenticated user.

**Authentication**: Required

**Query Parameters**: None

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "user-id",
    "username": "username",
    "email": "email@example.com",
    "roles": ["user", "admin"],
    "preferences": {
      "theme": "dark",
      "notifications": true
    },
    "lastLogin": "2025-03-21T10:00:00Z"
  },
  "meta": {
    "timestamp": "2025-03-21T15:36:45Z"
  }
}
```

### Job Management

#### POST /jobs

Create a new job.

**Authentication**: Required

**Request Body**:
```json
{
  "type": "repository-analysis",
  "params": {
    "repositoryUrl": "https://github.com/example/repo",
    "branch": "main",
    "analysisDepth": "deep"
  },
  "priority": "normal",
  "callbackUrl": "https://example.com/callback (optional)"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "jobId": "job-123",
    "status": "queued",
    "estimatedCompletion": "2025-03-21T16:00:00Z",
    "statusUrl": "/api/v1/jobs/job-123"
  },
  "meta": {
    "timestamp": "2025-03-21T15:37:45Z"
  }
}
```

#### GET /jobs/{id}

Get the status of a specific job.

**Authentication**: Required

**Path Parameters**:
- `id`: Job ID

**Response**:
```json
{
  "success": true,
  "data": {
    "jobId": "job-123",
    "type": "repository-analysis",
    "status": "running",
    "progress": 45,
    "createdAt": "2025-03-21T15:37:45Z",
    "startedAt": "2025-03-21T15:38:00Z",
    "estimatedCompletion": "2025-03-21T16:00:00Z",
    "result": null,
    "error": null
  },
  "meta": {
    "timestamp": "2025-03-21T15:39:45Z"
  }
}
```

#### DELETE /jobs/{id}

Cancel a running job.

**Authentication**: Required

**Path Parameters**:
- `id`: Job ID

**Response**:
```json
{
  "success": true,
  "data": {
    "jobId": "job-123",
    "status": "cancelled",
    "cancelledAt": "2025-03-21T15:40:45Z"
  },
  "meta": {
    "timestamp": "2025-03-21T15:40:45Z"
  }
}
```

#### GET /jobs

List jobs with pagination.

**Authentication**: Required

**Query Parameters**:
- `page`: Page number (default: 1)
- `limit`: Items per page (default: 10)
- `status`: Filter by status (queued, running, completed, failed, cancelled)
- `type`: Filter by job type
- `from`: Filter by creation date (start)
- `to`: Filter by creation date (end)

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "jobId": "job-123",
      "type": "repository-analysis",
      "status": "running",
      "progress": 45,
      "createdAt": "2025-03-21T15:37:45Z"
    },
    {
      "jobId": "job-124",
      "type": "code-generation",
      "status": "completed",
      "progress": 100,
      "createdAt": "2025-03-21T14:30:00Z"
    }
  ],
  "meta": {
    "pagination": {
      "page": 1,
      "limit": 10,
      "totalItems": 42,
      "totalPages": 5
    },
    "timestamp": "2025-03-21T15:41:45Z"
  }
}
```

### Command Execution

#### POST /commands

Execute a command.

**Authentication**: Required

**Request Body**:
```json
{
  "commandId": "string",
  "params": {
    "param1": "value1",
    "param2": "value2"
  },
  "context": {
    "contextId": "context-123"
  }
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "executionId": "exec-123",
    "result": {
      "status": "success",
      "data": {...}
    },
    "executionTime": 123
  },
  "meta": {
    "timestamp": "2025-03-21T15:42:45Z"
  }
}
```

#### GET /commands

List available commands.

**Authentication**: Required

**Query Parameters**:
- `category`: Filter by command category
- `search`: Search term

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "id": "analyze-repo",
      "name": "Analyze Repository",
      "description": "Analyze a Git repository",
      "category": "analysis",
      "parameters": [
        {
          "name": "repositoryUrl",
          "type": "string",
          "required": true,
          "description": "URL of the repository"
        },
        {
          "name": "branch",
          "type": "string",
          "required": false,
          "default": "main",
          "description": "Branch to analyze"
        }
      ]
    },
    {
      "id": "generate-code",
      "name": "Generate Code",
      "description": "Generate code from a specification",
      "category": "generation",
      "parameters": [...]
    }
  ],
  "meta": {
    "timestamp": "2025-03-21T15:43:45Z"
  }
}
```

#### GET /commands/{id}

Get details about a specific command.

**Authentication**: Required

**Path Parameters**:
- `id`: Command ID

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "analyze-repo",
    "name": "Analyze Repository",
    "description": "Analyze a Git repository",
    "category": "analysis",
    "parameters": [
      {
        "name": "repositoryUrl",
        "type": "string",
        "required": true,
        "description": "URL of the repository"
      },
      {
        "name": "branch",
        "type": "string",
        "required": false,
        "default": "main",
        "description": "Branch to analyze"
      }
    ],
    "examples": [
      {
        "description": "Analyze main branch",
        "request": {
          "repositoryUrl": "https://github.com/example/repo",
          "branch": "main"
        }
      },
      {
        "description": "Analyze develop branch",
        "request": {
          "repositoryUrl": "https://github.com/example/repo",
          "branch": "develop"
        }
      }
    ]
  },
  "meta": {
    "timestamp": "2025-03-21T15:44:45Z"
  }
}
```

## WebSocket API

### Connection

Connect to the WebSocket API:

```
wss://{host}:{port}/api/v1/ws
```

Authentication is required via one of two methods:
1. URL parameter: `?token=JWT_TOKEN`
2. In the initial connection message

### Message Format

All WebSocket messages use a JSON format:

```json
{
  "type": "message-type",
  "id": "unique-message-id",
  "data": { ... },
  "timestamp": "ISO-8601 timestamp"
}
```

### Message Types

#### Connection

**Client -> Server**:
```json
{
  "type": "connection",
  "id": "msg-1",
  "data": {
    "token": "JWT_TOKEN",
    "clientInfo": {
      "version": "1.0.0",
      "platform": "web"
    }
  },
  "timestamp": "2025-03-21T15:45:45Z"
}
```

**Server -> Client**:
```json
{
  "type": "connection_ack",
  "id": "server-msg-1",
  "data": {
    "status": "connected",
    "userId": "user-123",
    "sessionId": "session-456"
  },
  "timestamp": "2025-03-21T15:45:46Z"
}
```

#### Subscribe

**Client -> Server**:
```json
{
  "type": "subscribe",
  "id": "msg-2",
  "data": {
    "channel": "job-updates",
    "params": {
      "jobId": "job-123"
    }
  },
  "timestamp": "2025-03-21T15:46:45Z"
}
```

**Server -> Client**:
```json
{
  "type": "subscribe_ack",
  "id": "server-msg-2",
  "data": {
    "status": "subscribed",
    "channel": "job-updates",
    "subscriptionId": "sub-789"
  },
  "timestamp": "2025-03-21T15:46:46Z"
}
```

#### Event

**Server -> Client**:
```json
{
  "type": "event",
  "id": "server-msg-3",
  "data": {
    "channel": "job-updates",
    "event": "progress",
    "payload": {
      "jobId": "job-123",
      "progress": 50,
      "status": "running",
      "message": "Processing files"
    }
  },
  "timestamp": "2025-03-21T15:47:45Z"
}
```

#### Command

**Client -> Server**:
```json
{
  "type": "command",
  "id": "msg-3",
  "data": {
    "commandId": "analyze-repo",
    "params": {
      "repositoryUrl": "https://github.com/example/repo",
      "branch": "main"
    }
  },
  "timestamp": "2025-03-21T15:48:45Z"
}
```

**Server -> Client**:
```json
{
  "type": "command_result",
  "id": "server-msg-4",
  "data": {
    "commandId": "analyze-repo",
    "requestId": "msg-3",
    "status": "success",
    "result": {
      "repositoryInfo": { ... }
    }
  },
  "timestamp": "2025-03-21T15:48:46Z"
}
```

#### Heartbeat

**Client -> Server** (every 30 seconds):
```json
{
  "type": "ping",
  "id": "msg-4",
  "timestamp": "2025-03-21T15:49:45Z"
}
```

**Server -> Client**:
```json
{
  "type": "pong",
  "id": "server-msg-5",
  "timestamp": "2025-03-21T15:49:46Z"
}
```

## Error Handling

### HTTP Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {
      "field": "specific field in error",
      "reason": "specific reason for error"
    }
  },
  "meta": {
    "requestId": "unique-request-id",
    "timestamp": "ISO-8601 timestamp"
  }
}
```

### Common Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | INVALID_REQUEST | Request format is invalid |
| 400 | MISSING_PARAMETER | Required parameter is missing |
| 400 | INVALID_PARAMETER | Parameter value is invalid |
| 401 | UNAUTHORIZED | Authentication required |
| 401 | INVALID_TOKEN | Authentication token is invalid |
| 401 | EXPIRED_TOKEN | Authentication token has expired |
| 403 | FORBIDDEN | Insufficient permissions |
| 404 | NOT_FOUND | Resource not found |
| 409 | CONFLICT | Resource already exists or in invalid state |
| 422 | VALIDATION_ERROR | Request validation failed |
| 429 | RATE_LIMITED | Too many requests |
| 500 | INTERNAL_ERROR | Internal server error |
| 503 | SERVICE_UNAVAILABLE | Service is temporarily unavailable |

### WebSocket Error Response Format

```json
{
  "type": "error",
  "id": "server-msg-error",
  "data": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "requestId": "original-message-id",
    "details": {
      "field": "specific field in error",
      "reason": "specific reason for error"
    }
  },
  "timestamp": "ISO-8601 timestamp"
}
```

## Rate Limiting

The API implements rate limiting to protect against abuse:

- Rate limits are applied per user/API key
- HTTP responses include rate limit headers:
  ```
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 95
  X-RateLimit-Reset: 1616349585
  ```
- When rate limit is exceeded, a 429 response is returned

## Versioning

The API is versioned through the URL path:
```
/api/v1/...
```

Breaking changes will be introduced in new API versions. 