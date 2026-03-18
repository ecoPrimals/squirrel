// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Human dignity evaluation layer for AI routing (biomeOS sovereignty_guardian pattern).
//!
//! Per wateringHole standards, AI operations include dignity checks: discrimination
//! prevention, human oversight, manipulation prevention, and right to explanation.

#![forbid(unsafe_code)]
#![allow(dead_code)] // Public API surface awaiting consumer activation

use std::fmt;

/// Request payload for dignity evaluation.
#[derive(Debug, Clone)]
pub struct DignityCheckRequest<'a> {
    /// The AI prompt being evaluated.
    pub prompt: &'a str,
    /// Model being used (if known).
    pub model: Option<&'a str>,
    /// Usage context (e.g. "automated", "human_review").
    pub context: Option<&'a str>,
}

/// Result of a dignity evaluation.
#[derive(Debug, Clone)]
pub struct DignityCheckResult {
    /// Whether all dignity checks passed.
    pub passed: bool,
    /// Any dignity flags raised.
    pub flags: Vec<DignityFlag>,
    /// Human-readable explanation of the evaluation.
    pub explanation: String,
}

/// Dignity violation flag categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DignityFlag {
    /// Potential discriminatory content or high-stakes decision about people.
    DiscriminationRisk(String),
    /// No human-in-the-loop configured.
    MissingHumanOversight,
    /// Potential manipulation (urgency, scarcity, emotional).
    ManipulationRisk(String),
    /// Model output not explainable.
    MissingExplanation,
}

/// Error returned when a dignity check fails.
#[derive(Debug, Clone)]
pub struct DignityViolation {
    /// The evaluation result that triggered the violation.
    pub result: DignityCheckResult,
}

impl fmt::Display for DignityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.result.explanation)
    }
}

impl std::error::Error for DignityViolation {}

/// Evaluates AI requests for dignity violations using deterministic pattern matching.
#[derive(Debug, Clone, Default)]
pub struct DignityEvaluator;

