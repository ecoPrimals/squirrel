//! Alert notification delivery system
//! 
//! This module provides functionality for:
//! - Email notifications
//! - Slack notifications
//! - Webhook notifications
//! - Custom notification channels
//! - Notification routing
//! - Rate limiting

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, SystemTime};
use std::format as fmt;
use thiserror::Error;
use time::OffsetDateTime;
use handlebars::Handlebars;
use serde::{Serialize, Deserialize};
use serde_json::json;
use async_trait::async_trait;

use super::{AlertNotification, AlertSeverity};
use super::adapter::{NotificationManagerAdapter, create_notification_manager_adapter_with_manager};

/// Notification channel type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// Email notification
    Email {
        /// SMTP server
        smtp_server: String,
        /// SMTP port
        smtp_port: u16,
        /// SMTP username
        smtp_username: String,
        /// SMTP password
        smtp_password: String,
        /// From address
        from_address: String,
        /// To addresses
        to_addresses: Vec<String>,
    },
    /// Slack notification
    Slack {
        /// Webhook URL
        webhook_url: String,
        /// Channel name
        channel: String,
        /// Bot username
        username: String,
    },
    /// Generic webhook
    Webhook {
        /// Webhook URL
        url: String,
        /// HTTP method
        method: String,
        /// Custom headers
        headers: std::collections::HashMap<String, String>,
    },
}

/// Notification template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template content
    pub content: String,
    /// Template format (text, html, json, etc.)
    pub format: String,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
    /// Rate limit in seconds
    pub rate_limit: u64,
    /// Templates
    pub templates: Vec<NotificationTemplate>,
    /// Routing rules
    pub routing_rules: Vec<RoutingRule>,
}

/// Notification routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Severity filter
    pub severity: Option<AlertSeverity>,
    /// Component filter
    pub component: Option<String>,
    /// Target channels
    pub channels: Vec<String>,
}

/// Errors that can occur during alert notification delivery.
/// 
/// This enum represents various error conditions that may arise when
/// attempting to deliver alert notifications through different channels.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Error related to notification channel configuration or operation.
    #[error("Channel error: {0}")]
    ChannelError(String),
    /// Error related to notification template processing.
    #[error("Template error: {0}")]
    TemplateError(String),
    /// Error due to rate limiting restrictions.
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    /// Error related to notification routing.
    #[error("Routing error: {0}")]
    RoutingError(String),
    /// System-level error during notification processing.
    #[error("System error: {0}")]
    SystemError(String),
    /// Error in notification configuration.
    #[error("Config error: {0}")]
    ConfigError(String),
}

/// Manager for handling alert notifications and their delivery.
/// 
/// This struct is responsible for managing notification templates,
/// routing rules, rate limiting, and delivery through various channels.
#[derive(Debug)]
pub struct NotificationManager {
    /// Configuration for notification delivery.
    config: Arc<RwLock<NotificationConfig>>,
    /// Template engine for rendering notifications.
    templates: Arc<RwLock<Handlebars<'static>>>,
    /// HTTP client for webhook notifications.
    client: reqwest::Client,
    /// Rate limiter for notification channels.
    rate_limiter: Arc<RwLock<std::collections::HashMap<String, OffsetDateTime>>>,
}

/// Parameters for sending email notifications.
/// 
/// This struct contains all the necessary information for sending
/// an alert notification via email.
#[derive(Debug)]
#[allow(dead_code)]
struct EmailParams<'a> {
    /// SMTP server address.
    smtp_server: &'a str,
    /// SMTP port number.
    smtp_port: u16,
    /// SMTP username for authentication.
    username: &'a str,
    /// SMTP password for authentication.
    password: &'a str,
    /// Sender email address.
    from: &'a str,
    /// List of recipient email addresses.
    to: &'a [String],
    /// Alert notification to be sent.
    alert: &'a AlertNotification,
}

impl NotificationManager {
    /// Creates a new notification manager with the given configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - The notification configuration to use
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the new manager if successful, or an error
    /// if the configuration is invalid or channels cannot be initialized.
    pub fn new(config: NotificationConfig) -> Result<Self, NotificationError> {
        let mut handlebars = Handlebars::new();
        
        // Register templates
        for template in &config.templates {
            handlebars
                .register_template_string(&template.id, &template.content)
                .map_err(|e| NotificationError::TemplateError(e.to_string()))?;
        }

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            templates: Arc::new(RwLock::new(handlebars)),
            client: reqwest::Client::new(),
            rate_limiter: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }

