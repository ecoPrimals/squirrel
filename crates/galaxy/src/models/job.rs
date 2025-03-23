//! Module defining Galaxy job data models
//! 
//! This module contains the data structures for representing Galaxy jobs,
//! job states, and related objects.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::models::{ResourceMetadata, ParameterValue};

/// Represents a Galaxy job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyJob {
    /// Common metadata for the job
    pub metadata: ResourceMetadata,
    
    /// The tool ID that is being run
    pub tool_id: String,
    
    /// The tool version being used
    pub tool_version: Option<String>,
    
    /// The current state of this job
    pub state: JobState,
    
    /// When this job was created
    pub create_time: String,
    
    /// When this job was last updated
    pub update_time: String,
    
    /// The parameters used for this job
    pub params: HashMap<String, ParameterValue>,
    
    /// The Galaxy history ID this job belongs to
    pub history_id: String,
    
    /// The exit code from the job, if it has completed
    pub exit_code: Option<i32>,
    
    /// Whether the job has been deleted
    pub deleted: bool,
    
    /// The user who created this job
    pub user_id: Option<String>,
    
    /// Input datasets for this job
    pub inputs: HashMap<String, String>,
    
    /// Output datasets created by this job
    pub outputs: HashMap<String, String>,
    
    /// Details about where this job is running
    pub destination: Option<JobDestination>,
    
    /// Command line that was executed
    pub command_line: Option<String>,
    
    /// Metrics about the job execution
    pub metrics: Option<JobMetrics>,
    
    /// External ID for this job (e.g., cluster ID)
    pub external_id: Option<String>,
    
    /// Any dependencies for this job
    pub dependencies: Vec<String>,
}

/// Represents the current state of a Galaxy job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    /// The job is new and not yet scheduled
    #[serde(rename = "new")]
    New,
    
    /// The job has been accepted and is waiting to run
    #[serde(rename = "accepted")]
    Accepted,
    
    /// The job is waiting for resources
    #[serde(rename = "waiting")]
    Waiting,
    
    /// The job is in the queue
    #[serde(rename = "queued")]
    Queued,
    
    /// The job is running
    #[serde(rename = "running")]
    Running,
    
    /// The job has completed successfully
    #[serde(rename = "ok")]
    Ok,
    
    /// The job failed
    #[serde(rename = "error")]
    Error,
    
    /// The job was paused
    #[serde(rename = "paused")]
    Paused,
    
    /// The job was deleted
    #[serde(rename = "deleted")]
    Deleted,
    
    /// The job was resubmitted
    #[serde(rename = "resubmitted")]
    Resubmitted,
    
    /// Custom state not covered by the standard states
    #[serde(untagged)]
    Custom(String),
}

/// Represents where a job is running
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDestination {
    /// The ID of the destination
    pub id: String,
    
    /// The type of destination
    pub destination_type: String,
    
    /// The runner for this job
    pub runner: String,
    
    /// The environment where this job is running
    pub environment: HashMap<String, String>,
    
    /// The parameters for this destination
    pub params: HashMap<String, String>,
}

/// Metrics for a Galaxy job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetrics {
    /// CPU time used by the job
    pub cpu_time: Option<f64>,
    
    /// Memory used by the job (in MB)
    pub memory_used_mb: Option<f64>,
    
    /// Job runtime in seconds
    pub runtime_seconds: Option<f64>,
    
    /// Job start time
    pub start_time: Option<String>,
    
    /// Job end time
    pub end_time: Option<String>,
    
    /// Custom metrics specific to the job type
    pub custom_metrics: HashMap<String, String>,
}

/// Job search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSearchParams {
    /// Filter by tool ID
    pub tool_id: Option<String>,
    
    /// Filter by history ID
    pub history_id: Option<String>,
    
    /// Filter by job state
    pub state: Option<JobState>,
    
    /// Filter by user ID
    pub user_id: Option<String>,
    
    /// Filter by jobs created after this date
    pub created_after: Option<String>,
    
    /// Filter by jobs created before this date
    pub created_before: Option<String>,
    
    /// Include deleted jobs
    pub include_deleted: bool,
    
    /// Limit to specific dataset IDs
    pub dataset_ids: Option<Vec<String>>,
}

/// Parameters for cancelling a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCancelParams {
    /// The ID of the job to cancel
    pub job_id: String,
    
    /// Whether to mark the job as deleted
    pub mark_deleted: bool,
    
    /// Message explaining the cancellation
    pub message: Option<String>,
}

/// Output from a Galaxy job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobOutput {
    /// The name of this output
    pub name: String,
    
    /// The dataset ID for this output
    pub dataset_id: String,
    
    /// The history ID this output is in
    pub history_id: String,
    
    /// The file type of this output
    pub file_ext: String,
    
    /// The size of this output
    pub file_size: Option<u64>,
    
    /// URL to download this output
    pub download_url: Option<String>,
    
    /// The current state of this output
    pub state: String,
}