impl DignityEvaluator {
    /// Evaluate a request for dignity violations.
    pub fn evaluate_request(&self, request: &DignityCheckRequest<'_>) -> DignityCheckResult {
        let mut flags = Vec::new();
        let prompt_lower = request.prompt.to_lowercase();
        let context_lower = request.context.map(str::to_lowercase);

        // Discrimination: high-stakes decisions about people
        if Self::has_discrimination_risk(&prompt_lower) {
            flags.push(DignityFlag::DiscriminationRisk(
                "Prompt requests decisions about access, employment, housing, or credit".into(),
            ));
        }

        // Human oversight: context indicates no human review
        if Self::lacks_human_oversight(context_lower.as_ref()) {
            flags.push(DignityFlag::MissingHumanOversight);
        }

        // Manipulation: urgency, scarcity, emotional manipulation
        if let Some(detail) = Self::has_manipulation_risk(&prompt_lower) {
            flags.push(DignityFlag::ManipulationRisk(detail));
        }

        // Explainability: model may not support provenance
        if Self::lacks_explainability(request.model, context_lower.as_ref()) {
            flags.push(DignityFlag::MissingExplanation);
        }

        let passed = flags.is_empty();
        let explanation = if passed {
            "All dignity checks passed.".to_string()
        } else {
            format!(
                "Dignity check failed: {} flag(s). {}",
                flags.len(),
                flags
                    .iter()
                    .map(|f| match f {
                        DignityFlag::MissingHumanOversight => "No human oversight",
                        DignityFlag::DiscriminationRisk(s) | DignityFlag::ManipulationRisk(s) => {
                            s.as_str()
                        }
                        DignityFlag::MissingExplanation => "Output not explainable",
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };

        DignityCheckResult {
            passed,
            flags,
            explanation,
        }
    }

    fn has_discrimination_risk(prompt: &str) -> bool {
        const RISK_PHRASES: &[&str] = &[
            "hire",
            "fire",
            "applicant",
            "candidate",
            "employment",
            "job application",
            "housing",
            "tenant",
            "landlord",
            "rent approval",
            "mortgage",
            "credit score",
            "loan approval",
            "deny credit",
            "credit decision",
            "access denied",
            "eligibility",
            "deny access",
            "approve access",
        ];
        RISK_PHRASES.iter().any(|p| prompt.contains(p))
    }

    fn lacks_human_oversight(context: Option<&String>) -> bool {
        let Some(ctx) = context else { return false };
        const NO_OVERSIGHT: &[&str] = &["automated", "no human", "without review", "unattended"];
        NO_OVERSIGHT.iter().any(|p| ctx.contains(p))
    }

    fn has_manipulation_risk(prompt: &str) -> Option<String> {
        const URGENCY: &[&str] = &["act now", "limited time", "expires soon", "urgent"];
        const SCARCITY: &[&str] = &["only X left", "last chance", "don't miss", "exclusive"];
        const EMOTIONAL: &[&str] = &["you must", "you need to", "everyone else"];
        if URGENCY.iter().any(|p| prompt.contains(p)) {
            return Some("Urgency pressure detected".into());
        }
        if SCARCITY.iter().any(|p| prompt.contains(p)) {
            return Some("False scarcity detected".into());
        }
        if EMOTIONAL.iter().any(|p| prompt.contains(p)) {
            return Some("Emotional manipulation risk".into());
        }
        None
    }

    fn lacks_explainability(model: Option<&str>, context: Option<&String>) -> bool {
        let ctx = context.map_or("", String::as_str);
        if ctx.contains("explainable") || ctx.contains("provenance") {
            return false;
        }
        // Black-box models without known explainability
        model.is_some_and(|m| {
            let m = m.to_lowercase();
            m.contains("gpt-4") || m.contains("claude") || m.contains("opaque")
        })
    }
}

/// Guard that performs dignity checks before allowing AI routing.
#[derive(Debug, Clone)]
pub struct DignityGuard {
    evaluator: DignityEvaluator,
}

impl DignityGuard {
    /// Create a new dignity guard.
    pub const fn new() -> Self {
        Self {
            evaluator: DignityEvaluator,
        }
    }

    /// Run dignity check; returns `Ok(())` if passed, `Err(DignityViolation)` if failed.
    pub fn check_and_route(
        &self,
        request: &DignityCheckRequest<'_>,
    ) -> Result<(), DignityViolation> {
        let result = self.evaluator.evaluate_request(request);
        if result.passed {
            Ok(())
        } else {
            Err(DignityViolation { result })
        }
    }
}

impl Default for DignityGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request<'a>(
        prompt: &'a str,
        model: Option<&'a str>,
        context: Option<&'a str>,
    ) -> DignityCheckRequest<'a> {
        DignityCheckRequest {
            prompt,
            model,
            context,
        }
    }

    // --- DignityEvaluator::evaluate_request ---

    #[test]
    fn evaluate_request_clean_prompt_passes_all_checks() {
        let evaluator = DignityEvaluator;
        let request = make_request(
            "Summarize this article about climate science.",
            Some("mistral-7b"),
            Some("human_review"),
        );
        let result = evaluator.evaluate_request(&request);
        assert!(
            result.passed,
            "Clean prompt should pass: {}",
            result.explanation
        );
        assert!(result.flags.is_empty());
        assert!(result.explanation.contains("passed"));
    }

    #[test]
    fn evaluate_request_discrimination_keywords() {
        let evaluator = DignityEvaluator;
        let prompts = [
            "Should we hire this person?",
            "Should we fire this employee?",
            "Review this applicant for the role",
            "Evaluate housing eligibility",
            "Check credit score for approval",
        ];
        for prompt in prompts {
            let request = make_request(prompt, None, None);
            let result = evaluator.evaluate_request(&request);
            assert!(
                !result.passed,
                "Discrimination phrase '{prompt}' should fail"
            );
            assert!(
                result
                    .flags
                    .iter()
                    .any(|f| matches!(f, DignityFlag::DiscriminationRisk(_))),
                "Expected DiscriminationRisk for '{prompt}'"
            );
        }
    }

    #[test]
    fn evaluate_request_manipulation_keywords() {
        let evaluator = DignityEvaluator;
        let cases = [
            ("act now", "Urgency pressure detected"),
            ("limited time", "Urgency pressure detected"),
            ("last chance", "False scarcity detected"),
            ("don't miss", "False scarcity detected"),
        ];
        for (phrase, expected_detail) in cases {
            let prompt = format!("{phrase} to get this deal!");
            let request = make_request(&prompt, None, None);
            let result = evaluator.evaluate_request(&request);
            assert!(!result.passed, "Manipulation phrase '{phrase}' should fail");
            assert!(
                result
                    .flags
                    .iter()
                    .any(|f| matches!(f, DignityFlag::ManipulationRisk(s) if s == expected_detail)),
                "Expected ManipulationRisk for '{phrase}'"
            );
        }
    }

