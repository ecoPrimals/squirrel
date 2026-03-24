// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service Level Objective (SLO) constants for Squirrel AI operations.
//!
//! Centralizes all performance, cost, and quality thresholds used in AI
//! request processing, benchmarking, and health monitoring. Follows the
//! neuralSpring `tolerances/mod.rs` pattern: named constants with doc
//! justification and provenance, no magic numbers in application code.
//!
//! # Categories
//!
//! | Category | Purpose | Example |
//! |----------|---------|---------|
//! | Latency | Response-time budgets | P95 AI query < 30s |
//! | Cost | Per-request spend limits | < $0.10 per query |
//! | Quality | Output quality floors | Relevance score > 0.7 |
//! | Throughput | Capacity targets | > 10 queries/sec |
//! | Availability | Uptime targets | 99.9% provider availability |

use std::time::Duration;

// ============================================================================
// AI Query Latency SLOs
// ============================================================================

/// P50 latency target for AI text generation queries.
///
/// Median response time for a standard AI query (single-turn, < 4K tokens).
/// Based on observed OpenAI/Anthropic API latencies under normal load.
pub const AI_QUERY_P50_LATENCY: Duration = Duration::from_secs(5);

/// P95 latency target for AI text generation queries.
///
/// 95th-percentile response time. Accounts for occasional provider
/// congestion and retry overhead. Exceeding this triggers a Yellow
/// health status in monitoring.
pub const AI_QUERY_P95_LATENCY: Duration = Duration::from_secs(30);

/// P99 latency target for AI text generation queries.
///
/// 99th-percentile latency budget. Exceeding this triggers provider
/// failover and a Red health alert. Includes full retry chain.
pub const AI_QUERY_P99_LATENCY: Duration = Duration::from_secs(60);

/// Maximum wall-clock time before an AI query is considered timed out.
///
/// Hard cutoff. After this duration the request is cancelled regardless
/// of provider state. Prevents resource exhaustion from hung connections.
pub const AI_QUERY_HARD_TIMEOUT: Duration = Duration::from_secs(120);

/// Latency target for tool orchestration (non-AI compute).
///
/// Tool calls that don't involve external AI providers should complete
/// within this budget. Covers local file ops, config lookups, and
/// in-process tool execution.
pub const TOOL_EXECUTION_LATENCY: Duration = Duration::from_secs(5);

/// Maximum acceptable latency for capability discovery round-trip.
///
/// JSON-RPC `capability.discover` must complete within this budget to
/// prevent cascade delays during service startup.
pub const DISCOVERY_LATENCY: Duration = Duration::from_millis(500);

/// Maximum acceptable latency for health check probes.
///
/// Health checks must respond faster than this to be counted as "up".
/// Aligns with the ecosystem `heartbeat_interval / 3` rule.
pub const HEALTH_CHECK_LATENCY: Duration = Duration::from_secs(10);

// ============================================================================
// AI Cost SLOs (USD per unit)
// ============================================================================

/// Maximum acceptable cost per AI query (USD).
///
/// A single user-facing AI query (including retries) should not exceed
/// this cost. Covers prompt + completion tokens at current tier pricing.
/// Exceeding this triggers cost-alert logging.
pub const AI_QUERY_MAX_COST_USD: f64 = 0.10;

/// Cost warning threshold per AI query (USD).
///
/// When a single query exceeds this cost, a warning is logged for
/// cost monitoring dashboards. Set at 50% of the hard limit.
pub const AI_QUERY_COST_WARNING_USD: f64 = 0.05;

/// Maximum daily AI spend per user (USD).
///
/// Aggregate daily cost cap per user identity. Prevents runaway
/// cost from loops or high-volume automated usage.
pub const AI_DAILY_USER_SPEND_CAP_USD: f64 = 10.0;

/// Maximum cost per 1K input tokens (USD).
///
/// Baseline for input token pricing across providers. Used to
/// estimate request cost before submission.
pub const AI_INPUT_COST_PER_1K_TOKENS: f64 = 0.003;

/// Maximum cost per 1K output tokens (USD).
///
/// Baseline for output token pricing. Output tokens are typically
/// 3-5x more expensive than input tokens.
pub const AI_OUTPUT_COST_PER_1K_TOKENS: f64 = 0.015;

// ============================================================================
// AI Quality SLOs
// ============================================================================

/// Minimum relevance score for AI responses (0.0 – 1.0).
///
/// Responses scoring below this threshold are flagged for review.
/// Measured by the context-relevance evaluator against the original query.
pub const AI_RESPONSE_RELEVANCE_MIN: f64 = 0.70;

/// Minimum coherence score for AI responses (0.0 – 1.0).
///
/// Responses must be internally consistent. Below this threshold the
/// response is rejected and retried with a different provider/model.
pub const AI_RESPONSE_COHERENCE_MIN: f64 = 0.80;

