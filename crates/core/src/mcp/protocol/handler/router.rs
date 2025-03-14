use crate::mcp::{
    MCPMessage, MessageType, SecurityMetadata, SecurityLevel,
    MCPProtocol, PortManager, PortConfig, PortStatus,
    SecurityManager, SecurityConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, instrument};
use crate::mcp::error::{MCPError, ErrorContext, ErrorSeverity};
use crate::mcp::security::{Permission, Action};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamMessageType {
    CodeReview {
        file_path: String,
        line_numbers: Vec<u32>,
        severity: ReviewSeverity,
        comment: String,
        rule_violations: Vec<String>,
    },
    DocumentationUpdate {
        component: String,
        doc_type: DocType,
        content: String,
        priority: Priority,
    },
    ProcessError {
        component: String,
        error_type: ProcessErrorType,
        details: String,
        affected_rules: Vec<String>,
    },
    BuildStatus {
        branch: String,
        status: BuildStatus,
        metrics: BuildMetrics,
        warnings: Vec<String>,
    },
    Task,
    Review,
    Comment,
    Status,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocType {
    API,
    Architecture,
    Rules,
    Process,
    Code,
    Design,
    Documentation,
    Test,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessErrorType {
    Build,
    Test,
    Lint,
    Security,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Success,
    Warning,
    Error,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    pub duration: f64,
    pub test_coverage: f64,
    pub warnings_count: u32,
    pub errors_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub security_level: SecurityLevel,
    pub permissions: Vec<Permission>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    pub id: String,
    pub type_: TeamMessageType,
    pub content: String,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
    pub security_level: SecurityLevel,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub reviewer: String,
    pub severity: ReviewSeverity,
    pub doc_type: DocType,
    pub priority: Priority,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub state_transitions: Vec<StateTransition>,
    pub total_messages: u32,
    pub review_count: u32,
    pub average_review_time: f64,
    pub completion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: WorkflowStatus,
    pub to_state: WorkflowStatus,
    pub timestamp: DateTime<Utc>,
    pub initiator: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assignee: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    UnderReview,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFilter {
    pub status: Option<WorkflowStatus>,
    pub security_level: Option<SecurityLevel>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub assignee: Option<String>,
    pub priority: Option<Priority>,
}

pub struct TeamWorkflowManager {
    workflows: Arc<RwLock<HashMap<String, TeamWorkflow>>>,
    messages: Arc<RwLock<HashMap<String, Vec<TeamMessage>>>>,
    reviews: Arc<RwLock<HashMap<String, ReviewRequest>>>,
    tasks: Arc<RwLock<HashMap<String, Vec<Task>>>>,
    security: Arc<SecurityManager>,
}

impl TeamWorkflowManager {
    #[instrument(skip(security))]
    pub fn new(security: Arc<SecurityManager>) -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            security,
        }
    }

    #[instrument(skip(self))]
    pub async fn create_workflow(&self, workflow: TeamWorkflow, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_workflow(&self, id: &str, token: &str) -> Result<TeamWorkflow, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: Action::Read,
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        workflows.get(id).cloned().ok_or_else(|| MCPError::Tool {
            kind: crate::mcp::error::ToolErrorKind::NotFound,
            context: ErrorContext::new("get_workflow", "team_workflow")
                .with_severity(ErrorSeverity::Medium),
            tool_id: id.to_string(),
        })
    }

    #[instrument(skip(self))]
    pub async fn update_workflow_status(&self, id: &str, status: WorkflowStatus, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(id) {
            workflow.status = status;
            Ok(())
        } else {
            Err(MCPError::Tool {
                kind: crate::mcp::error::ToolErrorKind::NotFound,
                context: ErrorContext::new("update_workflow_status", "team_workflow")
                    .with_severity(ErrorSeverity::Medium),
                tool_id: id.to_string(),
            })
        }
    }

    #[instrument(skip(self))]
    pub async fn send_message(&self, workflow_id: &str, message: TeamMessage, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut messages = self.messages.write().await;
        messages.entry(workflow_id.to_string())
            .or_insert_with(Vec::new)
            .push(message);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_messages(&self, workflow_id: &str, token: &str) -> Result<Vec<TeamMessage>, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: Action::Read,
        };
        self.security.check_permission(token, &permission).await?;

        let messages = self.messages.read().await;
        Ok(messages.get(workflow_id).cloned().unwrap_or_default())
    }

    #[instrument(skip(self))]
    pub async fn create_review(&self, review: ReviewRequest, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "review".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut reviews = self.reviews.write().await;
        reviews.insert(review.id.clone(), review);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_review(&self, id: &str, token: &str) -> Result<ReviewRequest, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "review".to_string(),
            action: Action::Read,
        };
        self.security.check_permission(token, &permission).await?;

        let reviews = self.reviews.read().await;
        reviews.get(id).cloned().ok_or_else(|| MCPError::Tool {
            kind: crate::mcp::error::ToolErrorKind::NotFound,
            context: ErrorContext::new("get_review", "team_workflow")
                .with_severity(ErrorSeverity::Medium),
            tool_id: id.to_string(),
        })
    }

    #[instrument(skip(self))]
    pub async fn transition_workflow_state(
        &self,
        id: &str,
        new_status: WorkflowStatus,
        initiator: &str,
        reason: &str,
        token: &str
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(id) {
            let old_status = workflow.status.clone();
            workflow.status = new_status.clone();

            // Record state transition
            let transition = StateTransition {
                from_state: old_status,
                to_state: new_status,
                timestamp: Utc::now(),
                initiator: initiator.to_string(),
                reason: reason.to_string(),
            };

            // Update workflow metadata
            workflow.metadata.insert("last_transition".to_string(), serde_json::to_string(&transition).unwrap());
            workflow.metadata.insert("last_updated".to_string(), Utc::now().to_string());

            info!(
                workflow_id = id,
                from_state = ?old_status,
                to_state = ?new_status,
                initiator = initiator,
                "Workflow state transition completed"
            );

            Ok(())
        } else {
            Err(MCPError::Tool {
                kind: crate::mcp::error::ToolErrorKind::NotFound,
                context: ErrorContext::new("transition_workflow_state", "team_workflow")
                    .with_severity(ErrorSeverity::Medium),
                tool_id: id.to_string(),
            })
        }
    }

    #[instrument(skip(self))]
    pub async fn get_workflow_metrics(&self, id: &str, token: &str) -> Result<WorkflowMetrics, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: Action::Read,
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        let messages = self.messages.read().await;
        let reviews = self.reviews.read().await;

        let workflow = workflows.get(id).ok_or_else(|| MCPError::Tool {
            kind: crate::mcp::error::ToolErrorKind::NotFound,
            context: ErrorContext::new("get_workflow_metrics", "team_workflow")
                .with_severity(ErrorSeverity::Medium),
            tool_id: id.to_string(),
        })?;

        // Calculate metrics
        let workflow_messages = messages.get(id).cloned().unwrap_or_default();
        let workflow_reviews: Vec<_> = reviews.values()
            .filter(|r| workflow_messages.iter().any(|m| m.content.contains(&r.id)))
            .collect();

        let total_messages = workflow_messages.len() as u32;
        let review_count = workflow_reviews.len() as u32;

        // Calculate average review time
        let average_review_time = if !workflow_reviews.is_empty() {
            let total_time: f64 = workflow_messages.iter()
                .filter_map(|m| match &m.type_ {
                    TeamMessageType::Review => {
                        Some((Utc::now() - m.timestamp).num_seconds() as f64)
                    }
                    _ => None,
                })
                .sum();
            total_time / review_count as f64
        } else {
            0.0
        };

        // Calculate completion rate
        let completion_rate = match workflow.status {
            WorkflowStatus::Completed => 100.0,
            WorkflowStatus::Failed => 0.0,
            _ => {
                let total_tasks = workflow_messages.iter()
                    .filter(|m| matches!(m.type_, TeamMessageType::Task))
                    .count();
                let completed_tasks = workflow_messages.iter()
                    .filter(|m| {
                        matches!(m.type_, TeamMessageType::Task) &&
                        m.metadata.get("status").map_or(false, |s| s == "completed")
                    })
                    .count();
                if total_tasks > 0 {
                    (completed_tasks as f64 / total_tasks as f64) * 100.0
                } else {
                    0.0
                }
            }
        };

        // Parse state transitions from metadata
        let state_transitions = workflow.metadata.iter()
            .filter_map(|(key, value)| {
                if key.starts_with("transition_") {
                    serde_json::from_str::<StateTransition>(value).ok()
                } else {
                    None
                }
            })
            .collect();

        Ok(WorkflowMetrics {
            state_transitions,
            total_messages,
            review_count,
            average_review_time,
            completion_rate,
        })
    }

    #[instrument(skip(self))]
    pub async fn broadcast_team_message(
        &self,
        message: TeamMessage,
        workflows: Vec<String>,
        token: &str
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "message".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut messages = self.messages.write().await;
        for workflow_id in workflows {
            messages.entry(workflow_id)
                .or_insert_with(Vec::new)
                .push(message.clone());
        }

        info!(
            message_id = ?message.id,
            workflow_count = workflows.len(),
            "Team message broadcasted"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn create_task(&self, workflow_id: &str, task: Task, token: &str) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut tasks = self.tasks.write().await;
        tasks.entry(workflow_id.to_string())
            .or_insert_with(Vec::new)
            .push(task.clone());

        // Create a task message
        let task_message = TeamMessage {
            id: format!("task_msg_{}", task.id),
            type_: TeamMessageType::Task,
            content: format!("Task created: {}", task.title),
            sender: task.assignee.clone(),
            timestamp: Utc::now(),
            security_level: SecurityLevel::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("task_id".to_string(), task.id);
                map.insert("status".to_string(), format!("{:?}", task.status));
                map
            },
        };

        self.send_message(workflow_id, task_message, token).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update_task_status(
        &self,
        workflow_id: &str,
        task_id: &str,
        new_status: TaskStatus,
        token: &str
    ) -> Result<(), MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: Action::Write,
        };
        self.security.check_permission(token, &permission).await?;

        let mut tasks = self.tasks.write().await;
        if let Some(workflow_tasks) = tasks.get_mut(workflow_id) {
            if let Some(task) = workflow_tasks.iter_mut().find(|t| t.id == task_id) {
                task.status = new_status.clone();
                task.updated_at = Utc::now();

                // Create a status update message
                let status_message = TeamMessage {
                    id: format!("task_update_{}", Utc::now().timestamp()),
                    type_: TeamMessageType::Status,
                    content: format!("Task {} status updated to {:?}", task_id, new_status),
                    sender: task.assignee.clone(),
                    timestamp: Utc::now(),
                    security_level: SecurityLevel::High,
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("task_id".to_string(), task_id.to_string());
                        map.insert("status".to_string(), format!("{:?}", new_status));
                        map
                    },
                };

                self.send_message(workflow_id, status_message, token).await?;
                Ok(())
            } else {
                Err(MCPError::Tool {
                    kind: crate::mcp::error::ToolErrorKind::NotFound,
                    context: ErrorContext::new("update_task_status", "team_workflow")
                        .with_severity(ErrorSeverity::Medium),
                    tool_id: task_id.to_string(),
                })
            }
        } else {
            Err(MCPError::Tool {
                kind: crate::mcp::error::ToolErrorKind::NotFound,
                context: ErrorContext::new("update_task_status", "team_workflow")
                    .with_severity(ErrorSeverity::Medium),
                tool_id: workflow_id.to_string(),
            })
        }
    }

    #[instrument(skip(self))]
    pub async fn filter_workflows(&self, filter: &WorkflowFilter, token: &str) -> Result<Vec<TeamWorkflow>, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "workflow".to_string(),
            action: Action::Read,
        };
        self.security.check_permission(token, &permission).await?;

        let workflows = self.workflows.read().await;
        let filtered: Vec<TeamWorkflow> = workflows.values()
            .filter(|workflow| {
                // Apply filters
                let status_match = filter.status.as_ref()
                    .map_or(true, |s| &workflow.status == s);
                
                let security_match = filter.security_level.as_ref()
                    .map_or(true, |s| &workflow.security_level == s);
                
                let date_match = filter.date_range.as_ref()
                    .map_or(true, |(start, end)| {
                        workflow.metadata.get("last_updated")
                            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
                            .map_or(false, |date| date >= *start && date <= *end)
                    });

                let assignee_match = filter.assignee.as_ref()
                    .map_or(true, |a| {
                        workflow.metadata.get("assignee")
                            .map_or(false, |assignee| assignee == a)
                    });

                let priority_match = filter.priority.as_ref()
                    .map_or(true, |p| {
                        workflow.metadata.get("priority")
                            .map_or(false, |priority| priority == &format!("{:?}", p))
                    });

                status_match && security_match && date_match && assignee_match && priority_match
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    #[instrument(skip(self))]
    pub async fn get_workflow_tasks(&self, workflow_id: &str, token: &str) -> Result<Vec<Task>, MCPError> {
        // Validate permissions
        let permission = Permission {
            resource: "task".to_string(),
            action: Action::Read,
        };
        self.security.check_permission(token, &permission).await?;

        let tasks = self.tasks.read().await;
        Ok(tasks.get(workflow_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::security::{SecurityConfig, SecurityManager};
    use chrono::Duration;

    async fn setup_test_environment() -> (TeamWorkflowManager, String) {
        // Setup security manager
        let config = SecurityConfig {
            jwt_secret: "test_secret".to_string(),
            token_expiry: Duration::hours(1),
            encryption_algorithm: crate::mcp::security::EncryptionAlgorithm::AesGcm256,
            min_key_rotation: Duration::hours(24),
            security_level: SecurityLevel::High,
        };
        let security = Arc::new(SecurityManager::new(config).await.unwrap());
        
        // Create test token with all permissions
        let permissions = vec![
            Permission {
                resource: "workflow".to_string(),
                action: Action::Admin,
            },
            Permission {
                resource: "message".to_string(),
                action: Action::Admin,
            },
            Permission {
                resource: "review".to_string(),
                action: Action::Admin,
            },
        ];
        let token = security.create_token("test_user", permissions).await.unwrap();

        (TeamWorkflowManager::new(security), token.token)
    }

    #[tokio::test]
    async fn test_workflow_lifecycle() {
        let (manager, token) = setup_test_environment().await;

        // Create workflow
        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager.create_workflow(workflow.clone(), &token).await.unwrap();

        // Get workflow
        let retrieved = manager.get_workflow("test_workflow", &token).await.unwrap();
        assert_eq!(retrieved.id, workflow.id);

        // Update status
        manager.update_workflow_status("test_workflow", WorkflowStatus::Completed, &token).await.unwrap();
        let updated = manager.get_workflow("test_workflow", &token).await.unwrap();
        assert!(matches!(updated.status, WorkflowStatus::Completed));
    }

    #[tokio::test]
    async fn test_message_handling() {
        let (manager, token) = setup_test_environment().await;

        // Create message
        let message = TeamMessage {
            id: "test_message".to_string(),
            type_: TeamMessageType::Task,
            content: "Test message".to_string(),
            sender: "test_user".to_string(),
            timestamp: Utc::now(),
            security_level: SecurityLevel::High,
            metadata: HashMap::new(),
        };
        manager.send_message("test_workflow", message.clone(), &token).await.unwrap();

        // Get messages
        let messages = manager.get_messages("test_workflow", &token).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message.id);
    }

    #[tokio::test]
    async fn test_review_lifecycle() {
        let (manager, token) = setup_test_environment().await;

        // Create review
        let review = ReviewRequest {
            id: "test_review".to_string(),
            title: "Test Review".to_string(),
            description: "Test review description".to_string(),
            reviewer: "test_reviewer".to_string(),
            severity: ReviewSeverity::High,
            doc_type: DocType::Code,
            priority: Priority::High,
            security_level: SecurityLevel::High,
        };
        manager.create_review(review.clone(), &token).await.unwrap();

        // Get review
        let retrieved = manager.get_review("test_review", &token).await.unwrap();
        assert_eq!(retrieved.id, review.id);
    }

    #[tokio::test]
    async fn test_permission_handling() {
        let (manager, _) = setup_test_environment().await;

        // Create token with limited permissions
        let permissions = vec![Permission {
            resource: "workflow".to_string(),
            action: Action::Read,
        }];
        let limited_token = manager.security.create_token("limited_user", permissions).await.unwrap();

        // Test read access (should succeed)
        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        assert!(manager.create_workflow(workflow, &limited_token.token).await.is_err());
    }

    #[tokio::test]
    async fn test_workflow_state_transition() {
        let (manager, token) = setup_test_environment().await;

        // Create workflow
        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager.create_workflow(workflow, &token).await.unwrap();

        // Transition state
        manager.transition_workflow_state(
            "test_workflow",
            WorkflowStatus::Paused,
            "test_user",
            "Pausing for review",
            &token
        ).await.unwrap();

        // Verify state change
        let updated = manager.get_workflow("test_workflow", &token).await.unwrap();
        assert!(matches!(updated.status, WorkflowStatus::Paused));
    }

    #[tokio::test]
    async fn test_workflow_metrics() {
        let (manager, token) = setup_test_environment().await;

        // Create workflow with some activity
        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager.create_workflow(workflow, &token).await.unwrap();

        // Add some messages and reviews
        let message = TeamMessage {
            id: "test_message".to_string(),
            type_: TeamMessageType::Task,
            content: "Test task".to_string(),
            sender: "test_user".to_string(),
            timestamp: Utc::now(),
            security_level: SecurityLevel::High,
            metadata: {
                let mut map = HashMap::new();
                map.insert("status".to_string(), "completed".to_string());
                map
            },
        };
        manager.send_message("test_workflow", message, &token).await.unwrap();

        // Get metrics
        let metrics = manager.get_workflow_metrics("test_workflow", &token).await.unwrap();
        assert_eq!(metrics.total_messages, 1);
        assert!(metrics.completion_rate > 0.0);
    }

    #[tokio::test]
    async fn test_broadcast_team_message() {
        let (manager, token) = setup_test_environment().await;

        // Create multiple workflows
        for i in 1..=3 {
            let workflow = TeamWorkflow {
                id: format!("workflow_{}", i),
                name: format!("Workflow {}", i),
                description: "Test workflow".to_string(),
                status: WorkflowStatus::Active,
                security_level: SecurityLevel::High,
                permissions: vec![],
                metadata: HashMap::new(),
            };
            manager.create_workflow(workflow, &token).await.unwrap();
        }

        // Broadcast message
        let message = TeamMessage {
            id: "broadcast_message".to_string(),
            type_: TeamMessageType::Alert,
            content: "Important announcement".to_string(),
            sender: "test_user".to_string(),
            timestamp: Utc::now(),
            security_level: SecurityLevel::High,
            metadata: HashMap::new(),
        };
        manager.broadcast_team_message(
            message,
            vec![
                "workflow_1".to_string(),
                "workflow_2".to_string(),
                "workflow_3".to_string()
            ],
            &token
        ).await.unwrap();

        // Verify message received by all workflows
        for i in 1..=3 {
            let messages = manager.get_messages(&format!("workflow_{}", i), &token).await.unwrap();
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].id, "broadcast_message");
        }
    }

    #[tokio::test]
    async fn test_task_management() {
        let (manager, token) = setup_test_environment().await;

        // Create workflow
        let workflow = TeamWorkflow {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "Test workflow description".to_string(),
            status: WorkflowStatus::Active,
            security_level: SecurityLevel::High,
            permissions: vec![],
            metadata: HashMap::new(),
        };
        manager.create_workflow(workflow, &token).await.unwrap();

        // Create task
        let task = Task {
            id: "test_task".to_string(),
            title: "Test Task".to_string(),
            description: "Test task description".to_string(),
            assignee: "test_user".to_string(),
            status: TaskStatus::Todo,
            priority: Priority::High,
            due_date: Some(Utc::now() + Duration::days(1)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        manager.create_task("test_workflow", task.clone(), &token).await.unwrap();

        // Update task status
        manager.update_task_status("test_workflow", "test_task", TaskStatus::InProgress, &token).await.unwrap();

        // Get tasks
        let tasks = manager.get_workflow_tasks("test_workflow", &token).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(matches!(tasks[0].status, TaskStatus::InProgress));
    }

    #[tokio::test]
    async fn test_workflow_filtering() {
        let (manager, token) = setup_test_environment().await;

        // Create multiple workflows with different properties
        for i in 1..=3 {
            let workflow = TeamWorkflow {
                id: format!("workflow_{}", i),
                name: format!("Workflow {}", i),
                description: "Test workflow".to_string(),
                status: if i == 1 { WorkflowStatus::Active } else { WorkflowStatus::Completed },
                security_level: SecurityLevel::High,
                permissions: vec![],
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("last_updated".to_string(), Utc::now().to_rfc3339());
                    map.insert("assignee".to_string(), format!("user_{}", i));
                    map.insert("priority".to_string(), format!("{:?}", Priority::High));
                    map
                },
            };
            manager.create_workflow(workflow, &token).await.unwrap();
        }

        // Filter workflows
        let filter = WorkflowFilter {
            status: Some(WorkflowStatus::Active),
            security_level: Some(SecurityLevel::High),
            date_range: Some((Utc::now() - Duration::hours(1), Utc::now() + Duration::hours(1))),
            assignee: Some("user_1".to_string()),
            priority: Some(Priority::High),
        };

        let filtered = manager.filter_workflows(&filter, &token).await.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "workflow_1");
    }
} 