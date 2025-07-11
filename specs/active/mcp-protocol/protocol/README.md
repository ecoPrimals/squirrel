---
description: Machine Context Protocol (MCP) Specification
version: 1.0.0
last_updated: 2025-03-21
status: draft
---

# Machine Context Protocol (MCP) Specification

## Overview

The Machine Context Protocol (MCP) defines the communication standard for exchanging context information between AI systems and tools. This specification outlines the protocol's message format, communication flow, security requirements, and extensibility mechanisms.

## Core Components

### 1. Message Format

MCP messages use a structured JSON format with the following components:

```json
{
  "header": {
    "version": "1.0",
    "messageId": "uuid-string",
    "timestamp": "ISO-8601-datetime",
    "messageType": "request|response|notification|error",
    "requestId": "uuid-string (for responses)",
    "source": "source-identifier",
    "destination": "destination-identifier"
  },
  "payload": {
    "tool": {
      "id": "tool-identifier",
      "capability": "capability-name",
      "parameters": {
        // Tool-specific parameters
      }
    },
    "context": {
      // Context information
    },
    "security": {
      "token": "security-token",
      "permissions": ["read", "execute", "etc"]
    }
  },
  "metadata": {
    // Additional information
  }
}
```

### 2. Communication Flow

The protocol defines several types of communication flows:

1. **Request-Response**: Client sends a request, server responds
2. **Notification**: One-way messages that don't require a response
3. **Streaming**: Continuous data transmission 
4. **Broadcast**: Messages sent to multiple receivers

### 3. Security

- Token-based authentication
- Permission-based authorization
- Tool validation
- Message integrity verification
- Secure transport (TLS)

### 4. Error Handling

Error responses follow a standardized format:

```json
{
  "header": {
    "messageType": "error",
    "requestId": "original-request-id",
    "timestamp": "ISO-8601-datetime"
  },
  "error": {
    "code": "error-code",
    "message": "Human-readable message",
    "details": {
      // Additional error information
    }
  }
}
```

## Implementation Guidelines

### Tool Registration

Tools must be registered with the protocol handler before use, providing:
- Unique identifier
- Capabilities list
- Parameter schema
- Security requirements

### Context Management

Context information should be:
- Clearly structured
- Version-controlled
- Properly scoped
- Efficiently serializable

### Performance Considerations

- Message size limitations (recommend < 1MB)
- Rate limiting considerations
- Timeout handling
- Backpressure mechanisms

## Extension Mechanisms

The protocol can be extended through:
- Protocol version upgrades
- Custom message types
- Extended context formats
- Tool-specific extensions

## Compatibility

This protocol is designed to be:
- Language-agnostic
- Platform-independent
- Backwards compatible within major versions
- Extensible for future requirements

## Related Specifications

- [Tool Manager Specification](../tool-manager.md)
- [Security Manager Specification](../security-manager.md)
- [State Manager Specification](../state-manager.md)

<version>1.0.0</version> 