/// Maximum hallucination rate (0.0 – 1.0).
///
/// Fraction of claims in the response that are unsupported by context.
/// Above this threshold, the response is flagged for human review.
pub const AI_HALLUCINATION_RATE_MAX: f64 = 0.10;

/// Minimum success rate for AI provider requests (0.0 – 1.0).
///
/// Provider availability measured over a 5-minute rolling window.
/// Below this threshold the provider is marked degraded and traffic
/// is shifted to alternatives.
pub const AI_PROVIDER_SUCCESS_RATE_MIN: f64 = 0.95;

// ============================================================================
// Throughput SLOs
// ============================================================================

/// Minimum sustained queries per second for the AI subsystem.
///
/// At steady state, the system must handle at least this many concurrent
/// AI queries. Below this, capacity alerts fire.
pub const AI_MIN_THROUGHPUT_QPS: f64 = 10.0;

/// Maximum concurrent AI requests in flight.
///
/// Backpressure engages above this count. Prevents overwhelming the
/// provider API and exhausting local connection pools.
pub const AI_MAX_CONCURRENT_REQUESTS: u32 = 50;

/// Maximum pending requests in the AI queue before shedding load.
///
/// When the queue depth exceeds this, new requests receive a 429
/// (Too Many Requests) response. Prevents unbounded memory growth.
pub const AI_QUEUE_DEPTH_MAX: u32 = 200;

// ============================================================================
// Availability SLOs
// ============================================================================

/// Target uptime for the Squirrel AI service (fraction, 0.0 – 1.0).
///
/// 99.9% = 8.76 hours of downtime per year. Measured over 30-day
/// rolling windows.
pub const SERVICE_AVAILABILITY_TARGET: f64 = 0.999;

/// Minimum number of healthy AI providers to remain operational.
///
/// Below this threshold the service enters degraded mode and raises
/// a critical alert. At zero healthy providers, the service is down.
pub const MIN_HEALTHY_PROVIDERS: u32 = 1;

/// Maximum provider failover time.
///
/// When the primary provider fails, traffic must shift to a backup
/// within this budget. Covers detection + DNS/connection setup.
pub const PROVIDER_FAILOVER_BUDGET: Duration = Duration::from_secs(5);

// ============================================================================
// Benchmark SLOs
// ============================================================================

/// Minimum operations per second for JSON-RPC serialization benchmarks.
///
/// The JSON-RPC request/response serialization hot path must sustain
/// at least this throughput on a single core.
pub const BENCHMARK_JSONRPC_MIN_OPS: f64 = 10_000.0;

/// Maximum memory growth during a benchmark suite run (MB).
///
/// If memory grows by more than this during the full benchmark suite,
/// a leak is suspected. Based on observed steady-state RSS.
pub const BENCHMARK_MAX_MEMORY_GROWTH_MB: f64 = 100.0;

/// Minimum benchmark success rate (fraction).
///
/// All benchmark operations must succeed at this rate. Below this
/// threshold the benchmark suite report is flagged as unreliable.
pub const BENCHMARK_SUCCESS_RATE_MIN: f64 = 0.99;

// ============================================================================
// Helper: SLO metadata for introspection
// ============================================================================

/// Named SLO entry for runtime introspection and reporting.
#[derive(Debug, Clone)]
pub struct NamedSlo {
    /// Machine-readable identifier (e.g. `"ai_query_p95_latency"`).
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// SLO category.
    pub category: SloCategory,
}

/// SLO category for grouping in dashboards and reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SloCategory {
    /// Response-time budgets.
    Latency,
    /// Per-request and aggregate cost limits.
    Cost,
    /// Output quality floors.
    Quality,
    /// Capacity targets.
    Throughput,
    /// Uptime and failover targets.
    Availability,
    /// Benchmark performance gates.
    Benchmark,
}

