// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! NDJSON streaming types for pipeline coordination.
//!
//! Pattern absorbed from rhizoCrypt v0.13: structured streaming items
//! enable cross-primal pipeline coordination via newline-delimited JSON.
//!
//! Each [`StreamItem`] is a self-describing envelope that carries typed
//! payloads (data, progress, error, completion) so consumers can react
//! without parsing the inner payload first.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single item in an NDJSON stream.
///
/// Primals emit these line-by-line. Consumers match on `kind` to decide
/// how to handle each line without parsing `payload` first.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamItem {
    /// Discriminator for the item type.
    pub kind: StreamKind,
    /// Monotonically increasing sequence number (0-based).
    pub seq: u64,
    /// Optional correlation ID linking items in a pipeline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// The typed payload.
    pub payload: Value,
}

/// Discriminator for [`StreamItem`] kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamKind {
    /// A data record (the primary payload).
    Data,
    /// Progress update (e.g. "50% complete").
    Progress,
    /// Recoverable error (stream continues).
    Error,
    /// Terminal: stream is complete.
    Done,
    /// Heartbeat / keep-alive.
    Heartbeat,
}

impl StreamItem {
    /// Create a data item.
    #[must_use]
    pub fn data(seq: u64, payload: Value) -> Self {
        Self {
            kind: StreamKind::Data,
            seq,
            correlation_id: None,
            payload,
        }
    }

    /// Create a progress item.
    #[must_use]
    pub fn progress(seq: u64, percent: f64, message: &str) -> Self {
        Self {
            kind: StreamKind::Progress,
            seq,
            correlation_id: None,
            payload: serde_json::json!({
                "percent": percent,
                "message": message,
            }),
        }
    }

    /// Create a done sentinel.
    #[must_use]
    pub fn done(seq: u64) -> Self {
        Self {
            kind: StreamKind::Done,
            seq,
            correlation_id: None,
            payload: Value::Null,
        }
    }

    /// Create an error item (stream continues).
    #[must_use]
    pub fn error(seq: u64, message: &str) -> Self {
        Self {
            kind: StreamKind::Error,
            seq,
            correlation_id: None,
            payload: serde_json::json!({ "error": message }),
        }
    }

    /// Create a heartbeat.
    #[must_use]
    pub fn heartbeat(seq: u64) -> Self {
        Self {
            kind: StreamKind::Heartbeat,
            seq,
            correlation_id: None,
            payload: Value::Null,
        }
    }

    /// Attach a correlation ID.
    #[must_use]
    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Serialize to a single NDJSON line (no trailing newline).
    ///
    /// # Errors
    ///
    /// Returns serialization error if the payload cannot be converted to JSON.
    pub fn to_ndjson_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse from a single NDJSON line.
    ///
    /// # Errors
    ///
    /// Returns deserialization error if the line is not a valid `StreamItem`.
    pub fn from_ndjson_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_item_roundtrip() {
        let item = StreamItem::data(0, serde_json::json!({"key": "value"}));
        let line = item.to_ndjson_line().expect("should succeed");
        let parsed = StreamItem::from_ndjson_line(&line).expect("should succeed");
        assert_eq!(parsed, item);
        assert_eq!(parsed.kind, StreamKind::Data);
        assert_eq!(parsed.seq, 0);
    }

    #[test]
    fn progress_item() {
        let item = StreamItem::progress(1, 50.0, "halfway");
        assert_eq!(item.kind, StreamKind::Progress);
        assert_eq!(item.payload["percent"], 50.0);
    }

    #[test]
    fn done_sentinel() {
        let item = StreamItem::done(99);
        assert_eq!(item.kind, StreamKind::Done);
        assert_eq!(item.payload, Value::Null);
    }

    #[test]
    fn error_item() {
        let item = StreamItem::error(5, "disk full");
        assert_eq!(item.kind, StreamKind::Error);
        assert!(
            item.payload["error"]
                .as_str()
                .expect("should succeed")
                .contains("disk full")
        );
    }

    #[test]
    fn correlation_id() {
        let item = StreamItem::data(0, Value::Null).with_correlation("pipeline-42");
        assert_eq!(item.correlation_id.as_deref(), Some("pipeline-42"));
    }

    #[test]
    fn heartbeat() {
        let item = StreamItem::heartbeat(10);
        assert_eq!(item.kind, StreamKind::Heartbeat);
    }

    #[test]
    fn ndjson_line_no_trailing_newline() {
        let item = StreamItem::data(0, serde_json::json!(1));
        let line = item.to_ndjson_line().expect("should succeed");
        assert!(!line.ends_with('\n'));
    }

    #[test]
    fn skip_none_correlation_id() {
        let item = StreamItem::data(0, Value::Null);
        let json = serde_json::to_string(&item).expect("should succeed");
        assert!(!json.contains("correlation_id"));
    }
}
