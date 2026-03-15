// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Active Span Management
//!
//! This module provides the ActiveSpan wrapper that manages the lifecycle
//! of spans and ensures proper cleanup when spans are dropped.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::types::Span;

/// Thread-local storage for the current span
thread_local! {
    static CURRENT_SPAN: std::cell::RefCell<Option<Arc<Mutex<ActiveSpan>>>> = std::cell::RefCell::new(None);
}

/// A wrapper around a span that manages its active lifecycle
#[derive(Debug)]
pub struct ActiveSpan {
    /// The span being traced
    span: Span,
    /// Whether the span has been ended
    ended: bool,
}

impl ActiveSpan {
    /// Create a new active span
    pub fn new(span: Span) -> Self {
        Self {
            span,
            ended: false,
        }
    }

    /// Get a reference to the span
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// Get a mutable reference to the span
    pub fn span_mut(&mut self) -> &mut Span {
        &mut self.span
    }

    /// End the span and mark it as successful
    pub fn end(mut self) {
        if !self.ended {
            self.span.end();
            self.ended = true;
        }
    }

    /// End the span and mark it as error
    pub fn end_with_error(mut self) {
        if !self.ended {
            self.span.end_with_error();
            self.ended = true;
        }
    }

    /// Add an attribute to the span
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.span.add_attribute(key, value);
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.span.add_event(name, attributes);
    }

    /// Check if the span has been ended
    pub fn is_ended(&self) -> bool {
        self.ended
    }
}

impl Drop for ActiveSpan {
    fn drop(&mut self) {
        if !self.ended {
            // Auto-end the span when dropped
            self.span.end();
            self.ended = true;
        }
    }
}

/// Set the current span for this thread
pub fn set_current_span(span: Option<Arc<Mutex<ActiveSpan>>>) {
    CURRENT_SPAN.with(|current| {
        *current.borrow_mut() = span;
    });
}

/// Get the current span for this thread
pub fn current_span() -> Option<Arc<Mutex<ActiveSpan>>> {
    CURRENT_SPAN.with(|current| current.borrow().clone())
} 