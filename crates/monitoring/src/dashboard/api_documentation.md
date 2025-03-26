# Dashboard API Documentation

## Overview

This document describes the REST API endpoints available in the monitoring dashboard system. These endpoints provide access to dashboard configuration, component data, health status, and metrics.

## Base URL

All API endpoints are relative to the dashboard server base URL:

```
http://{host}:{port}/
```

For secure connections:

```
https://{host}:{port}/
```

## Authentication

For secured endpoints, authentication is performed using a Bearer token in the Authorization header:

```
Authorization: Bearer <token>
```

## API Endpoints

### Health Check

#### GET /health

Check the health status of the dashboard server.

**Authentication Required:** No

**Request:** None

**Response:**

```json
{
  "status": "healthy",
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Server is healthy
- 503: Server is unhealthy

### Server Status

#### GET /status

Get the current status of the dashboard server, including active connections.

**Authentication Required:** No

**Request:** None

**Response:**

```json
{
  "status": "running",
  "clients": 5,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful

### Components

#### GET /api/components

Get a list of all available dashboard components.

**Authentication Required:** Yes

**Request:** None

**Response:**

```json
{
  "components": [
    {
      "id": "system_cpu",
      "name": "CPU Usage",
      "type": "system",
      "description": "System CPU usage metrics",
      "refresh_interval": 5,
      "data_retention": 3600
    },
    {
      "id": "system_memory",
      "name": "Memory Usage",
      "type": "system",
      "description": "System memory usage metrics",
      "refresh_interval": 5,
      "data_retention": 3600
    }
  ],
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden

#### GET /api/components/{componentId}

Get details for a specific component.

**Authentication Required:** Yes

**Parameters:**
- `componentId`: The ID of the component

**Response:**

```json
{
  "id": "system_cpu",
  "name": "CPU Usage",
  "type": "system",
  "description": "System CPU usage metrics",
  "refresh_interval": 5,
  "data_retention": 3600,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden
- 404: Component not found

#### GET /api/components/{componentId}/data

Get the current data for a specific component.

**Authentication Required:** Yes

**Parameters:**
- `componentId`: The ID of the component

**Response:**

```json
{
  "id": "system_cpu",
  "data": {
    "usage": 45.2,
    "cores": [32.1, 56.7, 48.3, 43.9]
  },
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden
- 404: Component not found

#### GET /api/components/{componentId}/history

Get historical data for a specific component.

**Authentication Required:** Yes

**Parameters:**
- `componentId`: The ID of the component
- `start`: Start timestamp (milliseconds since epoch)
- `end`: End timestamp (milliseconds since epoch)
- `resolution`: Resolution in seconds (default: 60)

**Response:**

```json
{
  "id": "system_cpu",
  "history": [
    {
      "timestamp": 1653472092000,
      "data": {
        "usage": 42.1,
        "cores": [30.5, 52.3, 45.1, 40.5]
      }
    },
    {
      "timestamp": 1653472152000,
      "data": {
        "usage": 43.5,
        "cores": [31.2, 54.1, 46.8, 42.0]
      }
    },
    {
      "timestamp": 1653472212000,
      "data": {
        "usage": 44.8,
        "cores": [32.0, 55.3, 47.5, 43.2]
      }
    }
  ],
  "resolution": 60,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 400: Bad request (invalid parameters)
- 401: Unauthorized
- 403: Forbidden
- 404: Component not found

### Dashboard Layout

#### GET /api/dashboard/layouts

Get all available dashboard layouts.

**Authentication Required:** Yes

**Request:** None

**Response:**

```json
{
  "layouts": [
    {
      "id": "default",
      "name": "Default Layout",
      "description": "Default system monitoring layout",
      "created_at": 1653472092000,
      "updated_at": 1653472092000
    },
    {
      "id": "network",
      "name": "Network Monitoring",
      "description": "Layout focused on network metrics",
      "created_at": 1653472092000,
      "updated_at": 1653472092000
    }
  ],
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden

#### GET /api/dashboard/layouts/{layoutId}

Get a specific dashboard layout.

**Authentication Required:** Yes

**Parameters:**
- `layoutId`: The ID of the layout

**Response:**

```json
{
  "id": "default",
  "name": "Default Layout",
  "description": "Default system monitoring layout",
  "panels": [
    {
      "id": "panel1",
      "component_id": "system_cpu",
      "title": "CPU Usage",
      "type": "line_chart",
      "position": {
        "x": 0,
        "y": 0
      },
      "size": {
        "width": 6,
        "height": 4
      },
      "settings": {
        "show_legend": true,
        "y_axis_label": "Usage (%)",
        "y_axis_min": 0,
        "y_axis_max": 100
      }
    },
    {
      "id": "panel2",
      "component_id": "system_memory",
      "title": "Memory Usage",
      "type": "bar_chart",
      "position": {
        "x": 6,
        "y": 0
      },
      "size": {
        "width": 6,
        "height": 4
      },
      "settings": {
        "show_legend": true,
        "y_axis_label": "Usage (GB)",
        "y_axis_min": 0
      }
    }
  ],
  "created_at": 1653472092000,
  "updated_at": 1653472092000,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden
- 404: Layout not found

#### POST /api/dashboard/layouts

Create a new dashboard layout.

**Authentication Required:** Yes

**Request:**

```json
{
  "name": "Custom Layout",
  "description": "Custom monitoring layout",
  "panels": [
    {
      "component_id": "system_cpu",
      "title": "CPU Usage",
      "type": "line_chart",
      "position": {
        "x": 0,
        "y": 0
      },
      "size": {
        "width": 6,
        "height": 4
      },
      "settings": {
        "show_legend": true,
        "y_axis_label": "Usage (%)",
        "y_axis_min": 0,
        "y_axis_max": 100
      }
    }
  ]
}
```

**Response:**

```json
{
  "id": "custom-layout-12345",
  "name": "Custom Layout",
  "description": "Custom monitoring layout",
  "panels": [
    {
      "id": "panel1",
      "component_id": "system_cpu",
      "title": "CPU Usage",
      "type": "line_chart",
      "position": {
        "x": 0,
        "y": 0
      },
      "size": {
        "width": 6,
        "height": 4
      },
      "settings": {
        "show_legend": true,
        "y_axis_label": "Usage (%)",
        "y_axis_min": 0,
        "y_axis_max": 100
      }
    }
  ],
  "created_at": 1653472392000,
  "updated_at": 1653472392000,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 201: Layout created
- 400: Bad request (invalid parameters)
- 401: Unauthorized
- 403: Forbidden

#### PUT /api/dashboard/layouts/{layoutId}

Update an existing dashboard layout.

**Authentication Required:** Yes

**Parameters:**
- `layoutId`: The ID of the layout

**Request:**

```json
{
  "name": "Updated Layout",
  "description": "Updated monitoring layout",
  "panels": [
    {
      "id": "panel1",
      "component_id": "system_cpu",
      "title": "CPU Usage",
      "type": "line_chart",
      "position": {
        "x": 0,
        "y": 0
      },
      "size": {
        "width": 6,
        "height": 4
      },
      "settings": {
        "show_legend": true,
        "y_axis_label": "Usage (%)",
        "y_axis_min": 0,
        "y_axis_max": 100
      }
    }
  ]
}
```

**Response:**

```json
{
  "id": "default",
  "name": "Updated Layout",
  "description": "Updated monitoring layout",
  "panels": [
    {
      "id": "panel1",
      "component_id": "system_cpu",
      "title": "CPU Usage",
      "type": "line_chart",
      "position": {
        "x": 0,
        "y": 0
      },
      "size": {
        "width": 6,
        "height": 4
      },
      "settings": {
        "show_legend": true,
        "y_axis_label": "Usage (%)",
        "y_axis_min": 0,
        "y_axis_max": 100
      }
    }
  ],
  "created_at": 1653472092000,
  "updated_at": 1653472392000,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Layout updated
- 400: Bad request (invalid parameters)
- 401: Unauthorized
- 403: Forbidden
- 404: Layout not found

#### DELETE /api/dashboard/layouts/{layoutId}

Delete a dashboard layout.

**Authentication Required:** Yes

**Parameters:**
- `layoutId`: The ID of the layout

**Response:** None

**Status Codes:**
- 204: Layout deleted
- 401: Unauthorized
- 403: Forbidden
- 404: Layout not found

### Alerts

#### GET /api/alerts

Get a list of all alerts.

**Authentication Required:** Yes

**Request Parameters:**
- `status` (optional): Filter by status (active, acknowledged, resolved)
- `severity` (optional): Filter by severity (critical, warning, info)
- `limit` (optional): Maximum number of alerts to return (default: 50)
- `offset` (optional): Offset for pagination (default: 0)

**Response:**

```json
{
  "alerts": [
    {
      "id": "alert-12345",
      "type": "resource",
      "severity": "critical",
      "message": "Disk usage exceeded 90%",
      "source": "system_disk",
      "timestamp": 1653472092000,
      "status": "active",
      "details": {
        "usage": 92.5,
        "threshold": 90.0,
        "device": "/dev/sda1"
      },
      "acknowledged": false
    },
    {
      "id": "alert-12346",
      "type": "performance",
      "severity": "warning",
      "message": "High CPU usage (85%)",
      "source": "system_cpu",
      "timestamp": 1653472152000,
      "status": "acknowledged",
      "details": {
        "usage": 85.2,
        "threshold": 80.0
      },
      "acknowledged": true,
      "acknowledged_by": "admin",
      "acknowledged_at": 1653472212000
    }
  ],
  "total": 2,
  "limit": 50,
  "offset": 0,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden

#### GET /api/alerts/{alertId}

Get details for a specific alert.

**Authentication Required:** Yes

**Parameters:**
- `alertId`: The ID of the alert

**Response:**

```json
{
  "id": "alert-12345",
  "type": "resource",
  "severity": "critical",
  "message": "Disk usage exceeded 90%",
  "source": "system_disk",
  "timestamp": 1653472092000,
  "status": "active",
  "details": {
    "usage": 92.5,
    "threshold": 90.0,
    "device": "/dev/sda1"
  },
  "acknowledged": false,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden
- 404: Alert not found

#### POST /api/alerts/{alertId}/acknowledge

Acknowledge an alert.

**Authentication Required:** Yes

**Parameters:**
- `alertId`: The ID of the alert

**Request:**

```json
{
  "comment": "Investigating disk usage issue"
}
```

**Response:**

```json
{
  "id": "alert-12345",
  "type": "resource",
  "severity": "critical",
  "message": "Disk usage exceeded 90%",
  "source": "system_disk",
  "timestamp": 1653472092000,
  "status": "acknowledged",
  "details": {
    "usage": 92.5,
    "threshold": 90.0,
    "device": "/dev/sda1"
  },
  "acknowledged": true,
  "acknowledged_by": "admin",
  "acknowledged_at": 1653472392000,
  "comment": "Investigating disk usage issue",
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Alert acknowledged
- 400: Bad request (invalid parameters)
- 401: Unauthorized
- 403: Forbidden
- 404: Alert not found
- 409: Alert already acknowledged

### Metrics

#### GET /api/metrics

Get a list of all available metrics.

**Authentication Required:** Yes

**Request Parameters:**
- `category` (optional): Filter by category (system, network, tool, protocol)
- `type` (optional): Filter by type (counter, gauge, histogram)

**Response:**

```json
{
  "metrics": [
    {
      "name": "system.cpu.usage",
      "description": "CPU usage percentage",
      "category": "system",
      "type": "gauge",
      "unit": "percent"
    },
    {
      "name": "system.memory.usage",
      "description": "Memory usage in bytes",
      "category": "system",
      "type": "gauge",
      "unit": "bytes"
    },
    {
      "name": "network.rx_bytes",
      "description": "Network bytes received",
      "category": "network",
      "type": "counter",
      "unit": "bytes"
    }
  ],
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden

#### GET /api/metrics/{metricName}

Get values for a specific metric.

**Authentication Required:** Yes

**Parameters:**
- `metricName`: The name of the metric
- `start` (optional): Start timestamp (milliseconds since epoch, default: 1 hour ago)
- `end` (optional): End timestamp (milliseconds since epoch, default: now)
- `resolution` (optional): Resolution in seconds (default: 60)

**Response:**

```json
{
  "name": "system.cpu.usage",
  "description": "CPU usage percentage",
  "category": "system",
  "type": "gauge",
  "unit": "percent",
  "values": [
    {
      "timestamp": 1653472092000,
      "value": 42.1
    },
    {
      "timestamp": 1653472152000,
      "value": 43.5
    },
    {
      "timestamp": 1653472212000,
      "value": 44.8
    }
  ],
  "resolution": 60,
  "timestamp": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 400: Bad request (invalid parameters)
- 401: Unauthorized
- 403: Forbidden
- 404: Metric not found

### System Health

#### GET /api/health/components

Get the health status for all monitored components.

**Authentication Required:** Yes

**Request:** None

**Response:**

```json
{
  "components": [
    {
      "id": "cpu",
      "name": "CPU",
      "status": "healthy",
      "details": {
        "usage": 45.2,
        "threshold": 80.0
      },
      "last_checked": 1653472392000
    },
    {
      "id": "memory",
      "name": "Memory",
      "status": "healthy",
      "details": {
        "usage": 65.8,
        "threshold": 90.0
      },
      "last_checked": 1653472392000
    },
    {
      "id": "disk",
      "name": "Disk",
      "status": "warning",
      "details": {
        "usage": 85.5,
        "threshold": 80.0
      },
      "last_checked": 1653472392000
    }
  ],
  "overall_status": "warning",
  "last_checked": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden

#### GET /api/health/components/{componentId}

Get the health status for a specific component.

**Authentication Required:** Yes

**Parameters:**
- `componentId`: The ID of the component

**Response:**

```json
{
  "id": "cpu",
  "name": "CPU",
  "status": "healthy",
  "details": {
    "usage": 45.2,
    "threshold": 80.0
  },
  "history": [
    {
      "timestamp": 1653472092000,
      "status": "healthy",
      "details": {
        "usage": 42.1,
        "threshold": 80.0
      }
    },
    {
      "timestamp": 1653472152000,
      "status": "healthy",
      "details": {
        "usage": 43.5,
        "threshold": 80.0
      }
    },
    {
      "timestamp": 1653472212000,
      "status": "healthy",
      "details": {
        "usage": 44.8,
        "threshold": 80.0
      }
    }
  ],
  "last_checked": 1653472392000
}
```

**Status Codes:**
- 200: Request successful
- 401: Unauthorized
- 403: Forbidden
- 404: Component not found

## Error Responses

All API endpoints follow a standard error response format:

```json
{
  "error": {
    "code": "not_found",
    "message": "Component not found: system_disk",
    "details": {
      "component_id": "system_disk"
    }
  },
  "timestamp": 1653472392000
}
```

Common error codes:
- `bad_request`: The request parameters are invalid
- `unauthorized`: Authentication is required
- `forbidden`: User does not have permission for the requested resource
- `not_found`: The requested resource was not found
- `conflict`: The request conflicts with the current state
- `internal_error`: An internal server error occurred
- `rate_limited`: Request exceeds rate limit

## Change Log

| Version | Date | Description |
|---------|------|-------------|
| 1.0.0   | 2024-05-24 | Initial API documentation | 