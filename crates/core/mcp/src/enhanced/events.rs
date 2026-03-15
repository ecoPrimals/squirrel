// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Event Broadcasting System
//!
//! This module provides a universal event system that can handle events from
//! ANY AI system, provider, or custom implementation with real-time pub/sub.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock, Mutex};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{info, debug, warn, error, instrument};
use chrono::{DateTime, Utc};

use crate::error::Result;

/// Event channel for broadcasting events
#[derive(Debug, Clone)]
pub struct EventChannel {
    /// Channel sender
    pub sender: broadcast::Sender<MCPEvent>,
    
    /// Channel configuration
    pub config: EventChannelConfig,
}

/// Event channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChannelConfig {
    /// Channel capacity
    pub capacity: usize,
    
    /// Channel name
    pub name: String,
    
    /// Channel description
    pub description: String,
}

/// Universal event broadcaster - publishes events to all systems
#[derive(Debug)]
pub struct EventBroadcaster {
    /// Event channels by type
    channels: Arc<RwLock<HashMap<String, Vec<EventChannel>>>>,
    
    /// Event filters
    filters: Arc<RwLock<Vec<EventFilter>>>,
    
    /// Event history for replay
    history: Arc<RwLock<VecDeque<MCPEvent>>>,
    
    /// Configuration
    config: EventBroadcasterConfig,
    
    /// Metrics
    metrics: Arc<Mutex<EventMetrics>>,
}

/// Universal event types - covers ALL possible AI/MCP events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPEvent {
    /// Event ID
    pub id: String,
    
    /// Event type
    pub event_type: EventType,
    
    /// Source information
    pub source: EventSource,
    
    /// Event payload
    pub payload: serde_json::Value,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
    
    /// Event priority
    pub priority: EventPriority,
    
    /// Event tags for filtering
    pub tags: Vec<String>,
    
    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Event types covering all AI/MCP scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    // AI Provider Events
    /// AI request started
    AIRequestStarted,
    /// AI request completed
    AIRequestCompleted,
    /// AI request failed
    AIRequestFailed,
    /// AI model loaded
    AIModelLoaded,
    /// AI model unloaded
    AIModelUnloaded,
    /// AI provider registered
    AIProviderRegistered,
    /// AI provider health changed
    AIProviderHealthChanged,
    
    // MCP Protocol Events
    /// MCP session created
    SessionCreated,
    /// MCP session ended
    SessionEnded,
    /// MCP tool executed
    ToolExecuted,
    /// MCP protocol error
    ProtocolError,
    
    // System Events
    /// System startup
    SystemStartup,
    /// System shutdown
    SystemShutdown,
    /// Configuration changed
    ConfigurationChanged,
    /// Error occurred
    ErrorOccurred,
    
    // Streaming Events
    /// Stream started
    StreamStarted,
    /// Stream data
    StreamData,
    /// Stream ended
    StreamEnded,
    /// Stream error
    StreamError,
    
    // Resource Events
    /// Resource allocated
    ResourceAllocated,
    /// Resource released
    ResourceReleased,
    /// Resource limit exceeded
    ResourceLimitExceeded,
    
    // Custom Events
    /// Custom event type
    Custom(String),
    
    // Future Events
    /// Future event types (extensible)
    Future(String),
}

/// Event source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    /// Source type
    pub source_type: SourceType,
    
    /// Source identifier
    pub source_id: String,
    
    /// Source name
    pub source_name: String,
    
    /// Source version
    pub source_version: Option<String>,
    
    /// Source metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    /// AI provider
    AIProvider,
    
    /// Tool
    Tool,
    
    /// Server
    Server,
    
    /// Client
    Client,
    
    /// System
    System,
    
    /// Stream
    Stream,
    
    /// External
    External,
    
    /// Custom
    Custom(String),
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::AIProvider => write!(f, "ai_provider"),
            SourceType::Tool => write!(f, "tool"),
            SourceType::Server => write!(f, "server"),
            SourceType::Client => write!(f, "client"),
            SourceType::System => write!(f, "system"),
            SourceType::Stream => write!(f, "stream"),
            SourceType::External => write!(f, "external"),
            SourceType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// Event priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Low priority - logging, metrics
    Low,
    /// Normal priority - standard events
    Normal,
    /// High priority - important events
    High,
    /// Critical priority - urgent events
    Critical,
    /// Emergency - system-critical events
    Emergency,
}

