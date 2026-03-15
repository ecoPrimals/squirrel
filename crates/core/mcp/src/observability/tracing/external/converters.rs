// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Format Converters
//!
//! This module provides functions to convert internal span representation
//! to external format specifications like OpenTelemetry and Zipkin.

use crate::observability::tracing::types::{Span, SpanStatus};
use super::config::ExternalTracingConfig;

/// Convert spans to OpenTelemetry format
pub fn convert_to_otlp_format(spans: &[Span], config: &ExternalTracingConfig) -> serde_json::Value {
    // Implementation note: this is a simplified version that focuses on the structure
    // For a complete implementation, you'd need to follow the OpenTelemetry OTLP format exactly
    
    let span_values: Vec<serde_json::Value> = spans.iter().map(|span| {
        let mut attributes = serde_json::Map::new();
        
        // Add standard attributes if configured
        if config.add_standard_attributes {
            attributes.insert("service.name".to_string(), serde_json::Value::String(config.service_name.clone()));
            attributes.insert("environment".to_string(), serde_json::Value::String(config.environment.clone()));
        }
        
        // Add span attributes
        for (k, v) in span.attributes() {
            attributes.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        
        // Calculate duration
        let duration_nanos = span.duration()
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        
        // Create span representation
        serde_json::json!({
            "name": span.name(),
            "trace_id": span.trace_id(),
            "span_id": span.id(),
            "parent_span_id": span.parent_id(),
            "start_time_unix_nano": 0, // Would need real timestamp conversion in production
            "end_time_unix_nano": duration_nanos,
            "attributes": attributes,
            "status": {
                "code": match span.status() {
                    SpanStatus::Success => "OK",
                    SpanStatus::Error => "ERROR",
                    SpanStatus::Running => "UNSET",
                }
            },
            "events": span.events().iter().map(|event| {
                serde_json::json!({
                    "name": event.name(),
                    "timestamp": 0, // Would need real timestamp conversion in production
                    "attributes": event.attributes()
                })
            }).collect::<Vec<_>>()
        })
    }).collect();
    
    serde_json::json!({
        "resourceSpans": [
            {
                "resource": {
                    "attributes": {
                        "service.name": config.service_name
                    }
                },
                "scopeSpans": [
                    {
                        "scope": {
                            "name": "squirrel-mcp"
                        },
                        "spans": span_values
                    }
                ]
            }
        ]
    })
}

/// Convert spans to Zipkin format
pub fn convert_to_zipkin_format(spans: &[Span], config: &ExternalTracingConfig) -> serde_json::Value {
    // Implementation note: this is a simplified version that focuses on the structure
    // For a complete implementation, you'd need to follow the Zipkin format exactly
    
    let span_values: Vec<serde_json::Value> = spans.iter().map(|span| {
        let mut tags = serde_json::Map::new();
        
        // Add standard tags if configured
        if config.add_standard_attributes {
            tags.insert("service.name".to_string(), serde_json::Value::String(config.service_name.clone()));
            tags.insert("environment".to_string(), serde_json::Value::String(config.environment.clone()));
        }
        
        // Add span attributes as tags
        for (k, v) in span.attributes() {
            tags.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        
        // Calculate duration
        let duration_micros = span.duration()
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        
        // Create Zipkin span representation
        serde_json::json!({
            "name": span.name(),
            "traceId": span.trace_id(),
            "id": span.id(),
            "parentId": span.parent_id(),
            "timestamp": 0, // Would need real timestamp conversion in production
            "duration": duration_micros,
            "localEndpoint": {
                "serviceName": config.service_name
            },
            "tags": tags,
            "annotations": span.events().iter().map(|event| {
                serde_json::json!({
                    "value": event.name(),
                    "timestamp": 0 // Would need real timestamp conversion in production
                })
            }).collect::<Vec<_>>()
        })
    }).collect();
    
    serde_json::Value::Array(span_values)
} 