//! Multi-Agent Coordination Types
//!
//! This module contains all the type definitions for the multi-agent coordination system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tokio::sync::{RwLock, Mutex, mpsc};
use futures::future::AbortHandle;

/// Agent types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    /// Coordinator agent
    Coordinator,
    
    /// Processing agent
    Processor,
    
    /// Analyzer agent
    Analyzer,
    
    /// Validator agent
    Validator,
    
    /// Aggregator agent
    Aggregator,
    
    /// Specialist agent
    Specialist,
    
    /// Custom agent type
    Custom(String),
}

/// Agent states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentState {
    /// Idle
    Idle,
    
    /// Processing
    Processing,
    
    /// Collaborating
    Collaborating,
    
    /// Waiting
    Waiting,
    
    /// Error
    Error,
    
    /// Offline
    Offline,
}

/// Agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Capability name
    pub name: String,
    
    /// Capability description
    pub description: String,
    
    /// Capability parameters
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Capability requirements
    pub requirements: Vec<String>,
    
    /// Capability performance metrics
    pub performance: Option<CapabilityPerformance>,
}

/// Capability performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPerformance {
    /// Average execution time
    pub avg_execution_time: Duration,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Throughput
    pub throughput: f64,
    
    /// Quality score
    pub quality_score: f64,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    
    /// Agent description
    pub description: String,
    
    /// Agent timeout
    pub timeout: Duration,
    
    /// Agent retry configuration
    pub retry: RetryConfig,
    
    /// Agent resource limits
    pub resources: ResourceLimits,
    
    /// Agent behavior configuration
    pub behavior: BehaviorConfig,
    
    /// Agent metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Collaboration preference
    pub collaboration_preference: CollaborationPreference,
    
    /// Communication style
    pub communication_style: CommunicationStyle,
    
    /// Decision making strategy
    pub decision_strategy: DecisionStrategy,
    
    /// Learning enabled
    pub learning_enabled: bool,
    
    /// Adaptation enabled
    pub adaptation_enabled: bool,
}

/// Collaboration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationPreference {
    /// Prefers working alone
    Independent,
    
    /// Prefers small groups
    SmallGroup,
    
    /// Prefers large groups
    LargeGroup,
    
    /// Adaptive to situation
    Adaptive,
}

/// Communication styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStyle {
    /// Direct communication
    Direct,
    
    /// Diplomatic communication
    Diplomatic,
    
    /// Analytical communication
    Analytical,
    
    /// Collaborative communication
    Collaborative,
}

/// Decision making strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionStrategy {
    /// Quick decisions
    Fast,
    
    /// Careful analysis
    Thorough,
    
    /// Consensus-based
    Consensus,
    
    /// Data-driven
    DataDriven,
}

/// Agent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Message ID
    pub id: String,
    
    /// Sender agent ID
    pub sender: String,
    
    /// Receiver agent ID
    pub receiver: String,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Message content
    pub content: serde_json::Value,
    
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Message priority
    pub priority: MessagePriority,
}

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request message
    Request,
    
    /// Response message
    Response,
    
    /// Notification message
    Notification,
    
    /// Collaboration message
    Collaboration,
    
    /// Status update message
    StatusUpdate,
    
    /// Error message
    Error,
    
    /// Custom message
    Custom(String),
}

/// Message priorities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority
    Low,
    
    /// Normal priority
    Normal,
    
    /// High priority
    High,
    
    /// Critical priority
    Critical,
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent version
    pub version: String,
    
    /// Agent creation time
    pub created_at: DateTime<Utc>,
    
    /// Agent last update time
    pub updated_at: DateTime<Utc>,
    
    /// Agent tags
    pub tags: Vec<String>,
    
    /// Agent properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Agent statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatistics {
    /// Total messages processed
    pub messages_processed: u64,
    
    /// Total collaborations participated
    pub collaborations_participated: u64,
    
    /// Total errors encountered
    pub errors_encountered: u64,
    
    /// Average processing time
    pub avg_processing_time: Duration,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Uptime
    pub uptime: Duration,
}

/// Collaboration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationType {
    /// Sequential collaboration
    Sequential,
    
    /// Parallel collaboration
    Parallel,
    
    /// Hierarchical collaboration
    Hierarchical,
    
    /// Peer-to-peer collaboration
    PeerToPeer,
    
    /// Consensus-based collaboration
    Consensus,
    
    /// Custom collaboration
    Custom(String),
}

/// Conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Conversation ID
    pub id: String,
    
    /// Participants
    pub participants: Vec<String>,
    
    /// Conversation state
    pub state: ConversationState,
    
    /// Messages
    pub messages: Vec<ConversationMessage>,
    
    /// Conversation metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    
    /// Conversation timeout
    pub timeout: Duration,
}

