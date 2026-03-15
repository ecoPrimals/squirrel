// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Provenance tracking for validation baselines and benchmark results.
//!
//! Records *where* an expected value or benchmark baseline came from,
//! enabling reproducibility and audit. Follows the neuralSpring
//! `BaselineProvenance` and ludoSpring `ValidationResult::with_provenance()`
//! patterns.
//!
//! # Usage
//!
//! Attach provenance to any result that carries expected/baseline values:
//!
//! ```
//! use universal_patterns::provenance::Provenance;
//!
//! let prov = Provenance::builder()
//!     .script("benchmarks/ai_latency.rs")
//!     .commit("a1b2c3d")
//!     .date("2026-03-14")
//!     .command("cargo bench --bench ai_latency")
//!     .environment("Linux 6.17 / Rust 1.93 / AMD EPYC 7763")
//!     .build();
//!
//! assert_eq!(prov.script(), Some("benchmarks/ai_latency.rs"));
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Records the origin of a validation baseline or benchmark result.
///
/// All fields are optional because provenance is accumulated incrementally:
/// automated scripts may set `script` + `commit`, while manual runs may
/// only set `command` + `date`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    script: Option<String>,
    commit: Option<String>,
    date: Option<String>,
    command: Option<String>,
    environment: Option<String>,
    timestamp: Option<DateTime<Utc>>,
}

impl Provenance {
    /// Start building a new provenance record.
    #[must_use]
    pub fn builder() -> ProvenanceBuilder {
        ProvenanceBuilder::default()
    }

    /// Create a provenance record capturing the current environment automatically.
    ///
    /// Populates `date` and `timestamp` from the system clock, and `environment`
    /// from `CARGO_PKG_VERSION` + `rustc --version` if available.
    #[must_use]
    pub fn auto() -> Self {
        let now = Utc::now();
        Self {
            date: Some(now.format("%Y-%m-%d").to_string()),
            timestamp: Some(now),
            environment: Some(format!(
                "{} / Rust {}",
                std::env::consts::OS,
                env!("CARGO_PKG_VERSION"),
            )),
            ..Default::default()
        }
    }

    /// The script or binary that generated the baseline.
    #[must_use]
    pub fn script(&self) -> Option<&str> {
        self.script.as_deref()
    }

    /// The git commit hash at generation time.
    #[must_use]
    pub fn commit(&self) -> Option<&str> {
        self.commit.as_deref()
    }

    /// The human-readable date of generation (ISO 8601).
    #[must_use]
    pub fn date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    /// The exact command invoked to produce the baseline.
    #[must_use]
    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }

    /// A description of the execution environment (OS, toolchain, hardware).
    #[must_use]
    pub fn environment(&self) -> Option<&str> {
        self.environment.as_deref()
    }

    /// Machine-generated UTC timestamp.
    #[must_use]
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.timestamp
    }

    /// Returns `true` if no provenance fields have been set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.script.is_none()
            && self.commit.is_none()
            && self.date.is_none()
            && self.command.is_none()
            && self.environment.is_none()
            && self.timestamp.is_none()
    }
}

impl std::fmt::Display for Provenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if let Some(s) = &self.script {
            parts.push(format!("script={s}"));
        }
        if let Some(c) = &self.commit {
            parts.push(format!("commit={c}"));
        }
        if let Some(d) = &self.date {
            parts.push(format!("date={d}"));
        }
        if let Some(cmd) = &self.command {
            parts.push(format!("command={cmd}"));
        }
        if let Some(env) = &self.environment {
            parts.push(format!("env={env}"));
        }
        if parts.is_empty() {
            write!(f, "Provenance(empty)")
        } else {
            write!(f, "Provenance({})", parts.join(", "))
        }
    }
}

/// Builder for [`Provenance`].
#[derive(Debug, Default)]
pub struct ProvenanceBuilder {
    inner: Provenance,
}

impl ProvenanceBuilder {
    /// Set the generating script path.
    #[must_use]
    pub fn script(mut self, script: &str) -> Self {
        self.inner.script = Some(script.to_owned());
        self
    }

    /// Set the git commit hash.
    #[must_use]
    pub fn commit(mut self, commit: &str) -> Self {
        self.inner.commit = Some(commit.to_owned());
        self
    }

    /// Set the generation date (ISO 8601 string).
    #[must_use]
    pub fn date(mut self, date: &str) -> Self {
        self.inner.date = Some(date.to_owned());
        self
    }

    /// Set the exact command used.
    #[must_use]
    pub fn command(mut self, command: &str) -> Self {
        self.inner.command = Some(command.to_owned());
        self
    }

    /// Set the execution environment description.
    #[must_use]
    pub fn environment(mut self, environment: &str) -> Self {
        self.inner.environment = Some(environment.to_owned());
        self
    }

    /// Set the UTC timestamp.
    #[must_use]
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.inner.timestamp = Some(timestamp);
        self
    }

    /// Consume the builder and produce the [`Provenance`].
    #[must_use]
    pub fn build(self) -> Provenance {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_all_fields() {
        let prov = Provenance::builder()
            .script("bench/ai.rs")
            .commit("abc1234")
            .date("2026-03-14")
            .command("cargo bench")
            .environment("Linux x86_64")
            .build();

        assert_eq!(prov.script(), Some("bench/ai.rs"));
        assert_eq!(prov.commit(), Some("abc1234"));
        assert_eq!(prov.date(), Some("2026-03-14"));
        assert_eq!(prov.command(), Some("cargo bench"));
        assert_eq!(prov.environment(), Some("Linux x86_64"));
        assert!(!prov.is_empty());
    }

    #[test]
    fn test_default_is_empty() {
        let prov = Provenance::default();
        assert!(prov.is_empty());
    }

    #[test]
    fn test_auto_populates_date_and_env() {
        let prov = Provenance::auto();
        assert!(prov.date().is_some());
        assert!(prov.environment().is_some());
        assert!(prov.timestamp().is_some());
        assert!(!prov.is_empty());
    }

    #[test]
    fn test_display_empty() {
        let prov = Provenance::default();
        assert_eq!(format!("{prov}"), "Provenance(empty)");
    }

    #[test]
    fn test_display_with_fields() {
        let prov = Provenance::builder()
            .script("test.rs")
            .commit("deadbeef")
            .build();
        let s = format!("{prov}");
        assert!(s.contains("script=test.rs"));
        assert!(s.contains("commit=deadbeef"));
    }

    #[test]
    fn test_serde_round_trip() {
        let prov = Provenance::builder()
            .script("bench/ai.rs")
            .commit("abc1234")
            .date("2026-03-14")
            .command("cargo bench")
            .environment("Linux x86_64")
            .build();

        let json = serde_json::to_string(&prov).expect("serialize");
        let restored: Provenance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(prov, restored);
    }

    #[test]
    fn test_partial_builder() {
        let prov = Provenance::builder().script("only-script.py").build();
        assert_eq!(prov.script(), Some("only-script.py"));
        assert!(prov.commit().is_none());
        assert!(prov.date().is_none());
        assert!(!prov.is_empty());
    }
}
