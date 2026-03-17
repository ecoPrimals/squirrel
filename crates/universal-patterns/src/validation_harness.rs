// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Validation harness for multi-check binary validation.
//!
//! Absorbed from rhizoCrypt v0.13. Runs N checks and reports
//! pass/fail/skip per check with a summary. Useful for `squirrel doctor`
//! and validation subcommands.
//!
//! # Usage
//!
//! ```ignore
//! use universal_patterns::ValidationHarness;
//!
//! let mut harness = ValidationHarness::new("squirrel doctor");
//! harness.check("config", || { validate_config() });
//! harness.check("socket", || { validate_socket() });
//! harness.print_summary();
//! ```

use std::fmt;
use std::time::{Duration, Instant};

/// Result of a single validation check.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Check name.
    pub name: String,
    /// Check outcome.
    pub outcome: CheckOutcome,
    /// How long the check took.
    pub duration: Duration,
    /// Optional detail message.
    pub detail: Option<String>,
}

/// Outcome of a validation check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutcome {
    /// Check passed.
    Pass,
    /// Check failed.
    Fail,
    /// Check was skipped (precondition not met).
    Skip,
    /// Check produced a warning (non-fatal issue).
    Warn,
}

impl fmt::Display for CheckOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Fail => write!(f, "FAIL"),
            Self::Skip => write!(f, "SKIP"),
            Self::Warn => write!(f, "WARN"),
        }
    }
}

/// Multi-check validation harness.
pub struct ValidationHarness {
    /// Harness name (e.g., "squirrel doctor").
    pub name: String,
    /// Collected check results.
    pub results: Vec<CheckResult>,
}

impl ValidationHarness {
    /// Create a new harness.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            results: Vec::new(),
        }
    }

    /// Run a synchronous check and record the result.
    pub fn check<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce() -> Result<(), String>,
    {
        let start = Instant::now();
        let (outcome, detail) = match f() {
            Ok(()) => (CheckOutcome::Pass, None),
            Err(msg) => (CheckOutcome::Fail, Some(msg)),
        };
        self.results.push(CheckResult {
            name: name.to_string(),
            outcome,
            duration: start.elapsed(),
            detail,
        });
    }

    /// Run an async check and record the result.
    pub async fn check_async<F, Fut>(&mut self, name: &str, f: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let start = Instant::now();
        let (outcome, detail) = match f().await {
            Ok(()) => (CheckOutcome::Pass, None),
            Err(msg) => (CheckOutcome::Fail, Some(msg)),
        };
        self.results.push(CheckResult {
            name: name.to_string(),
            outcome,
            duration: start.elapsed(),
            detail,
        });
    }

    /// Record a skipped check.
    pub fn skip(&mut self, name: &str, reason: &str) {
        self.results.push(CheckResult {
            name: name.to_string(),
            outcome: CheckOutcome::Skip,
            duration: Duration::ZERO,
            detail: Some(reason.to_string()),
        });
    }

    /// Record a warning check.
    pub fn warn(&mut self, name: &str, message: &str) {
        self.results.push(CheckResult {
            name: name.to_string(),
            outcome: CheckOutcome::Warn,
            duration: Duration::ZERO,
            detail: Some(message.to_string()),
        });
    }

    /// Count of passed checks.
    #[must_use]
    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.outcome == CheckOutcome::Pass)
            .count()
    }

    /// Count of failed checks.
    #[must_use]
    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.outcome == CheckOutcome::Fail)
            .count()
    }

    /// Count of skipped checks.
    #[must_use]
    pub fn skipped(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.outcome == CheckOutcome::Skip)
            .count()
    }

    /// Count of warnings.
    #[must_use]
    pub fn warnings(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.outcome == CheckOutcome::Warn)
            .count()
    }

    /// Total number of checks.
    #[must_use]
    pub fn total(&self) -> usize {
        self.results.len()
    }

    /// Whether all non-skipped checks passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.failed() == 0
    }

    /// Format results as a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        use std::fmt::Write;
        let mut out = format!("=== {} ===\n", self.name);
        for r in &self.results {
            let detail = r.detail.as_deref().unwrap_or("");
            let _ = writeln!(
                out,
                "  [{:4}] {:30} {:>8.1?} {}",
                r.outcome, r.name, r.duration, detail
            );
        }
        let _ = writeln!(
            out,
            "\n  {} passed, {} failed, {} skipped, {} warnings ({} total)",
            self.passed(),
            self.failed(),
            self.skipped(),
            self.warnings(),
            self.total(),
        );
        out
    }

    /// Format results as JSON.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "harness": self.name,
            "results": self.results.iter().map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "outcome": r.outcome.to_string(),
                    "duration_ms": r.duration.as_millis(),
                    "detail": r.detail,
                })
            }).collect::<Vec<_>>(),
            "summary": {
                "passed": self.passed(),
                "failed": self.failed(),
                "skipped": self.skipped(),
                "warnings": self.warnings(),
                "total": self.total(),
                "all_passed": self.all_passed(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_harness() {
        let h = ValidationHarness::new("test");
        assert_eq!(h.total(), 0);
        assert!(h.all_passed());
    }

    #[test]
    fn passing_check() {
        let mut h = ValidationHarness::new("test");
        h.check("ok check", || Ok(()));
        assert_eq!(h.passed(), 1);
        assert_eq!(h.failed(), 0);
        assert!(h.all_passed());
    }

    #[test]
    fn failing_check() {
        let mut h = ValidationHarness::new("test");
        h.check("bad check", || Err("something broke".into()));
        assert_eq!(h.passed(), 0);
        assert_eq!(h.failed(), 1);
        assert!(!h.all_passed());
    }

    #[test]
    fn skip_and_warn() {
        let mut h = ValidationHarness::new("test");
        h.skip("optional", "not applicable");
        h.warn("deprecation", "old API used");
        assert_eq!(h.skipped(), 1);
        assert_eq!(h.warnings(), 1);
        assert!(h.all_passed());
    }

    #[test]
    fn mixed_results() {
        let mut h = ValidationHarness::new("mixed");
        h.check("pass", || Ok(()));
        h.check("fail", || Err("broken".into()));
        h.skip("skip", "n/a");
        h.warn("warn", "heads up");
        assert_eq!(h.total(), 4);
        assert_eq!(h.passed(), 1);
        assert_eq!(h.failed(), 1);
        assert!(!h.all_passed());
    }

    #[test]
    fn summary_formatting() {
        let mut h = ValidationHarness::new("test");
        h.check("ok", || Ok(()));
        let summary = h.summary();
        assert!(summary.contains("PASS"));
        assert!(summary.contains("1 passed"));
    }

    #[test]
    fn json_output() {
        let mut h = ValidationHarness::new("test");
        h.check("ok", || Ok(()));
        let json = h.to_json();
        assert_eq!(json["summary"]["passed"], 1);
        assert_eq!(json["summary"]["all_passed"], true);
    }

    #[tokio::test]
    async fn async_check() {
        let mut h = ValidationHarness::new("async test");
        h.check_async("async ok", || async { Ok(()) }).await;
        h.check_async("async fail", || async { Err("nope".into()) })
            .await;
        assert_eq!(h.passed(), 1);
        assert_eq!(h.failed(), 1);
    }
}