/// Conversation states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationState {
    /// Active
    Active,
    
    /// Paused
    Paused,
    
    /// Completed
    Completed,
    
    /// Cancelled
    Cancelled,
    
    /// Expired
    Expired,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message ID
    pub id: String,
    
    /// Sender ID
    pub sender: String,
    
    /// Message content
    pub content: serde_json::Value,
    
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Workflow ID
    pub id: String,
    
    /// Workflow name
    pub name: String,
    
    /// Workflow description
    pub description: String,
    
    /// Workflow steps
    pub steps: Vec<WorkflowStep>,
    
    /// Workflow configuration
    pub config: WorkflowConfig,
    
    /// Workflow metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step ID
    pub id: String,
    
    /// Step name
    pub name: String,
    
    /// Step type
    pub step_type: WorkflowStepType,
    
    /// Agent assignments
    pub agent_assignments: Vec<String>,
    
    /// Step dependencies
    pub dependencies: Vec<String>,
    
    /// Step configuration
    pub config: serde_json::Value,
    
    /// Step timeout
    pub timeout: Duration,
}

/// Workflow step types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStepType {
    /// Single agent step
    SingleAgent,
    
    /// Multi-agent step
    MultiAgent,
    
    /// Collaboration step
    Collaboration,
    
    /// Aggregation step
    Aggregation,
    
    /// Validation step
    Validation,
    
    /// Custom step
    Custom(String),
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Execution strategy
    pub execution_strategy: ExecutionStrategy,
    
    /// Timeout configuration
    pub timeout: Duration,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Error handling configuration
    pub error_handling: ErrorHandlingConfig,
    
    /// Resource limits
    pub resources: ResourceLimits,
}

/// Execution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    /// Sequential execution
    Sequential,
    
    /// Parallel execution
    Parallel,
    
    /// Adaptive execution
    Adaptive,
    
    /// Priority-based execution
    Priority,
}

/// Agent event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    /// Event ID
    pub id: String,
    
    /// Event type
    pub event_type: AgentEventType,
    
    /// Agent ID
    pub agent_id: String,
    
    /// Event data
    pub data: serde_json::Value,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEventType {
    /// Agent started
    Started,
    
    /// Agent stopped
    Stopped,
    
    /// Agent error
    Error,
    
    /// Agent message sent
    MessageSent,
    
    /// Agent message received
    MessageReceived,
    
    /// Agent collaboration started
    CollaborationStarted,
    
    /// Agent collaboration completed
    CollaborationCompleted,
    
    /// Custom event
    Custom(String),
}

/// Multi-agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentMetrics {
    /// Total agents
    pub total_agents: usize,
    
    /// Active agents
    pub active_agents: usize,
    
    /// Total messages processed
    pub total_messages: u64,
    
    /// Total collaborations
    pub total_collaborations: u64,
    
    /// Average response time
    pub avg_response_time: Duration,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Resource utilization
    pub resource_utilization: f64,
}

/// Multi-agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    /// Maximum number of agents
    pub max_agents: usize,
    
    /// Agent timeout
    pub agent_timeout: Duration,
    
    /// Conversation timeout
    pub conversation_timeout: Duration,
    
    /// Collaboration timeout
    pub collaboration_timeout: Duration,
    
    /// Workflow timeout
    pub workflow_timeout: Duration,
    
    /// Message buffer size
    pub message_buffer_size: usize,
    
    /// Metrics collection interval
    pub metrics_interval: Duration,
    
    /// Cleanup interval
    pub cleanup_interval: Duration,
    
    /// Resource limits
    pub resources: ResourceLimits,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Retry delay
    pub delay: Duration,
    
    /// Backoff strategy
    pub backoff_strategy: BackoffStrategy,
}

/// Backoff strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay
    Fixed,
    
    /// Exponential backoff
    Exponential,
    
    /// Linear backoff
    Linear,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU
    pub max_cpu: f64,
    
    /// Maximum memory
    pub max_memory: u64,
    
    /// Maximum concurrent operations
    pub max_concurrent_ops: u32,
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingConfig {
    /// Error handling strategy
    pub strategy: ErrorHandlingStrategy,
    
    /// Error recovery actions
    pub recovery_actions: Vec<String>,
    
    /// Error notifications
    pub notifications: Vec<String>,
}

/// Error handling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    /// Fail fast
    FailFast,
    
    /// Retry
    Retry,
    
    /// Ignore
    Ignore,
    
    /// Escalate
    Escalate,
}

/// Collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// Session ID
    pub id: String,
    
    /// Session type
    pub session_type: CollaborationType,
    
    /// Participants
    pub participants: Vec<String>,
    
    /// Session state
    pub state: CollaborationState,
    
    /// Session configuration
    pub config: CollaborationConfig,
    
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Collaboration states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollaborationState {
    /// Pending
    Pending,
    
    /// Active
    Active,
    
    /// Paused
    Paused,
    
    /// Completed
    Completed,
    
    /// Failed
    Failed,
    
    /// Cancelled
    Cancelled,
}

/// Collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    /// Collaboration timeout
    pub timeout: Duration,
    
    /// Synchronization strategy
    pub sync_strategy: SyncStrategy,
    
    /// Result aggregation strategy
    pub aggregation_strategy: AggregationStrategy,
    
    /// Quality threshold
    pub quality_threshold: f64,
}

/// Synchronization strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStrategy {
    /// Strict synchronization
    Strict,
    
    /// Eventual synchronization
    Eventual,
    
    /// Best effort synchronization
    BestEffort,
}

/// Aggregation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Simple average
    Average,
    
    /// Weighted average
    WeightedAverage,
    
    /// Majority vote
    MajorityVote,
    
    /// Consensus
    Consensus,
    
    /// Custom aggregation
    Custom(String),
} 