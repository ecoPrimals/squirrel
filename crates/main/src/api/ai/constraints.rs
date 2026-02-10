// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Idiomatic constraint system for AI routing
#![allow(dead_code)] // Public API surface awaiting consumer activation
//!
//! Allows users, teams, and other primals to configure routing preferences.
//! Designed to be extensible for future constraint types.

use serde::{Deserialize, Serialize};

/// Routing constraint that influences provider selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingConstraint {
    /// Optimize for minimum cost
    OptimizeCost,

    /// Optimize for minimum latency
    OptimizeSpeed,

    /// Optimize for quality/accuracy
    OptimizeQuality,

    /// Require local execution (privacy)
    RequireLocal,

    /// Require specific provider
    RequireProvider(String),

    /// Prefer but don't require local
    PreferLocal,

    /// Prefer specific quality tier
    PreferQuality(QualityTier),

    /// Maximum acceptable cost per request
    MaxCost(f64),

    /// Maximum acceptable latency (ms)
    MaxLatency(u64),

    /// Minimum acceptable quality tier
    MinQuality(QualityTier),

    /// Custom constraint (extensible)
    Custom {
        key: String,
        value: serde_json::Value,
    },
}

// Re-export QualityTier from adapters to avoid duplication
pub use super::adapters::QualityTier;

/// Constraint priority for conflict resolution
///
/// Variants are ordered from lowest to highest so derived `Ord`
/// gives `Optional < Preferred < Required`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConstraintPriority {
    /// Nice to have but not important
    Optional,

    /// Should be satisfied if possible
    Preferred,

    /// Must be satisfied (hard constraint)
    Required,
}

/// A constraint with its priority
#[derive(Debug, Clone)]
pub struct PrioritizedConstraint {
    pub constraint: RoutingConstraint,
    pub priority: ConstraintPriority,
}

/// Source of a constraint (for audit/debug)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintSource {
    /// User-specified in request
    User,

    /// Team/organization policy
    Team(String),

    /// Another primal service
    Primal { name: String, reason: String },

    /// System default
    System,

    /// Compliance requirement (GDPR, etc.)
    Compliance(String),
}

/// Complete constraint set with sources
#[derive(Debug, Clone)]
pub struct ConstraintSet {
    constraints: Vec<(PrioritizedConstraint, ConstraintSource)>,
}

impl ConstraintSet {
    /// Create an empty constraint set
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add a constraint with priority and source
    pub fn add(
        &mut self,
        constraint: RoutingConstraint,
        priority: ConstraintPriority,
        source: ConstraintSource,
    ) {
        self.constraints.push((
            PrioritizedConstraint {
                constraint,
                priority,
            },
            source,
        ));
    }

    /// Add a required constraint
    pub fn require(&mut self, constraint: RoutingConstraint, source: ConstraintSource) {
        self.add(constraint, ConstraintPriority::Required, source);
    }

    /// Add a preferred constraint
    pub fn prefer(&mut self, constraint: RoutingConstraint, source: ConstraintSource) {
        self.add(constraint, ConstraintPriority::Preferred, source);
    }

    /// Get all required constraints
    pub fn required(&self) -> Vec<&RoutingConstraint> {
        self.constraints
            .iter()
            .filter(|(pc, _)| pc.priority == ConstraintPriority::Required)
            .map(|(pc, _)| &pc.constraint)
            .collect()
    }

    /// Get all preferred constraints
    pub fn preferred(&self) -> Vec<&RoutingConstraint> {
        self.constraints
            .iter()
            .filter(|(pc, _)| pc.priority == ConstraintPriority::Preferred)
            .map(|(pc, _)| &pc.constraint)
            .collect()
    }

    /// Get constraints by source
    pub fn by_source(&self, source: &ConstraintSource) -> Vec<&RoutingConstraint> {
        self.constraints
            .iter()
            .filter(|(_, s)| s == source)
            .map(|(pc, _)| &pc.constraint)
            .collect()
    }

    /// Check if constraint set has any cost optimization
    pub fn optimizes_cost(&self) -> bool {
        self.constraints
            .iter()
            .any(|(pc, _)| matches!(pc.constraint, RoutingConstraint::OptimizeCost))
    }

    /// Check if constraint set requires local execution
    pub fn requires_local(&self) -> bool {
        self.required()
            .iter()
            .any(|c| matches!(c, RoutingConstraint::RequireLocal))
    }

