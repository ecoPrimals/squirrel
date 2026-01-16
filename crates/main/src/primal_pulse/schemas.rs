//! JSON schemas for PrimalPulse tools
//!
//! Defines input/output schemas for dynamic tool registration

use serde_json::{json, Value};

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