/// All registered SLOs for runtime introspection.
///
/// Enables dashboards and monitoring to enumerate every SLO
/// without hard-coding names elsewhere.
#[must_use]
pub fn all_slos() -> Vec<NamedSlo> {
    vec![
        NamedSlo {
            name: "ai_query_p50_latency",
            description: "P50 AI query latency (5s)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "ai_query_p95_latency",
            description: "P95 AI query latency (30s)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "ai_query_p99_latency",
            description: "P99 AI query latency (60s)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "ai_query_hard_timeout",
            description: "AI query hard timeout (120s)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "tool_execution_latency",
            description: "Tool execution latency (5s)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "discovery_latency",
            description: "Capability discovery latency (500ms)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "health_check_latency",
            description: "Health check latency (10s)",
            category: SloCategory::Latency,
        },
        NamedSlo {
            name: "ai_query_max_cost_usd",
            description: "Max cost per AI query ($0.10)",
            category: SloCategory::Cost,
        },
        NamedSlo {
            name: "ai_query_cost_warning_usd",
            description: "Cost warning per AI query ($0.05)",
            category: SloCategory::Cost,
        },
        NamedSlo {
            name: "ai_daily_user_spend_cap_usd",
            description: "Daily user spend cap ($10)",
            category: SloCategory::Cost,
        },
        NamedSlo {
            name: "ai_response_relevance_min",
            description: "Min AI response relevance (0.70)",
            category: SloCategory::Quality,
        },
        NamedSlo {
            name: "ai_response_coherence_min",
            description: "Min AI response coherence (0.80)",
            category: SloCategory::Quality,
        },
        NamedSlo {
            name: "ai_hallucination_rate_max",
            description: "Max hallucination rate (0.10)",
            category: SloCategory::Quality,
        },
        NamedSlo {
            name: "ai_provider_success_rate_min",
            description: "Min provider success rate (0.95)",
            category: SloCategory::Quality,
        },
        NamedSlo {
            name: "ai_min_throughput_qps",
            description: "Min AI throughput (10 QPS)",
            category: SloCategory::Throughput,
        },
        NamedSlo {
            name: "service_availability_target",
            description: "Service availability (99.9%)",
            category: SloCategory::Availability,
        },
        NamedSlo {
            name: "benchmark_jsonrpc_min_ops",
            description: "Min JSON-RPC bench ops (10K/s)",
            category: SloCategory::Benchmark,
        },
        NamedSlo {
            name: "benchmark_success_rate_min",
            description: "Min benchmark success rate (0.99)",
            category: SloCategory::Benchmark,
        },
    ]
}

/// Return all SLOs in a given category.
#[must_use]
pub fn slos_by_category(category: SloCategory) -> Vec<NamedSlo> {
    all_slos()
        .into_iter()
        .filter(|s| s.category == category)
        .collect()
}

#[cfg(test)]
#[allow(
    clippy::assertions_on_constants,
    reason = "validating SLO constant relationships"
)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_slo_ordering() {
        assert!(AI_QUERY_P50_LATENCY < AI_QUERY_P95_LATENCY);
        assert!(AI_QUERY_P95_LATENCY < AI_QUERY_P99_LATENCY);
        assert!(AI_QUERY_P99_LATENCY < AI_QUERY_HARD_TIMEOUT);
    }

    #[test]
    fn test_cost_slo_ordering() {
        assert!(AI_QUERY_COST_WARNING_USD < AI_QUERY_MAX_COST_USD);
        assert!(AI_QUERY_MAX_COST_USD < AI_DAILY_USER_SPEND_CAP_USD);
    }

    #[test]
    fn test_quality_slos_in_valid_range() {
        assert!((0.0..=1.0).contains(&AI_RESPONSE_RELEVANCE_MIN));
        assert!((0.0..=1.0).contains(&AI_RESPONSE_COHERENCE_MIN));
        assert!((0.0..=1.0).contains(&AI_HALLUCINATION_RATE_MAX));
        assert!((0.0..=1.0).contains(&AI_PROVIDER_SUCCESS_RATE_MIN));
    }

    #[test]
    fn test_throughput_slos_positive() {
        assert!(AI_MIN_THROUGHPUT_QPS > 0.0);
        assert!(AI_MAX_CONCURRENT_REQUESTS > 0);
        assert!(AI_QUEUE_DEPTH_MAX > AI_MAX_CONCURRENT_REQUESTS);
    }

    #[test]
    fn test_availability_slos() {
        assert!((0.0..=1.0).contains(&SERVICE_AVAILABILITY_TARGET));
        assert!(MIN_HEALTHY_PROVIDERS >= 1);
        assert!(PROVIDER_FAILOVER_BUDGET <= AI_QUERY_P50_LATENCY);
    }

    #[test]
    fn test_benchmark_slos() {
        assert!(BENCHMARK_JSONRPC_MIN_OPS > 0.0);
        assert!(BENCHMARK_MAX_MEMORY_GROWTH_MB > 0.0);
        assert!((0.0..=1.0).contains(&BENCHMARK_SUCCESS_RATE_MIN));
    }

    #[test]
    fn test_all_slos_non_empty() {
        let slos = all_slos();
        assert!(!slos.is_empty());
        assert!(slos.len() >= 18);
    }

    #[test]
    fn test_slos_by_category_filters_correctly() {
        let latency_slos = slos_by_category(SloCategory::Latency);
        assert!(latency_slos.len() >= 7);
        for slo in &latency_slos {
            assert_eq!(slo.category, SloCategory::Latency);
        }

        let cost_slos = slos_by_category(SloCategory::Cost);
        assert!(cost_slos.len() >= 3);
        for slo in &cost_slos {
            assert_eq!(slo.category, SloCategory::Cost);
        }
    }

    #[test]
    fn test_input_output_token_cost_relationship() {
        assert!(AI_OUTPUT_COST_PER_1K_TOKENS > AI_INPUT_COST_PER_1K_TOKENS);
    }
}