    #[test]
    fn evaluate_request_missing_human_oversight() {
        let evaluator = DignityEvaluator;
        let request = make_request("Translate this text.", None, Some("automated"));
        let result = evaluator.evaluate_request(&request);
        assert!(!result.passed);
        assert!(result.flags.contains(&DignityFlag::MissingHumanOversight));
    }

    #[test]
    fn evaluate_request_non_explainable_model() {
        let evaluator = DignityEvaluator;
        let request = make_request("Summarize this document.", Some("gpt-4"), None);
        let result = evaluator.evaluate_request(&request);
        assert!(!result.passed);
        assert!(result.flags.contains(&DignityFlag::MissingExplanation));
    }

    #[test]
    fn evaluate_request_explainable_context_overrides_model() {
        let evaluator = DignityEvaluator;
        let request = make_request(
            "Summarize this document.",
            Some("gpt-4"),
            Some("explainable"),
        );
        let result = evaluator.evaluate_request(&request);
        assert!(result.passed, "explainable context should override gpt-4");
    }

    #[test]
    fn evaluate_request_multiple_violations() {
        let evaluator = DignityEvaluator;
        let request = make_request(
            "act now to hire the best applicant before last chance",
            Some("claude-3"),
            Some("automated"),
        );
        let result = evaluator.evaluate_request(&request);
        assert!(!result.passed);
        assert!(
            result.flags.len() >= 3,
            "Expected multiple flags, got {:?}",
            result.flags
        );
        assert!(result.explanation.contains("flag(s)"));
    }

    // --- DignityGuard::check_and_route ---

    #[test]
    fn check_and_route_passes_returns_ok() {
        let guard = DignityGuard::new();
        let request = make_request(
            "What is the capital of France?",
            Some("mistral"),
            Some("human_review"),
        );
        let outcome = guard.check_and_route(&request);
        assert!(outcome.is_ok());
    }

    #[test]
    fn check_and_route_fails_returns_dignity_violation() {
        let guard = DignityGuard::new();
        let request = make_request("Should we fire this applicant?", None, None);
        let outcome = guard.check_and_route(&request);
        let err = outcome.unwrap_err();
        assert!(!err.result.passed);
        assert!(!err.result.flags.is_empty());
        assert!(err.to_string().contains("Dignity check failed"));
    }

    // --- Edge cases ---

    #[test]
    fn edge_case_empty_prompt() {
        let evaluator = DignityEvaluator;
        let request = make_request("", None, None);
        let result = evaluator.evaluate_request(&request);
        assert!(
            result.passed,
            "Empty prompt with no model/context should pass"
        );
    }

    #[test]
    fn edge_case_very_long_prompt() {
        let evaluator = DignityEvaluator;
        let prompt = "Summarize this. ".repeat(10_000);
        let request = make_request(&prompt, None, None);
        let result = evaluator.evaluate_request(&request);
        assert!(result.passed);
    }

    #[test]
    fn edge_case_none_context() {
        let evaluator = DignityEvaluator;
        let request = make_request("Translate hello", Some("gpt-4"), None);
        let result = evaluator.evaluate_request(&request);
        assert!(!result.passed);
        assert!(result.flags.contains(&DignityFlag::MissingExplanation));
        // None context does NOT trigger MissingHumanOversight (lacks_human_oversight returns false)
    }

    #[test]
    fn edge_case_unknown_model() {
        let evaluator = DignityEvaluator;
        let request = make_request("Summarize this.", Some("mistral-7b-instruct"), None);
        let result = evaluator.evaluate_request(&request);
        assert!(
            result.passed,
            "Unknown/explainable model should not trigger MissingExplanation"
        );
    }

    #[test]
    fn dignity_violation_display_and_error() {
        let guard = DignityGuard::new();
        let request = make_request("hire or fire", None, None);
        let err = guard.check_and_route(&request).unwrap_err();
        let display = err.to_string();
        assert!(!display.is_empty());
        assert!(display.contains("flag"));
        assert!(std::error::Error::source(&err).is_none());
    }
}
