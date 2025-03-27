use crate::components::{Component, ComponentType};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Job status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// Job is queued for execution
    Queued,
    /// Job is currently running
    Running,
    /// Job has completed successfully
    Completed,
    /// Job has failed
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job status is unknown
    Unknown,
}

impl JobStatus {
    /// Get the string representation of the status
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Queued => "QUEUED",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
            JobStatus::Cancelled => "CANCELLED",
            JobStatus::Unknown => "UNKNOWN",
        }
    }

    /// Get a CSS class for the status
    pub fn css_class(&self) -> &'static str {
        match self {
            JobStatus::Queued => "status-queued",
            JobStatus::Running => "status-running",
            JobStatus::Completed => "status-completed",
            JobStatus::Failed => "status-failed",
            JobStatus::Cancelled => "status-cancelled",
            JobStatus::Unknown => "status-unknown",
        }
    }

    /// Convert from a string to a JobStatus
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "QUEUED" => JobStatus::Queued,
            "RUNNING" => JobStatus::Running,
            "COMPLETED" => JobStatus::Completed,
            "FAILED" => JobStatus::Failed,
            "CANCELLED" => JobStatus::Cancelled,
            _ => JobStatus::Unknown,
        }
    }
}

/// Job progress information
#[derive(Debug, Clone)]
pub struct JobProgress {
    /// Current progress value
    pub current: u64,
    /// Total progress value
    pub total: u64,
}

impl JobProgress {
    /// Create a new job progress
    pub fn new(current: u64, total: u64) -> Self {
        Self { current, total }
    }

    /// Calculate progress percentage
    pub fn percentage(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        let percentage = (self.current as f64 / self.total as f64) * 100.0;
        percentage.min(100.0) as u8
    }
}

/// Job data structure
#[derive(Debug, Clone)]
pub struct JobData {
    /// Job ID
    pub id: String,
    /// Job name
    pub name: String,
    /// Job status
    pub status: JobStatus,
    /// Job progress
    pub progress: Option<JobProgress>,
    /// Created timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Updated timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Result data
    pub result: Option<serde_json::Value>,
    /// Error message
    pub error: Option<String>,
}

/// Job list component
pub struct JobList {
    /// Component ID
    id: Uuid,
    /// Component title
    title: String,
    /// List of jobs
    jobs: Vec<JobData>,
    /// Selected job ID
    selected_job: Option<String>,
    /// Jobs loaded indicator
    loaded: bool,
}

impl JobList {
    /// Create a new job list
    pub fn new(id: impl Into<Uuid>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            jobs: Vec::new(),
            selected_job: None,
            loaded: false,
        }
    }

    /// Set jobs for the list
    pub fn set_jobs(&mut self, jobs: Vec<JobData>) {
        self.jobs = jobs;
        self.loaded = true;
    }

    /// Select a job
    pub fn select_job(&mut self, job_id: impl Into<String>) {
        self.selected_job = Some(job_id.into());
    }

    /// Get the selected job
    pub fn selected_job(&self) -> Option<&JobData> {
        if let Some(id) = &self.selected_job {
            self.jobs.iter().find(|job| &job.id == id)
        } else {
            None
        }
    }

    /// Check if jobs are loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

impl Component for JobList {
    /// Get component ID
    fn id(&self) -> Uuid {
        self.id
    }

    /// Get component name
    fn name(&self) -> String {
        format!("JobList: {}", self.title)
    }

    /// Get component type
    fn component_type(&self) -> ComponentType {
        ComponentType::Content
    }

    /// Render component to HTML
    fn render_html(&self) -> String {
        let job_items = if self.jobs.is_empty() {
            if self.loaded {
                "<div class=\"empty-message\">No jobs available.</div>".to_string()
            } else {
                "<div class=\"loading\">Loading jobs...</div>".to_string()
            }
        } else {
            let mut html = String::new();
            for job in &self.jobs {
                let selected_class = if let Some(selected_id) = &self.selected_job {
                    if selected_id == &job.id {
                        " selected"
                    } else {
                        ""
                    }
                } else {
                    ""
                };

                html.push_str(&format!(
                    "<div class=\"job-item{}\" data-job-id=\"{}\">\
                        <div class=\"job-name\">{}</div>\
                        <div class=\"job-status {}\">{}</div>\
                    </div>",
                    selected_class,
                    htmlescape::encode_minimal(&job.id),
                    htmlescape::encode_minimal(&job.name),
                    job.status.css_class(),
                    htmlescape::encode_minimal(job.status.as_str())
                ));
            }
            html
        };

        format!(
            "<div class=\"jobs-list\" id=\"{}\">{}</div>",
            self.id, job_items
        )
    }
}