/// Event filter for controlling which events are published
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Filter ID
    pub id: String,
    
    /// Event types to match
    pub event_types: Vec<EventType>,
    
    /// Source types to match
    pub source_types: Vec<SourceType>,
    
    /// Minimum priority
    pub min_priority: Option<EventPriority>,
    
    /// Required tags
    pub required_tags: Vec<String>,
    
    /// Filter description
    pub description: String,
}

impl EventFilter {
    /// Check if event matches this filter
    pub fn matches(&self, event: &MCPEvent) -> bool {
        // Check event type
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }
        
        // Check source type
        if !self.source_types.is_empty() && !self.source_types.contains(&event.source.source_type) {
            return false;
        }
        
        // Check priority
        if let Some(min_priority) = &self.min_priority {
            if event.priority < *min_priority {
                return false;
            }
        }
        
        // Check required tags
        for required_tag in &self.required_tags {
            if !event.tags.contains(required_tag) {
                return false;
            }
        }
        
        true
    }
}

/// Query for retrieving historical events
#[derive(Debug, Clone)]
pub struct EventQuery {
    /// Event types to filter
    pub event_types: Option<Vec<String>>,
    
    /// Start time
    pub since: Option<DateTime<Utc>>,
    
    /// End time  
    pub until: Option<DateTime<Utc>>,
    
    /// Limit number of results
    pub limit: Option<usize>,
    
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Event statistics
#[derive(Debug, Clone)]
pub struct EventStatistics {
    /// Total events published
    pub total_events_published: u64,
    
    /// Total active subscribers
    pub total_subscribers: u64,
    
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    
    /// Average processing time
    pub avg_processing_time: Duration,
}

/// Event metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    /// Total events processed
    pub total_events: u64,
    
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    
    /// Events by source
    pub events_by_source: HashMap<String, u64>,
    
    /// Events by priority
    pub events_by_priority: HashMap<String, u64>,
    
    /// Total subscribers
    pub total_subscribers: u64,
    
    /// Average processing time
    pub avg_processing_time: Duration,
    
    /// Error rate
    pub error_rate: f64,
    
    /// Event queue size
    pub queue_size: u64,
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_source: HashMap::new(),
            events_by_priority: HashMap::new(),
            total_subscribers: 0,
            avg_processing_time: Duration::default(),
            error_rate: 0.0,
            queue_size: 0,
        }
    }
}

/// Configuration for event broadcaster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBroadcasterConfig {
    /// Maximum number of event types to track
    pub max_event_types: usize,
    
    /// Channel buffer size for each event type
    pub channel_capacity: usize,
    
    /// Maximum history size
    pub max_history_size: usize,
    
    /// Maximum age for history events in seconds
    pub max_history_age_seconds: u64,
    
    /// Enable event persistence
    pub enable_persistence: bool,
}

impl Default for EventBroadcasterConfig {
    fn default() -> Self {
        Self {
            max_event_types: 1000,
            channel_capacity: 1000,
            max_history_size: 10000,
            max_history_age_seconds: 86400, // 24 hours
            enable_persistence: false,
        }
    }
}

