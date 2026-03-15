// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use thiserror::Error;

/// Error related to the alert system
///
/// Represents errors that occur within the alert processing system,
/// including notification failures, alert validation errors, and
/// alert delivery issues.
#[derive(Debug, Clone, Error)]
pub enum AlertError {
    /// Error that occurs when a notification fails to be sent
    #[error("Notification failed: {0}")]
    NotificationFailed(String),

    /// Error that occurs when an alert validation fails
    #[error("Alert validation failed: {0}")]
    ValidationFailed(String),

    /// Error that occurs when an alert delivery fails
    #[error("Alert delivery failed: {0}")]
    DeliveryFailed(String),

    /// Error that occurs when an alert processing fails
    #[error("Alert processing failed: {0}")]
    ProcessingFailed(String),

    /// Error that occurs when an alert is not found
    #[error("Alert not found: {0}")]
    NotFound(String),

    /// Error that occurs when an alert is already processed
    #[error("Alert already processed: {0}")]
    AlreadyProcessed(String),

    /// Error that occurs when an alert is not authorized
    #[error("Alert not authorized: {0}")]
    NotAuthorized(String),
}