    /// Get maximum acceptable cost
    pub fn max_cost(&self) -> Option<f64> {
        self.required()
            .iter()
            .filter_map(|c| match c {
                RoutingConstraint::MaxCost(cost) => Some(*cost),
                _ => None,
            })
            .min_by(|a, b| {
                // Safe comparison: treat NaN as greater (worse) than any number
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Greater)
            })
    }

    /// Get maximum acceptable latency
    pub fn max_latency(&self) -> Option<u64> {
        self.required()
            .iter()
            .filter_map(|c| match c {
                RoutingConstraint::MaxLatency(ms) => Some(*ms),
                _ => None,
            })
            .min()
    }

    /// Get minimum required quality
    pub fn min_quality(&self) -> Option<QualityTier> {
        self.required()
            .iter()
            .filter_map(|c| match c {
                RoutingConstraint::MinQuality(tier) => Some(*tier),
                _ => None,
            })
            .max()
    }
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating constraint sets idiomatically
pub struct ConstraintBuilder {
    set: ConstraintSet,
}

impl ConstraintBuilder {
    /// Create a new constraint builder
    pub fn new() -> Self {
        Self {
            set: ConstraintSet::new(),
        }
    }

    /// User wants to optimize for cost
    pub fn optimize_cost(mut self) -> Self {
        self.set
            .prefer(RoutingConstraint::OptimizeCost, ConstraintSource::User);
        self
    }

    /// User wants to optimize for speed
    pub fn optimize_speed(mut self) -> Self {
        self.set
            .prefer(RoutingConstraint::OptimizeSpeed, ConstraintSource::User);
        self
    }

    /// User wants to optimize for quality
    pub fn optimize_quality(mut self) -> Self {
        self.set
            .prefer(RoutingConstraint::OptimizeQuality, ConstraintSource::User);
        self
    }

    /// User requires local execution
    pub fn require_local(mut self) -> Self {
        self.set
            .require(RoutingConstraint::RequireLocal, ConstraintSource::User);
        self
    }

    /// User prefers local execution
    pub fn prefer_local(mut self) -> Self {
        self.set
            .prefer(RoutingConstraint::PreferLocal, ConstraintSource::User);
        self
    }

    /// Set maximum cost
    pub fn max_cost(mut self, cost: f64) -> Self {
        self.set
            .require(RoutingConstraint::MaxCost(cost), ConstraintSource::User);
        self
    }

    /// Set maximum latency
    pub fn max_latency(mut self, ms: u64) -> Self {
        self.set
            .require(RoutingConstraint::MaxLatency(ms), ConstraintSource::User);
        self
    }

    /// Set minimum quality
    pub fn min_quality(mut self, tier: QualityTier) -> Self {
        self.set
            .require(RoutingConstraint::MinQuality(tier), ConstraintSource::User);
        self
    }

    /// Add team policy
    pub fn with_team_policy(mut self, team: String, constraint: RoutingConstraint) -> Self {
        self.set.require(constraint, ConstraintSource::Team(team));
        self
    }

    /// Add primal constraint (e.g., from `ToadStool` or `NestGate`)
    pub fn with_primal_constraint(
        mut self,
        primal: String,
        reason: String,
        constraint: RoutingConstraint,
    ) -> Self {
        self.set.prefer(
            constraint,
            ConstraintSource::Primal {
                name: primal,
                reason,
            },
        );
        self
    }

    /// Add compliance requirement
    pub fn with_compliance(mut self, regulation: String, constraint: RoutingConstraint) -> Self {
        self.set
            .require(constraint, ConstraintSource::Compliance(regulation));
        self
    }

    /// Add custom constraint
    pub fn with_custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.set.prefer(
            RoutingConstraint::Custom { key, value },
            ConstraintSource::User,
        );
        self
    }

    /// Build the constraint set
    pub fn build(self) -> ConstraintSet {
        self.set
    }
}