    /// Sends a notification for the given alert.
    /// 
    /// This method handles routing the alert to appropriate channels based on
    /// routing rules, applying rate limiting, and delivering the notification.
    /// 
    /// # Arguments
    /// 
    /// * `alert` - The alert to send a notification for
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` indicating whether the notification was sent successfully.
    pub async fn send_notification(&self, alert: &AlertNotification) -> Result<(), NotificationError> {
        let config = self.config.read().await;

        // Find matching routing rules
        let matching_rules: Vec<&RoutingRule> = config.routing_rules.iter()
            .filter(|rule| Self::check_routing_rule(rule, alert))
            .collect();

        // Send to each matching channel
        for rule in matching_rules {
            for channel_id in &rule.channels {
                // Check rate limit
                if !self.check_rate_limit(channel_id).await? {
                    continue;
                }

                // Find channel configuration
                if let Some(channel) = config.channels.iter().find(|c| match c {
                    NotificationChannel::Email { .. } => channel_id == "email",
                    NotificationChannel::Slack { .. } => channel_id == "slack",
                    NotificationChannel::Webhook { .. } => channel_id == "webhook",
                }) {
                    // Send notification through channel
                    self.send_through_channel(channel, alert).await?;
                }
            }
        }

        Ok(())
    }

    /// Send notification through a specific channel
    async fn send_through_channel(
        &self,
        channel: &NotificationChannel,
        alert: &AlertNotification,
    ) -> Result<(), NotificationError> {
        match channel {
            NotificationChannel::Email {
                smtp_server,
                smtp_port,
                smtp_username,
                smtp_password,
                from_address,
                to_addresses,
            } => {
                let params = EmailParams {
                    smtp_server,
                    smtp_port: *smtp_port,
                    username: smtp_username,
                    password: smtp_password,
                    from: from_address,
                    to: to_addresses,
                    alert,
                };
                self.send_email(params).await
            }
            NotificationChannel::Slack { webhook_url, channel, username } => {
                self.send_slack(webhook_url, channel, username, alert).await
            }
            NotificationChannel::Webhook { url, method, headers } => {
                self.send_webhook(url, method, headers, alert).await
            }
        }
    }

    /// Send email notification
    async fn send_email(
        &self,
        params: EmailParams<'_>,
    ) -> Result<(), NotificationError> {
        // TODO: Implement email sending using lettre or similar
        // For now, just log the attempt
        tracing::info!(
            "Would send email notification to {:?} for alert: {}",
            params.to,
            params.alert.message
        );
        Ok(())
    }

    /// Send Slack notification
    async fn send_slack(
        &self,
        webhook_url: &str,
        channel: &str,
        username: &str,
        alert: &AlertNotification,
    ) -> Result<(), NotificationError> {
        // Prepare slack message
        let color = get_severity_color(&alert.severity);
        
        let payload = serde_json::json!({
            "channel": channel,
            "username": username,
            "icon_emoji": ":warning:",
            "attachments": [
                {
                    "fallback": format!("Alert: {}", alert.message),
                    "color": color,
                    "title": format!("Alert: {}", alert.message),
                    "text": alert.description.clone(),
                    "fields": [
                        {
                            "title": "Severity",
                            "value": format!("{:?}", alert.severity),
                            "short": true
                        },
                        {
                            "title": "Component",
                            "value": alert.component.clone(),
                            "short": true
                        },
                        {
                            "title": "Time",
                            "value": alert.updated_at.to_string(),
                            "short": true
                        }
                    ],
                    "footer": "Squirrel Monitoring Service",
                    "ts": alert.updated_at
                }
            ]
        });

        // Send to Slack
        let client = &self.client;
        let res = client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::ChannelError(format!("Slack request error: {e}")))?;

        if !res.status().is_success() {
            return Err(NotificationError::ChannelError(
                format!("Slack returned error: {}", res.status())
            ));
        }

        Ok(())
    }

    /// Send webhook notification
    async fn send_webhook(
        &self,
        url: &str,
        _method: &str,
        headers: &std::collections::HashMap<String, String>,
        alert: &AlertNotification,
    ) -> Result<(), NotificationError> {
        // Prepare payload
        let payload = serde_json::json!({
            "alert": {
                "message": alert.message,
                "details": alert.description.clone(),
                "severity": format!("{:?}", alert.severity),
                "component": alert.component,
                "timestamp": alert.updated_at,
            }
        });

        // Create request
        let client = &self.client;
        let mut request = client.post(url); // Default to POST

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Send request
        let res = request
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::ChannelError(format!("Webhook request error: {e}")))?;

        if !res.status().is_success() {
            return Err(NotificationError::ChannelError(
                format!("Webhook returned error: {}", res.status())
            ));
        }

        Ok(())
    }

    /// Checks if a notification can be sent through a channel based on rate limits.
    /// 
    /// # Arguments
    /// 
    /// * `channel_id` - The ID of the channel to check
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a boolean indicating whether sending is allowed.
    async fn check_rate_limit(&self, channel_id: &str) -> Result<bool, NotificationError> {
        let config = self.config.read().await;
        let rate_limit = config.rate_limit;
        
        if rate_limit == 0 {
            return Ok(false); // No rate limiting
        }
        
        let mut limiter = self.rate_limiter.write().await;
        let now = OffsetDateTime::now_utc();
        
        if let Some(last_notification) = limiter.get(channel_id) {
            let elapsed = now - *last_notification;
            let elapsed_secs = elapsed.as_seconds_f64() as i64;
            if elapsed_secs < rate_limit as i64 {
                return Ok(true); // Rate limited
            }
        }
        
        // Update last notification time
        limiter.insert(channel_id.to_string(), now);
        Ok(false) // Not rate limited
    }

    /// Updates the notification configuration
    ///
    /// # Parameters
    /// * `config` - The new notification configuration
    ///
    /// # Errors
    /// Returns an error if the configuration lock cannot be acquired or if the configuration is invalid
    pub async fn update_config(&self, config: NotificationConfig) -> Result<(), NotificationError> {
        // Update templates
        {
            let mut handlebars = self.templates.write().await;
            for template in &config.templates {
                handlebars.register_template_string(&template.id, &template.content)
                    .map_err(|e| NotificationError::TemplateError(e.to_string()))?;
            }
        }
        
        // Update config
        let mut current_config = self.config.write().await;
        *current_config = config;
        
        Ok(())
    }

    /// Checks if a routing rule matches an alert.
    /// 
    /// # Arguments
    /// 
    /// * `rule` - The routing rule to check
    /// * `alert` - The alert to check against
    /// 
    /// # Returns
    /// 
    /// Returns true if the rule matches the alert.
    fn check_routing_rule(rule: &RoutingRule, alert: &AlertNotification) -> bool {
        // Check severity filter
        let severity_match = match &rule.severity {
            None => true,
            Some(s) => &alert.severity == s,
        };
        
        // Check component filter
        let component_match = match &rule.component {
            None => true,
            Some(c) => alert.component.contains(c),
        };
        
        severity_match && component_match
    }
}

const fn get_severity_color(severity: &AlertSeverity) -> &'static str {
    severity.color()
}

/// Factory for creating and managing notification manager instances
pub struct NotificationManagerFactory {
    /// Configuration for creating notification managers
    config: NotificationConfig,
}

impl NotificationManagerFactory {
    /// Creates a new factory with default configuration
    ///
    /// # Errors
    /// Returns an error if the default configuration is invalid
    pub fn new() -> Result<Self, NotificationError> {
        let config = NotificationConfig {
            channels: Vec::new(),
            rate_limit: 60,
            templates: Vec::new(),
            routing_rules: Vec::new(),
        };
        
        Ok(Self { config })
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: NotificationConfig) -> Self {
        Self { config }
    }

    /// Creates a notification manager
    ///
    /// # Errors
    /// Returns an error if the notification manager cannot be created
    pub fn create_manager(&self) -> Result<Arc<NotificationManager>, NotificationError> {
        Ok(Arc::new(NotificationManager::new(self.config.clone())?))
    }

    /// Creates a notification manager with dependencies
    pub fn create_manager_with_dependencies(
        &self,
        // Add any required dependencies here in the future
    ) -> Result<Arc<NotificationManager>, NotificationError> {
        self.create_manager()
    }

    /// Creates an adapter
    pub fn create_adapter(&self) -> Result<Arc<NotificationManagerAdapter>, NotificationError> {
        let manager = self.create_manager()?;
        Ok(Arc::new(NotificationManagerAdapter::with_manager(manager)))
    }
}

/// Create a notification manager adapter using dependency injection
///
/// # Errors
/// Returns an error if the adapter cannot be created
pub fn create_adapter(config: NotificationConfig) -> Result<Arc<NotificationManagerAdapter>, Box<dyn std::error::Error>> {
    let factory = NotificationManagerFactory::with_config(config);
    factory.create_adapter().map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))
}