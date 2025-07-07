---
description: Tool Definition Specification for Machine Context Protocol
version: 1.0.0
last_updated: 2025-03-21
status: draft
---

# MCP Tool Definition Specification

## Overview

This specification defines the standard format for tool definitions in the Machine Context Protocol (MCP). Tools are the fundamental units of functionality that can be registered, discovered, and executed through the MCP.

## Tool Definition Format

A tool definition consists of the following components:

```json
{
  "id": "unique-tool-id",
  "name": "Human-Readable Tool Name",
  "version": "1.0.0",
  "description": "Comprehensive description of the tool's purpose and functionality",
  "capabilities": [
    {
      "name": "capability-name",
      "description": "Description of what this capability does",
      "parameters": [
        {
          "name": "parameter-name",
          "description": "Parameter description",
          "type": "string|number|boolean|object|array|any",
          "required": true|false
        }
      ],
      "return": {
        "description": "Description of the return value",
        "schema": {
          // JSON Schema for the return value
        }
      }
    }
  ],
  "securityLevel": 0-10,
  "metadata": {
    // Additional tool-specific metadata
  }
}
```

## Core Components

### 1. Tool Identity

- **id**: Unique identifier for the tool (required)
- **name**: Human-readable name (required)
- **version**: Semantic version string (required)
- **description**: Comprehensive description (required)

### 2. Capabilities

Each tool must define one or more capabilities:

- **name**: Unique capability identifier (required)
- **description**: What the capability does (required)
- **parameters**: Input parameters the capability accepts (optional)
- **return**: Description and schema of the return value (optional)

### 3. Security

- **securityLevel**: Integer from 0-10 indicating required security level
  - 0: No security requirements
  - 1-3: Basic security (authentication)
  - 4-7: Medium security (authentication + authorization)
  - 8-10: High security (authentication + authorization + additional verification)

## Parameter Types

The following parameter types are supported:

| Type | Description | Example |
|------|-------------|---------|
| string | Text value | `"example"` |
| number | Numeric value | `42`, `3.14` |
| boolean | True/false value | `true`, `false` |
| object | JSON object | `{"key": "value"}` |
| array | JSON array | `[1, 2, 3]` |
| any | Any valid JSON value | _Any value_ |

## Implementation Guidelines

### Tool Registration

When registering a tool:

1. The tool definition must be validated against this schema
2. Tool IDs must be unique within the system
3. Capability names must be unique within a tool
4. Security levels must be honored during execution

### Versioning

Tool definitions follow semantic versioning:

- MAJOR: Breaking changes to capabilities
- MINOR: New capabilities added
- PATCH: Bug fixes, documentation updates

### Best Practices

- Keep capability names clear and descriptive
- Document all parameters thoroughly
- Use specific parameter types when possible
- Include examples in descriptions
- Keep security level appropriate to functionality

## Example Tool Definition

```json
{
  "id": "calculator",
  "name": "Calculator Tool",
  "version": "1.0.0",
  "description": "Performs basic arithmetic operations",
  "capabilities": [
    {
      "name": "add",
      "description": "Adds two numbers",
      "parameters": [
        {
          "name": "a",
          "description": "First number",
          "type": "number",
          "required": true
        },
        {
          "name": "b",
          "description": "Second number",
          "type": "number",
          "required": true
        }
      ],
      "return": {
        "description": "The sum of the two numbers",
        "schema": {
          "type": "object",
          "properties": {
            "result": {
              "type": "number"
            }
          }
        }
      }
    },
    {
      "name": "subtract",
      "description": "Subtracts the second number from the first",
      "parameters": [
        {
          "name": "a",
          "description": "First number",
          "type": "number",
          "required": true
        },
        {
          "name": "b",
          "description": "Second number",
          "type": "number",
          "required": true
        }
      ],
      "return": {
        "description": "The result of a - b",
        "schema": {
          "type": "object",
          "properties": {
            "result": {
              "type": "number"
            }
          }
        }
      }
    }
  ],
  "securityLevel": 1,
  "metadata": {
    "category": "utility",
    "tags": ["math", "calculation"]
  }
}
```

## Related Specifications

- [Protocol Specification](./README.md)
- [Tool Execution Specification](./tool-execution.md)
- [Tool Manager Specification](../tool-manager.md)

<version>1.0.0</version> 