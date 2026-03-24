// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON schemas for PrimalPulse tools
//!
//! Defines input/output schemas for dynamic tool registration
#![allow(dead_code)] // Tool schema helpers for dynamic registration

use serde_json::{Value, json};

/// Schema for primal.analyze action
pub fn primal_analyze_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "primal_path": {
                "type": "string",
                "description": "Path to the primal directory to analyze"
            },
            "depth": {
                "type": "string",
                "enum": ["quick", "standard", "full"],
                "default": "standard",
                "description": "Analysis depth: quick (structure only), standard (+ metrics), full (+ recommendations)"
            }
        },
        "required": ["primal_path"]
    })
}

pub fn primal_analyze_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "primal_name": {"type": "string"},
            "grade": {"type": "string"},
            "architecture_pattern": {"type": "string"},
            "capabilities": {
                "type": "array",
                "items": {"type": "string"}
            },
            "dependencies": {
                "type": "array",
                "items": {"type": "string"}
            },
            "hardcoding_issues": {"type": "integer"},
            "evolution_opportunities": {
                "type": "array",
                "items": {"type": "string"}
            }
        }
    })
}

/// Schema for primal.audit_hardcoding action
pub fn primal_audit_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "primal_path": {
                "type": "string",
                "description": "Path to the primal directory to audit"
            },
            "check_types": {
                "type": "array",
                "items": {
                    "type": "string",
                    "enum": ["primal_names", "ips", "ports", "vendors"]
                },
                "default": ["primal_names", "ips", "ports", "vendors"],
                "description": "Types of hardcoding to check for"
            }
        },
        "required": ["primal_path"]
    })
}

pub fn primal_audit_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "total_violations": {"type": "integer"},
            "by_type": {
                "type": "object",
                "properties": {
                    "primal_names": {"type": "integer"},
                    "ips": {"type": "integer"},
                    "ports": {"type": "integer"},
                    "vendors": {"type": "integer"}
                }
            },
            "critical_files": {
                "type": "array",
                "items": {"type": "string"}
            },
            "suggested_fixes": {
                "type": "array",
                "items": {"type": "string"}
            },
            "grade": {"type": "string"},
            "evolution_path": {"type": "string"}
        }
    })
}

/// Schema for rootpulse.semantic_commit action
pub fn rootpulse_commit_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "diff": {
                "type": "string",
                "description": "Git diff or file changes to analyze"
            },
            "context": {
                "type": "string",
                "description": "Additional context about the changes"
            }
        },
        "required": ["diff"]
    })
}

pub fn rootpulse_commit_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "commit_message": {"type": "string"},
            "semantic_tags": {
                "type": "array",
                "items": {"type": "string"}
            },
            "attribution_weight": {"type": "number"},
            "related_primals": {
                "type": "array",
                "items": {"type": "string"}
            },
            "estimated_impact": {"type": "string"}
        }
    })
}

/// Input schema for `neural.graph_optimize`
pub fn neural_graph_optimize_input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "graph_description": {
                "type": "string",
                "description": "Description of the coordination graph (e.g., 'songbird -> toadstool -> squirrel -> nestgate')."
            },
            "purpose": {
                "type": "string",
                "description": "Purpose of this coordination (e.g., 'AI-powered data analysis pipeline')."
            },
            "expected_latency_ms": {
                "type": "integer",
                "description": "Expected latency budget in milliseconds."
            },
            "cost_budget_usd": {
                "type": "number",
                "description": "Cost budget in USD."
            },
            "constraints": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Constraints for optimization (e.g., 'prefer_local', 'minimize_latency').",
                "default": []
            }
        },
        "required": ["graph_description", "purpose"]
    })
}