/// Job details component
pub struct JobDetails {
    /// Component ID
    id: Uuid,
    /// Job data
    job: Option<JobData>,
    /// Can cancel flag
    can_cancel: bool,
}

impl JobDetails {
    /// Create a new job details component
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self {
            id: id.into(),
            job: None,
            can_cancel: true,
        }
    }

    /// Set the job to display
    pub fn set_job(&mut self, job: JobData) {
        self.job = Some(job);
    }

    /// Clear the job
    pub fn clear_job(&mut self) {
        self.job = None;
    }

    /// Get the current job
    pub fn job(&self) -> Option<&JobData> {
        self.job.as_ref()
    }

    /// Set whether the job can be cancelled
    pub fn set_can_cancel(&mut self, can_cancel: bool) {
        self.can_cancel = can_cancel;
    }

    /// Check if the job can be cancelled
    pub fn can_cancel(&self) -> bool {
        self.can_cancel && 
        self.job.as_ref().map_or(false, |job| {
            job.status == JobStatus::Queued || job.status == JobStatus::Running
        })
    }
}

impl Component for JobDetails {
    /// Get component ID
    fn id(&self) -> Uuid {
        self.id
    }

    /// Get component name
    fn name(&self) -> String {
        "Job Details".to_string()
    }

    /// Get component type
    fn component_type(&self) -> ComponentType {
        ComponentType::Content
    }

    /// Render component to HTML
    fn render_html(&self) -> String {
        if let Some(job) = &self.job {
            // Format the created timestamp
            let created_at = job.created_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            // Format the result
            let result = job.result.as_ref()
                .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| String::from("{}")))
                .unwrap_or_else(|| String::from("No result available"));
            
            // Calculate progress
            let (progress_value, progress_text) = if let Some(progress) = &job.progress {
                let percentage = progress.percentage();
                (format!("{}%", percentage), format!("{}%", percentage))
            } else {
                match job.status {
                    JobStatus::Completed => ("100%".to_string(), "100%".to_string()),
                    JobStatus::Failed | JobStatus::Cancelled => ("0%".to_string(), "0%".to_string()),
                    _ => ("0%".to_string(), "0%".to_string()),
                }
            };
            
            // Determine if cancel button should be shown
            let cancel_button = if self.can_cancel() {
                format!("<button id=\"cancel-job\" class=\"danger-button\" data-job-id=\"{}\">Cancel Job</button>", 
                    htmlescape::encode_minimal(&job.id))
            } else {
                String::new()
            };
            
            // Show error if present
            let error_section = if let Some(error) = &job.error {
                format!(
                    "<div class=\"detail-item\">\
                        <span class=\"detail-label\">Error:</span>\
                        <pre class=\"detail-value error\">{}</pre>\
                    </div>",
                    htmlescape::encode_minimal(error)
                )
            } else {
                String::new()
            };
            
            format!(
                "<div class=\"job-details\" id=\"{}\">\
                    <h3>Job Details</h3>\
                    <div class=\"detail-item\">\
                        <span class=\"detail-label\">ID:</span>\
                        <span class=\"detail-value\" id=\"job-id\">{}</span>\
                    </div>\
                    <div class=\"detail-item\">\
                        <span class=\"detail-label\">Status:</span>\
                        <span class=\"detail-value {} status-pill\" id=\"job-status\">{}</span>\
                    </div>\
                    <div class=\"detail-item\">\
                        <span class=\"detail-label\">Progress:</span>\
                        <div class=\"progress-bar\">\
                            <div class=\"progress\" id=\"job-progress\" style=\"width: {}\"></div>\
                        </div>\
                        <span class=\"progress-text\" id=\"job-progress-text\">{}</span>\
                    </div>\
                    <div class=\"detail-item\">\
                        <span class=\"detail-label\">Created:</span>\
                        <span class=\"detail-value\" id=\"job-created-at\">{}</span>\
                    </div>\
                    <div class=\"detail-item\">\
                        <span class=\"detail-label\">Result:</span>\
                        <pre class=\"detail-value\" id=\"job-result\">{}</pre>\
                    </div>\
                    {}\
                    {}\
                </div>",
                self.id,
                htmlescape::encode_minimal(&job.id),
                job.status.css_class(),
                htmlescape::encode_minimal(job.status.as_str()),
                progress_value,
                progress_text,
                htmlescape::encode_minimal(&created_at),
                htmlescape::encode_minimal(&result),
                error_section,
                cancel_button
            )
        } else {
            format!(
                "<div class=\"job-details hidden\" id=\"{}\">\
                    <div class=\"empty-message\">Select a job to view details</div>\
                </div>",
                self.id
            )
        }
    }
} 