// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Human dignity evaluation layer for AI routing (biomeOS sovereignty_guardian pattern).
//!
//! Per wateringHole standards, AI operations include dignity checks: discrimination
//! prevention, human oversight, manipulation prevention, and right to explanation.

use std::fmt;
use tracing::{info, warn};

/// Environment variable controlling dignity check enforcement (`warn` | `enforce` | `audit`).
pub const DIGNITY_ENFORCEMENT_ENV: &str = "SQUIRREL_DIGNITY_ENFORCEMENT";

/// How failed dignity checks are handled at the router boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DignityEnforcementLevel {
    /// Log a warning and allow the request (default).
    #[default]
    WarnOnly,
    /// Block the request and surface an error to the caller.
    Enforce,
    /// Emit a structured audit event and allow the request.
    AuditLog,
}

impl DignityEnforcementLevel {
    /// Read enforcement level from [`DIGNITY_ENFORCEMENT_ENV`]; defaults to [`WarnOnly`].
    pub fn from_env() -> Self {
        std::env::var(DIGNITY_ENFORCEMENT_ENV)
            .ok()
            .as_deref()
            .map_or(Self::WarnOnly, Self::parse)
    }

    /// Parse env value: `"warn"`, `"enforce"`, or `"audit"` (unknown values → warn).
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "enforce" => Self::Enforce,
            "audit" => Self::AuditLog,
            _ => Self::WarnOnly,
        }
    }
}

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
    enforcement_level: DignityEnforcementLevel,
}

impl DignityGuard {
    /// Create a guard with warn-only enforcement (matches legacy router behavior).
    pub const fn new() -> Self {
        Self::with_enforcement(DignityEnforcementLevel::WarnOnly)
    }

    /// Create a guard with the given enforcement level.
    pub const fn with_enforcement(enforcement_level: DignityEnforcementLevel) -> Self {
        Self {
            evaluator: DignityEvaluator,
            enforcement_level,
        }
    }

    /// Apply dignity checks using the configured enforcement level.
    pub fn guard(&self, request: &DignityCheckRequest<'_>) -> Result<(), DignityViolation> {
        let result = self.evaluator.evaluate_request(request);
        if result.passed {
            if matches!(self.enforcement_level, DignityEnforcementLevel::AuditLog) {
                info!(
                    target: "dignity.audit",
                    passed = true,
                    prompt_len = request.prompt.len(),
                    model = ?request.model,
                    context = ?request.context,
                    "Dignity audit: all checks passed"
                );
            }
            return Ok(());
        }

        match self.enforcement_level {
            DignityEnforcementLevel::WarnOnly => {
                warn!(
                    "Dignity check failed (wateringHole/sovereignty guard): {}",
                    result.explanation
                );
                Ok(())
            }
            DignityEnforcementLevel::AuditLog => {
                info!(
                    target: "dignity.audit",
                    passed = false,
                    flag_count = result.flags.len(),
                    flags = ?result.flags,
                    explanation = %result.explanation,
                    prompt_len = request.prompt.len(),
                    model = ?request.model,
                    context = ?request.context,
                    "Dignity audit: check failed"
                );
                Ok(())
            }
            DignityEnforcementLevel::Enforce => Err(DignityViolation { result }),
        }
    }

    /// Run dignity check; always blocks on failure regardless of enforcement level.
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

    // --- DignityEnforcementLevel ---

    #[test]
    fn enforcement_level_parse_values() {
        assert_eq!(
            DignityEnforcementLevel::parse("warn"),
            DignityEnforcementLevel::WarnOnly
        );
        assert_eq!(
            DignityEnforcementLevel::parse("enforce"),
            DignityEnforcementLevel::Enforce
        );
        assert_eq!(
            DignityEnforcementLevel::parse("audit"),
            DignityEnforcementLevel::AuditLog
        );
        assert_eq!(
            DignityEnforcementLevel::parse("unknown"),
            DignityEnforcementLevel::WarnOnly
        );
    }

    #[test]
    fn enforcement_level_from_env_defaults_to_warn() {
        temp_env::with_var(DIGNITY_ENFORCEMENT_ENV, None::<&str>, || {
            assert_eq!(
                DignityEnforcementLevel::from_env(),
                DignityEnforcementLevel::WarnOnly
            );
        });
    }

    #[test]
    fn enforcement_level_from_env_reads_enforce() {
        temp_env::with_var(DIGNITY_ENFORCEMENT_ENV, Some("enforce"), || {
            assert_eq!(
                DignityEnforcementLevel::from_env(),
                DignityEnforcementLevel::Enforce
            );
        });
    }

    // --- DignityGuard::guard (enforcement modes) ---

    #[test]
    fn guard_warn_only_allows_failed_check() {
        let guard = DignityGuard::with_enforcement(DignityEnforcementLevel::WarnOnly);
        let request = make_request("Should we hire this applicant?", None, None);
        assert!(guard.guard(&request).is_ok());
    }

    #[test]
    fn guard_audit_log_allows_failed_check() {
        let guard = DignityGuard::with_enforcement(DignityEnforcementLevel::AuditLog);
        let request = make_request("Should we hire this applicant?", None, None);
        assert!(guard.guard(&request).is_ok());
    }

    #[test]
    fn guard_enforce_blocks_failed_check() {
        let guard = DignityGuard::with_enforcement(DignityEnforcementLevel::Enforce);
        let request = make_request("Should we hire this applicant?", None, None);
        let err = guard.guard(&request).unwrap_err();
        assert!(!err.result.passed);
    }

    #[test]
    fn guard_enforce_allows_clean_prompt() {
        let guard = DignityGuard::with_enforcement(DignityEnforcementLevel::Enforce);
        let request = make_request("Summarize this article.", Some("mistral"), None);
        assert!(guard.guard(&request).is_ok());
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
