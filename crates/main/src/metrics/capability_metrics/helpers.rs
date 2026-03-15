// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Helper functions for metrics collection
//!
//! Provides utility functions for bucketing and categorizing metric data.

/// Get time bucket for histogram (milliseconds)
///
/// Categorizes timing measurements into buckets for histogram distribution.
///
/// # Buckets
/// - "0-10ms": 0-10 milliseconds
/// - "10-50ms": 10-50 milliseconds
/// - "50-100ms": 50-100 milliseconds
/// - "100-500ms": 100-500 milliseconds
/// - "500-1000ms": 500-1000 milliseconds
/// - "1000+ms": Over 1 second
pub fn get_time_bucket(time_ms: f64) -> String {
    if time_ms < 10.0 {
        "0-10ms".to_string()
    } else if time_ms < 50.0 {
        "10-50ms".to_string()
    } else if time_ms < 100.0 {
        "50-100ms".to_string()
    } else if time_ms < 500.0 {
        "100-500ms".to_string()
    } else if time_ms < 1000.0 {
        "500-1000ms".to_string()
    } else {
        "1000+ms".to_string()
    }
}

/// Get score bucket for distribution (0.0 to 1.0)
///
/// Categorizes scores into buckets for distribution analysis.
///
/// # Buckets
/// - "0.0-0.5": Low quality scores
/// - "0.5-0.7": Medium quality scores
/// - "0.7-0.8": Good quality scores
/// - "0.8-0.9": Very good quality scores
/// - "0.9-1.0": Excellent quality scores
pub fn get_score_bucket(score: f64) -> String {
    if score < 0.5 {
        "0.0-0.5".to_string()
    } else if score < 0.7 {
        "0.5-0.7".to_string()
    } else if score < 0.8 {
        "0.7-0.8".to_string()
    } else if score < 0.9 {
        "0.8-0.9".to_string()
    } else {
        "0.9-1.0".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_time_bucket() {
        assert_eq!(get_time_bucket(5.0), "0-10ms");
        assert_eq!(get_time_bucket(25.0), "10-50ms");
        assert_eq!(get_time_bucket(75.0), "50-100ms");
        assert_eq!(get_time_bucket(250.0), "100-500ms");
        assert_eq!(get_time_bucket(750.0), "500-1000ms");
        assert_eq!(get_time_bucket(1500.0), "1000+ms");
    }

    #[test]
    fn test_get_time_bucket_boundaries() {
        assert_eq!(get_time_bucket(0.0), "0-10ms");
        assert_eq!(get_time_bucket(10.0), "10-50ms");
        assert_eq!(get_time_bucket(50.0), "50-100ms");
        assert_eq!(get_time_bucket(100.0), "100-500ms");
        assert_eq!(get_time_bucket(500.0), "500-1000ms");
        assert_eq!(get_time_bucket(1000.0), "1000+ms");
    }

    #[test]
    fn test_get_score_bucket() {
        assert_eq!(get_score_bucket(0.3), "0.0-0.5");
        assert_eq!(get_score_bucket(0.6), "0.5-0.7");
        assert_eq!(get_score_bucket(0.75), "0.7-0.8");
        assert_eq!(get_score_bucket(0.85), "0.8-0.9");
        assert_eq!(get_score_bucket(0.95), "0.9-1.0");
    }

    #[test]
    fn test_get_score_bucket_boundaries() {
        assert_eq!(get_score_bucket(0.0), "0.0-0.5");
        assert_eq!(get_score_bucket(0.5), "0.5-0.7");
        assert_eq!(get_score_bucket(0.7), "0.7-0.8");
        assert_eq!(get_score_bucket(0.8), "0.8-0.9");
        assert_eq!(get_score_bucket(0.9), "0.9-1.0");
        assert_eq!(get_score_bucket(1.0), "0.9-1.0");
    }
}