/// Output schema for `neural.graph_optimize`
pub fn neural_graph_optimize_output_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "analysis": {
                "type": "object",
                "properties": {
                    "depth": {"type": "integer"},
                    "width": {"type": "integer"},
                    "estimated_latency_ms": {"type": "integer"},
                    "estimated_cost_usd": {"type": "number"},
                    "bottlenecks": {
                        "type": "array",
                        "items": {"type": "string"}
                    },
                    "inefficiencies": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                }
            },
            "recommendations": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": {"type": "string"},
                        "description": {"type": "string"},
                        "expected_improvement": {"type": "object"},
                        "confidence": {"type": "number"}
                    }
                }
            },
            "optimized_graph": {
                "type": "string",
                "description": "Suggested optimized graph structure."
            },
            "cost_usd": {"type": "number"},
            "latency_ms": {"type": "integer"}
        },
        "required": ["analysis", "recommendations"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_object_schema(val: &Value) {
        assert_eq!(val.get("type").and_then(|v| v.as_str()), Some("object"));
        assert!(val.get("properties").is_some());
    }

    #[test]
    fn test_primal_analyze_input_schema() {
        let schema = primal_analyze_input_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("primal_path"));
        assert!(props.contains_key("depth"));
        let required = schema["required"].as_array().expect("required");
        assert!(required.contains(&json!("primal_path")));
    }

    #[test]
    fn test_primal_analyze_output_schema() {
        let schema = primal_analyze_output_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("primal_name"));
        assert!(props.contains_key("grade"));
        assert!(props.contains_key("capabilities"));
        assert!(props.contains_key("hardcoding_issues"));
    }

    #[test]
    fn test_primal_audit_input_schema() {
        let schema = primal_audit_input_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("primal_path"));
        assert!(props.contains_key("check_types"));
        let required = schema["required"].as_array().expect("required");
        assert!(required.contains(&json!("primal_path")));
    }

    #[test]
    fn test_primal_audit_output_schema() {
        let schema = primal_audit_output_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("total_violations"));
        assert!(props.contains_key("grade"));
        assert!(props.contains_key("suggested_fixes"));
    }

    #[test]
    fn test_rootpulse_commit_input_schema() {
        let schema = rootpulse_commit_input_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("diff"));
        assert!(props.contains_key("context"));
        let required = schema["required"].as_array().expect("required");
        assert!(required.contains(&json!("diff")));
    }

    #[test]
    fn test_rootpulse_commit_output_schema() {
        let schema = rootpulse_commit_output_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("commit_message"));
        assert!(props.contains_key("semantic_tags"));
        assert!(props.contains_key("attribution_weight"));
    }

    #[test]
    fn test_neural_graph_optimize_input_schema() {
        let schema = neural_graph_optimize_input_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("graph_description"));
        assert!(props.contains_key("purpose"));
        assert!(props.contains_key("expected_latency_ms"));
        let required = schema["required"].as_array().expect("required");
        assert!(required.contains(&json!("graph_description")));
        assert!(required.contains(&json!("purpose")));
    }

    #[test]
    fn test_neural_graph_optimize_output_schema() {
        let schema = neural_graph_optimize_output_schema();
        assert_object_schema(&schema);
        let props = schema["properties"].as_object().expect("properties");
        assert!(props.contains_key("analysis"));
        assert!(props.contains_key("recommendations"));
        assert!(props.contains_key("optimized_graph"));
        let required = schema["required"].as_array().expect("required");
        assert!(required.contains(&json!("analysis")));
        assert!(required.contains(&json!("recommendations")));
    }

    #[test]
    fn test_schemas_are_valid_json() {
        // All schemas should serialize to valid JSON strings
        let schemas: Vec<Value> = vec![
            primal_analyze_input_schema(),
            primal_analyze_output_schema(),
            primal_audit_input_schema(),
            primal_audit_output_schema(),
            rootpulse_commit_input_schema(),
            rootpulse_commit_output_schema(),
            neural_graph_optimize_input_schema(),
            neural_graph_optimize_output_schema(),
        ];
        for schema in schemas {
            let json_str = serde_json::to_string(&schema).expect("serialize");
            let _roundtrip: Value = serde_json::from_str(&json_str).expect("roundtrip");
        }
    }
}