impl Default for ConstraintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse constraints from request JSON
pub fn from_request(req: &serde_json::Value) -> ConstraintSet {
    let mut builder = ConstraintBuilder::new();

    // Legacy field support
    if let Some(cost_pref) = req.get("cost_preference").and_then(|v| v.as_str()) {
        match cost_pref {
            "optimize" | "minimize" => builder = builder.optimize_cost(),
            "balanced" => {} // Default
            _ => {}
        }
    }

    if let Some(privacy) = req.get("privacy_level").and_then(|v| v.as_str()) {
        match privacy {
            "local" | "private" => builder = builder.require_local(),
            "prefer_local" => builder = builder.prefer_local(),
            _ => {}
        }
    }

    if let Some(quality) = req.get("quality").and_then(|v| v.as_str()) {
        let tier = match quality {
            "basic" => QualityTier::Basic,
            "standard" => QualityTier::Standard,
            "high" => QualityTier::High,
            "premium" => QualityTier::Premium,
            _ => QualityTier::Standard,
        };
        builder = builder.min_quality(tier);
    }

    if let Some(speed) = req.get("speed_preference").and_then(|v| v.as_str()) {
        if speed == "fast" || speed == "optimize" {
            builder = builder.optimize_speed();
        }
    }

    // New constraint system
    if let Some(constraints) = req.get("constraints").and_then(|v| v.as_array()) {
        for constraint in constraints {
            if let Some(type_) = constraint.get("type").and_then(|v| v.as_str()) {
                match type_ {
                    "optimize_cost" => builder = builder.optimize_cost(),
                    "optimize_speed" => builder = builder.optimize_speed(),
                    "optimize_quality" => builder = builder.optimize_quality(),
                    "require_local" => builder = builder.require_local(),
                    "prefer_local" => builder = builder.prefer_local(),
                    "max_cost" => {
                        if let Some(cost) =
                            constraint.get("value").and_then(serde_json::Value::as_f64)
                        {
                            builder = builder.max_cost(cost);
                        }
                    }
                    "max_latency" => {
                        if let Some(ms) =
                            constraint.get("value").and_then(serde_json::Value::as_u64)
                        {
                            builder = builder.max_latency(ms);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_builder() {
        let constraints = ConstraintBuilder::new()
            .optimize_cost()
            .prefer_local()
            .max_latency(5000)
            .build();

        assert!(constraints.optimizes_cost());
        assert!(!constraints.requires_local());
        assert_eq!(constraints.max_latency(), Some(5000));
    }

    #[test]
    fn test_constraint_priority() {
        let mut constraints = ConstraintSet::new();
        constraints.require(RoutingConstraint::RequireLocal, ConstraintSource::User);
        constraints.prefer(RoutingConstraint::OptimizeCost, ConstraintSource::User);

        assert!(constraints.requires_local());
        assert!(constraints.optimizes_cost());
    }

    #[test]
    fn test_from_request() {
        let req = serde_json::json!({
            "privacy_level": "local",
            "cost_preference": "optimize"
        });

        let constraints = from_request(&req);
        assert!(constraints.requires_local());
        assert!(constraints.optimizes_cost());
    }

    #[test]
    fn test_team_policy() {
        let constraints = ConstraintBuilder::new()
            .with_team_policy("engineering".to_string(), RoutingConstraint::MaxCost(0.01))
            .build();

        assert_eq!(constraints.max_cost(), Some(0.01));
    }

    #[test]
    fn test_primal_constraint() {
        let constraints = ConstraintBuilder::new()
            .with_primal_constraint(
                "nestgate".to_string(),
                "sensitive data".to_string(),
                RoutingConstraint::RequireLocal,
            )
            .build();

        // Primal constraints are preferred by default, not required
        assert!(!constraints.requires_local());
    }

    #[test]
    fn test_constraint_set_default_empty() {
        let set = ConstraintSet::default();
        assert!(set.required().is_empty());
        assert!(set.preferred().is_empty());
        assert!(!set.optimizes_cost());
        assert!(!set.requires_local());
        assert!(set.max_cost().is_none());
        assert!(set.max_latency().is_none());
        assert!(set.min_quality().is_none());
    }

    #[test]
    fn test_min_quality() {
        let constraints = ConstraintBuilder::new()
            .min_quality(QualityTier::High)
            .build();
        assert_eq!(constraints.min_quality(), Some(QualityTier::High));
    }

    #[test]
    fn test_multiple_max_costs_takes_minimum() {
        let mut set = ConstraintSet::new();
        set.require(RoutingConstraint::MaxCost(0.05), ConstraintSource::User);
        set.require(
            RoutingConstraint::MaxCost(0.01),
            ConstraintSource::Team("eng".to_string()),
        );
        assert!((set.max_cost().unwrap() - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_max_latencies_takes_minimum() {
        let mut set = ConstraintSet::new();
        set.require(RoutingConstraint::MaxLatency(5000), ConstraintSource::User);
        set.require(
            RoutingConstraint::MaxLatency(1000),
            ConstraintSource::System,
        );
        assert_eq!(set.max_latency(), Some(1000));
    }

    #[test]
    fn test_by_source() {
        let mut set = ConstraintSet::new();
        set.require(RoutingConstraint::RequireLocal, ConstraintSource::User);
        set.require(RoutingConstraint::MaxCost(0.01), ConstraintSource::System);
        set.prefer(RoutingConstraint::OptimizeCost, ConstraintSource::User);

        let user_constraints = set.by_source(&ConstraintSource::User);
        assert_eq!(user_constraints.len(), 2);

        let system_constraints = set.by_source(&ConstraintSource::System);
        assert_eq!(system_constraints.len(), 1);
    }

    #[test]
    fn test_routing_constraint_serde_roundtrip() {
        let constraints = vec![
            RoutingConstraint::OptimizeCost,
            RoutingConstraint::OptimizeSpeed,
            RoutingConstraint::OptimizeQuality,
            RoutingConstraint::RequireLocal,
            RoutingConstraint::PreferLocal,
            RoutingConstraint::RequireProvider("openai".to_string()),
            RoutingConstraint::PreferQuality(QualityTier::Premium),
            RoutingConstraint::MaxCost(0.05),
            RoutingConstraint::MaxLatency(3000),
            RoutingConstraint::MinQuality(QualityTier::High),
            RoutingConstraint::Custom {
                key: "region".to_string(),
                value: serde_json::json!("us-east-1"),
            },
        ];
        for c in &constraints {
            let json = serde_json::to_string(c).unwrap();
            let deser: RoutingConstraint = serde_json::from_str(&json).unwrap();
            assert_eq!(*c, deser, "Failed roundtrip for: {:?}", c);
        }
    }

    #[test]
    fn test_constraint_source_serde() {
        let sources = vec![
            ConstraintSource::User,
            ConstraintSource::Team("eng".to_string()),
            ConstraintSource::Primal {
                name: "compute".to_string(),
                reason: "gpu needed".to_string(),
            },
            ConstraintSource::System,
            ConstraintSource::Compliance("GDPR".to_string()),
        ];
        for s in &sources {
            let json = serde_json::to_string(s).unwrap();
            let deser: ConstraintSource = serde_json::from_str(&json).unwrap();
            assert_eq!(*s, deser, "Failed roundtrip for: {:?}", s);
        }
    }

    #[test]
    fn test_constraint_priority_ordering() {
        assert!(ConstraintPriority::Required > ConstraintPriority::Preferred);
        assert!(ConstraintPriority::Preferred > ConstraintPriority::Optional);
    }

    #[test]
    fn test_from_request_quality() {
        let req = serde_json::json!({"quality": "premium"});
        let constraints = from_request(&req);
        assert_eq!(constraints.min_quality(), Some(QualityTier::Premium));
    }

    #[test]
    fn test_from_request_speed() {
        let req = serde_json::json!({"speed_preference": "fast"});
        let constraints = from_request(&req);
        let preferred = constraints.preferred();
        assert!(preferred
            .iter()
            .any(|c| matches!(c, RoutingConstraint::OptimizeSpeed)));
    }

    #[test]
    fn test_from_request_constraints_array() {
        let req = serde_json::json!({
            "constraints": [
                {"type": "max_cost", "value": 0.05},
                {"type": "max_latency", "value": 2000},
                {"type": "require_local"}
            ]
        });
        let constraints = from_request(&req);
        assert_eq!(constraints.max_cost(), Some(0.05));
        assert_eq!(constraints.max_latency(), Some(2000));
        assert!(constraints.requires_local());
    }

    #[test]
    fn test_from_request_empty() {
        let req = serde_json::json!({});
        let constraints = from_request(&req);
        assert!(!constraints.optimizes_cost());
        assert!(!constraints.requires_local());
    }

    #[test]
    fn test_compliance_constraint() {
        let constraints = ConstraintBuilder::new()
            .with_compliance("GDPR".to_string(), RoutingConstraint::RequireLocal)
            .build();
        assert!(constraints.requires_local());
        let gdpr = constraints.by_source(&ConstraintSource::Compliance("GDPR".to_string()));
        assert_eq!(gdpr.len(), 1);
    }

    #[test]
    fn test_custom_constraint() {
        let constraints = ConstraintBuilder::new()
            .with_custom("region".to_string(), serde_json::json!("us-east"))
            .build();
        let preferred = constraints.preferred();
        assert!(preferred
            .iter()
            .any(|c| matches!(c, RoutingConstraint::Custom { .. })));
    }

    #[test]
    fn test_quality_tier_ordering() {
        assert!(QualityTier::Premium > QualityTier::High);
        assert!(QualityTier::High > QualityTier::Standard);
        assert!(QualityTier::Standard > QualityTier::Fast);
        assert!(QualityTier::Fast > QualityTier::Basic);
    }
}