impl GalaxyJob {
    /// Create a new Galaxy job with a tool ID and history ID
    pub fn new(tool_id: &str, history_id: &str) -> Self {
        Self {
            metadata: ResourceMetadata::new(&format!("Job for {}", tool_id)),
            tool_id: tool_id.to_string(),
            tool_version: None,
            state: JobState::New,
            create_time: chrono::Utc::now().to_rfc3339(),
            update_time: chrono::Utc::now().to_rfc3339(),
            params: HashMap::new(),
            history_id: history_id.to_string(),
            exit_code: None,
            deleted: false,
            user_id: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            destination: None,
            command_line: None,
            metrics: None,
            external_id: None,
            dependencies: Vec::new(),
        }
    }
    
    /// Set the tool version for this job
    pub fn with_tool_version(&mut self, version: &str) -> &mut Self {
        self.tool_version = Some(version.to_string());
        self
    }
    
    /// Set the state of this job
    pub fn with_state(&mut self, state: JobState) -> &mut Self {
        self.state = state;
        self.update_time = chrono::Utc::now().to_rfc3339();
        self
    }
    
    /// Add a parameter to this job
    pub fn add_param(&mut self, name: &str, value: ParameterValue) -> &mut Self {
        self.params.insert(name.to_string(), value);
        self
    }
    
    /// Add an input dataset to this job
    pub fn add_input(&mut self, name: &str, dataset_id: &str) -> &mut Self {
        self.inputs.insert(name.to_string(), dataset_id.to_string());
        self
    }
    
    /// Add an output dataset to this job
    pub fn add_output(&mut self, name: &str, dataset_id: &str) -> &mut Self {
        self.outputs.insert(name.to_string(), dataset_id.to_string());
        self
    }
    
    /// Set the command line for this job
    pub fn with_command_line(&mut self, command: &str) -> &mut Self {
        self.command_line = Some(command.to_string());
        self
    }
    
    /// Set the user ID for this job
    pub fn with_user_id(&mut self, user_id: &str) -> &mut Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    /// Set the job destination
    pub fn with_destination(&mut self, destination: JobDestination) -> &mut Self {
        self.destination = Some(destination);
        self
    }
    
    /// Add metrics to this job
    pub fn with_metrics(&mut self, metrics: JobMetrics) -> &mut Self {
        self.metrics = Some(metrics);
        self
    }
    
    /// Add a dependency to this job
    pub fn add_dependency(&mut self, job_id: &str) -> &mut Self {
        self.dependencies.push(job_id.to_string());
        self
    }
    
    /// Check if this job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self.state, JobState::Ok | JobState::Error | JobState::Deleted)
    }
    
    /// Check if this job is running
    pub fn is_running(&self) -> bool {
        matches!(self.state, JobState::Running)
    }
    
    /// Check if this job was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.state, JobState::Ok)
    }
}

/// Create a new job metrics object
pub fn create_job_metrics() -> JobMetrics {
    JobMetrics {
        cpu_time: None,
        memory_used_mb: None,
        runtime_seconds: None,
        start_time: None,
        end_time: None,
        custom_metrics: HashMap::new(),
    }
}

/// Create a new job destination
pub fn create_job_destination(id: &str, destination_type: &str, runner: &str) -> JobDestination {
    JobDestination {
        id: id.to_string(),
        destination_type: destination_type.to_string(),
        runner: runner.to_string(),
        environment: HashMap::new(),
        params: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_job() {
        let mut job = GalaxyJob::new("toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.72+galaxy1", "history123");
        
        job.with_tool_version("0.72+galaxy1")
           .with_state(JobState::Running)
           .add_input("input1", "dataset123")
           .add_output("output_html", "dataset124")
           .with_user_id("user123");
        
        assert_eq!(job.tool_id, "toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.72+galaxy1");
        assert_eq!(job.history_id, "history123");
        assert_eq!(job.tool_version, Some("0.72+galaxy1".to_string()));
        assert_eq!(job.state, JobState::Running);
        assert_eq!(job.inputs.get("input1"), Some(&"dataset123".to_string()));
        assert_eq!(job.outputs.get("output_html"), Some(&"dataset124".to_string()));
        assert_eq!(job.user_id, Some("user123".to_string()));
        assert_eq!(job.is_terminal(), false);
        assert_eq!(job.is_running(), true);
        assert_eq!(job.is_successful(), false);
    }
    
    #[test]
    fn test_job_states() {
        let mut job = GalaxyJob::new("test_tool", "history123");
        
        // Test various state transitions
        job.with_state(JobState::Accepted);
        assert_eq!(job.state, JobState::Accepted);
        assert_eq!(job.is_terminal(), false);
        
        job.with_state(JobState::Queued);
        assert_eq!(job.state, JobState::Queued);
        assert_eq!(job.is_terminal(), false);
        
        job.with_state(JobState::Running);
        assert_eq!(job.state, JobState::Running);
        assert_eq!(job.is_terminal(), false);
        assert_eq!(job.is_running(), true);
        
        job.with_state(JobState::Ok);
        assert_eq!(job.state, JobState::Ok);
        assert_eq!(job.is_terminal(), true);
        assert_eq!(job.is_running(), false);
        assert_eq!(job.is_successful(), true);
        
        // Create a new job to test error state
        let mut error_job = GalaxyJob::new("test_tool", "history123");
        error_job.with_state(JobState::Error);
        assert_eq!(error_job.state, JobState::Error);
        assert_eq!(error_job.is_terminal(), true);
        assert_eq!(error_job.is_successful(), false);
    }
} 