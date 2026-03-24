// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Lossy `f64` conversions for metrics, ratios, and routing scores.
//!
//! Clippy flags `as f64` from wide integers; these helpers centralize that with a single,
//! documented expectation per use case.

#![expect(
    clippy::missing_const_for_fn,
    reason = "Functions contain #[expect] for cast_precision_loss; cannot be const"
)]

/// Ratio `numer / denom` for statistics; returns `0.0` when `denom == 0`.
#[inline]
pub fn u64_ratio(numer: u64, denom: u64) -> f64 {
    if denom == 0 {
        return 0.0;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "Request/token counts; ratio is for display and routing heuristics only"
    )]
    let r = numer as f64 / denom as f64;
    r
}

/// Convert a dimension (e.g. context length) to `f64` for scoring heuristics.
#[inline]
pub fn usize_to_f64_lossy(x: usize) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "Context sizes in scoring; typical values fit f64 mantissa for ratios"
    )]
    let v = x as f64;
    v
}

/// Convert accumulated latency in milliseconds (may be `u128` from `Duration::as_millis`).
#[inline]
pub fn u128_to_f64_lossy(x: u128) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "Elapsed ms for rolling average; sub-ms precision not required"
    )]
    let v = x as f64;
    v
}

/// Convert provider throughput or similar `u64` metrics to `f64`.
#[inline]
pub fn u64_to_f64_lossy(x: u64) -> f64 {
    #[expect(
        clippy::cast_precision_loss,
        reason = "Latency/throughput metrics for normalized scores"
    )]
    let v = x as f64;
    v
}
