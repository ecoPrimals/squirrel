// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal lifecycle coordination with observer notifications.
//!
//! Used to broadcast phase changes (for example to monitoring or federation subsystems) when this
//! primal moves between initialization, ready, degraded, and shutdown states.

use parking_lot::RwLock;
use std::fmt;
use std::sync::Arc;

use crate::{Error, Result};

/// High-level lifecycle phases for a primal instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimalLifecyclePhase {
    /// Starting up; not yet accepting full work.
    Initializing,
    /// Normal operation.
    Ready,
    /// Operating with reduced capacity or partial failures.
    Degraded,
    /// Draining and stopping.
    ShuttingDown,
}

/// Observer notified when the lifecycle phase changes.
pub trait LifecycleObserver: Send + Sync {
    /// Called after a successful transition (including no-op when `previous == current`).
    fn on_phase_changed(&self, previous: PrimalLifecyclePhase, current: PrimalLifecyclePhase);
}

/// Coordinates primal lifecycle state and notifies registered observers.
pub struct CoordinationService {
    phase: Arc<RwLock<PrimalLifecyclePhase>>,
    observers: Arc<RwLock<Vec<Arc<dyn LifecycleObserver>>>>,
}

impl fmt::Debug for CoordinationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoordinationService")
            .field("phase", &self.current_phase())
            .finish_non_exhaustive()
    }
}

impl Default for CoordinationService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CoordinationService {
    fn clone(&self) -> Self {
        Self {
            phase: Arc::clone(&self.phase),
            observers: Arc::clone(&self.observers),
        }
    }
}

impl CoordinationService {
    /// Creates a service starting in [`PrimalLifecyclePhase::Initializing`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            phase: Arc::new(RwLock::new(PrimalLifecyclePhase::Initializing)),
            observers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Current lifecycle phase.
    #[must_use]
    pub fn current_phase(&self) -> PrimalLifecyclePhase {
        *self.phase.read()
    }

    /// Registers an observer. Observers are invoked in registration order.
    pub fn add_observer(&self, observer: Arc<dyn LifecycleObserver>) {
        self.observers.write().push(observer);
    }

    /// Attempts a validated transition. On success, notifies all observers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Coordination`] when `next` is not reachable from the current phase.
    pub fn transition_to(&self, next: PrimalLifecyclePhase) -> Result<()> {
        let mut guard = self.phase.write();
        let previous = *guard;
        if previous == next {
            self.notify(previous, next);
            return Ok(());
        }
        if !Self::is_allowed(previous, next) {
            return Err(Error::Coordination(format!(
                "invalid lifecycle transition: {previous:?} -> {next:?}"
            )));
        }
        *guard = next;
        drop(guard);
        self.notify(previous, next);
        Ok(())
    }

    /// Force-set phase without validation (for example test hooks). Prefer [`Self::transition_to`]
    /// in production paths.
    pub fn set_phase_unchecked(&self, next: PrimalLifecyclePhase) {
        let mut guard = self.phase.write();
        let previous = *guard;
        *guard = next;
        drop(guard);
        self.notify(previous, next);
    }

    fn notify(&self, previous: PrimalLifecyclePhase, current: PrimalLifecyclePhase) {
        let observers = self.observers.read();
        for o in observers.iter() {
            o.on_phase_changed(previous, current);
        }
    }

    #[must_use]
    const fn is_allowed(from: PrimalLifecyclePhase, to: PrimalLifecyclePhase) -> bool {
        use PrimalLifecyclePhase::{Degraded, Initializing, Ready, ShuttingDown};
        matches!(
            (from, to),
            (Initializing, Ready | Degraded | ShuttingDown)
                | (Ready, Degraded | ShuttingDown)
                | (Degraded, Ready | ShuttingDown)
        )
    }
}