impl EventBroadcaster {
    /// Create new event broadcaster
    pub async fn new(config: EventBroadcasterConfig) -> Result<Self> {
        Ok(Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            filters: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            config,
            metrics: Arc::new(Mutex::new(EventMetrics::default())),
        })
    }
    
    /// Publish event to all relevant channels
    #[instrument(skip(self, event))]
    pub async fn publish(&self, event: MCPEvent) -> Result<()> {
        debug!("Publishing event: {:?}", event.event_type);
        
        // Add to history
        let mut history = self.history.write().await;
        history.push_back(event.clone());
        
        // Keep history size under control
        if history.len() > self.config.max_history_size {
            history.pop_front();
        }
        drop(history);
        
        // Apply filters
        if !self.should_publish(&event).await {
            return Ok(());
        }
        
        // Send to relevant channels
        let channels = self.channels.read().await;
        let event_type = format!("{:?}", event.event_type);
        
        if let Some(event_channels) = channels.get(&event_type) {
            for channel in event_channels {
                let _ = channel.sender.send(event.clone());
            }
        }
        
        // Send to wildcard channels
        if let Some(wildcard_channels) = channels.get("*") {
            for channel in wildcard_channels {
                let _ = channel.sender.send(event.clone());
            }
        }
        
        // Update metrics
        self.update_metrics(&event).await;
        
        debug!("Event published successfully");
        Ok(())
    }
    
    /// Subscribe to events of a specific type
    pub async fn subscribe(&self, event_type: &str) -> Result<broadcast::Receiver<MCPEvent>> {
        let mut channels = self.channels.write().await;
        let event_channels = channels.entry(event_type.to_string()).or_insert_with(Vec::new);
        
        // Create new channel if none exists
        if event_channels.is_empty() {
            let (sender, _) = broadcast::channel(self.config.channel_capacity);
            event_channels.push(EventChannel {
                sender: sender.clone(),
                config: EventChannelConfig {
                    capacity: self.config.channel_capacity,
                    name: event_type.to_string(),
                    description: format!("Channel for {} events", event_type),
                },
            });
        }
        
        // Get receiver from first channel (we only use one channel per event type for now)
        let receiver = event_channels[0].sender.subscribe();
        
        info!("Subscribed to event type: {}", event_type);
        Ok(receiver)
    }
    
    /// Unsubscribe from events
    pub async fn unsubscribe(&self, _subscription_id: &str) -> Result<()> {
        // For now, receivers will automatically be dropped when subscriber drops them
        // In a more sophisticated implementation, we'd track subscriptions
        Ok(())
    }
    
    /// Check if event should be published based on filters
    async fn should_publish(&self, event: &MCPEvent) -> bool {
        let filters = self.filters.read().await;
        
        // If no filters, publish everything
        if filters.is_empty() {
            return true;
        }
        
        // Check all filters
        for filter in filters.iter() {
            if filter.matches(event) {
                return true;
            }
        }
        
        false
    }
    
    /// Add event filter
    pub async fn add_filter(&self, filter: EventFilter) -> Result<()> {
        let mut filters = self.filters.write().await;
        filters.push(filter);
        Ok(())
    }
    
    /// Remove event filter
    pub async fn remove_filter(&self, filter_id: &str) -> Result<()> {
        let mut filters = self.filters.write().await;
        filters.retain(|f| f.id != filter_id);
        Ok(())
    }
    
    /// Get event statistics
    pub async fn get_statistics(&self) -> Result<EventStatistics> {
        // Return simple statistics for now
        Ok(EventStatistics {
            total_events_published: 0,
            total_subscribers: 0,
            events_by_type: HashMap::new(),
            avg_processing_time: Duration::from_millis(0),
        })
    }
    
    /// Replay events for a subscriber
    pub async fn replay_events(&self, query: &EventQuery) -> Result<Vec<MCPEvent>> {
        let history = self.history.read().await;
        
        let events: Vec<MCPEvent> = history
            .iter()
            .filter(|event| {
                // Apply query filters
                if let Some(ref event_types) = query.event_types {
                    if !event_types.contains(&format!("{:?}", event.event_type)) {
                        return false;
                    }
                }
                
                if let Some(since) = query.since {
                    if event.timestamp < since {
                        return false;
                    }
                }
                
                if let Some(until) = query.until {
                    if event.timestamp > until {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect();
        
        Ok(events)
    }
    
    /// Start background cleanup task
    fn start_background_tasks(&self) {
        let history = self.history.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let mut history = history.write().await;
                let cutoff = Utc::now() - chrono::Duration::seconds(config.max_history_age_seconds as i64);
                
                // Remove old events
                while let Some(event) = history.front() {
                    if event.timestamp < cutoff {
                        history.pop_front();
                    } else {
                        break;
                    }
                }
            }
        });
    }
    
    /// Update metrics
    async fn update_metrics(&self, event: &MCPEvent) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_events += 1;
        
        let type_key = format!("{:?}", event.event_type);
        *metrics.events_by_type.entry(type_key).or_insert(0) += 1;
        
        let priority_key = format!("{:?}", event.priority);
        *metrics.events_by_priority.entry(priority_key).or_insert(0) += 1;
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> EventMetrics {
        self.metrics.lock().await.clone()
    }
}

impl MCPEvent {
    /// Create new event
    pub fn new(
        event_type: EventType,
        source: EventSource,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            source,
            payload,
            timestamp: Utc::now(),
            correlation_id: None,
            priority: EventPriority::Normal,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }
    
    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl EventSource {
    /// Create new event source
    pub fn new(source_type: SourceType, source_id: String, source_name: String) -> Self {
        Self {
            source_type,
            source_id,
            source_name,
            source_version: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set version
    pub fn with_version(mut self, version: String) -> Self {
        self.source_version = Some(version);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
} 