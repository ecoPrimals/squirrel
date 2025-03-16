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
use thiserror::Error;
use time::OffsetDateTime;
use handlebars::Handlebars;
use serde::{Serialize, Deserialize};

use super::{AlertNotification, AlertSeverity};

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

/// Notification errors
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    #[error("Routing error: {0}")]
    RoutingError(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
}

/// Notification manager for handling alert notifications
#[derive(Debug)]
pub struct NotificationManager {
    /// Configuration
    config: Arc<RwLock<NotificationConfig>>,
    /// Template engine
    templates: Arc<RwLock<Handlebars<'static>>>,
    /// HTTP client
    client: reqwest::Client,
    /// Rate limiter
    rate_limiter: Arc<RwLock<std::collections::HashMap<String, OffsetDateTime>>>,
}

/// Email notification parameters
#[derive(Debug)]
#[allow(dead_code)]
struct EmailParams<'a> {
    /// SMTP server address
    smtp_server: &'a str,
    /// SMTP port
    smtp_port: u16,
    /// SMTP username for authentication
    username: &'a str,
    /// SMTP password for authentication
    password: &'a str,
    /// Sender email address
    from: &'a str,
    /// Recipient email addresses
    to: &'a [String],
    /// Alert notification to send
    alert: &'a AlertNotification,
}

impl NotificationManager {
    /// Creates a new notification manager with the given configuration
    ///
    /// # Parameters
    /// * `config` - The notification configuration to use
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid or if the notification channels cannot be initialized
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

    /// Sends a notification for the given alert
    ///
    /// # Parameters
    /// * `alert` - The alert to send a notification for
    ///
    /// # Errors
    /// Returns an error if the notification cannot be sent or if the configuration is invalid
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

    /// Check rate limit for a channel
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
            if elapsed.whole_seconds() < rate_limit as i64 {
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

    /// Checks if an alert matches a routing rule
    ///
    /// # Parameters
    /// * `rule` - The routing rule to check
    /// * `alert` - The alert to check against the rule
    ///
    /// # Returns
    /// `true` if the alert matches the rule, `false` otherwise
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

fn get_severity_color(severity: &AlertSeverity) -> &'static str {
    severity.color()
}

// Module initialization
static NOTIFICATION_MANAGER: tokio::sync::OnceCell<Arc<NotificationManager>> = 
    tokio::sync::OnceCell::const_new();

/// Initializes the notification system with the given configuration
///
/// # Parameters
/// * `config` - The notification configuration to use
///
/// # Errors
/// Returns an error if the notification manager is already initialized or if initialization fails
pub async fn initialize(config: NotificationConfig) -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(NotificationManager::new(config)?);

    NOTIFICATION_MANAGER.set(manager)
        .map_err(|_| Box::<dyn std::error::Error>::from("Notification manager already initialized"))?;
    
    Ok(())
}

/// Get the notification manager instance
pub fn get_manager() -> Option<Arc<NotificationManager>> {
    NOTIFICATION_MANAGER.get().cloned()
}

/// Check if the notification system is initialized
pub fn is_initialized() -> bool {
    NOTIFICATION_MANAGER.get().is_some()
}