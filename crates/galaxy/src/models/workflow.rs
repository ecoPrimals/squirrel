//! Module defining Galaxy workflow data models
//! 
//! This module contains the data structures for representing Galaxy workflows,
//! including workflow definitions, steps, connections, and execution information.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{ResourceMetadata, GalaxyDataReference, ParameterValue};

/// Represents a Galaxy workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyWorkflow {
    /// Common metadata for the workflow
    pub metadata: ResourceMetadata,
    
    /// The steps in this workflow
    pub steps: Vec<WorkflowStep>,
    
    /// The connections between steps
    pub connections: Vec<WorkflowConnection>,
    
    /// Input parameters for the workflow
    pub inputs: Vec<WorkflowInput>,
    
    /// Output parameters for the workflow
    pub outputs: Vec<WorkflowOutput>,
    
    /// Tags associated with this workflow
    pub tags: Vec<String>,
    
    /// Whether this workflow is published or not
    pub published: bool,
    
    /// Annotation/notes for this workflow
    pub annotation: Option<String>,
    
    /// The format version of this workflow
    pub format_version: String,
    
    /// The license for this workflow
    pub license: Option<String>,
    
    /// The creator of this workflow
    pub creator: Option<String>,
}

/// Represents an individual step in a Galaxy workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// The ID for this step
    pub id: String,
    
    /// The type of this step (tool, subworkflow, input, etc.)
    pub step_type: WorkflowStepType,
    
    /// The tool associated with this step, if it's a tool step
    pub tool: Option<String>,
    
    /// The tool version if applicable
    pub tool_version: Option<String>,
    
    /// The input connections to this step
    pub input_connections: HashMap<String, StepInput>,
    
    /// The position of this step in the workflow editor
    pub position: StepPosition,
    
    /// The parameters for this step
    pub parameters: HashMap<String, ParameterValue>,
    
    /// For subworkflows, the ID of the referenced workflow
    pub subworkflow_id: Option<String>,
    
    /// A user-provided label for this step
    pub label: Option<String>,
    
    /// The UUID for this step
    pub uuid: Option<Uuid>,
}

/// The types of steps that can exist in a Galaxy workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStepType {
    /// A tool step that executes a Galaxy tool
    #[serde(rename = "tool")]
    Tool,
    
    /// A data input step
    #[serde(rename = "data_input")]
    DataInput,
    
    /// A collection input step
    #[serde(rename = "data_collection_input")]
    CollectionInput,
    
    /// A step that references another workflow
    #[serde(rename = "subworkflow")]
    Subworkflow,
    
    /// A parameter input step
    #[serde(rename = "parameter_input")]
    ParameterInput,
    
    /// A pause point in the workflow
    #[serde(rename = "pause")]
    Pause,
    
    /// A custom step type
    #[serde(untagged)]
    Custom(String),
}

/// Represents the position of a step in the workflow editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepPosition {
    /// X coordinate in the workflow editor
    pub left: f64,
    
    /// Y coordinate in the workflow editor
    pub top: f64,
}

/// Represents an input connection to a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepInput {
    /// The ID of the step that outputs this data
    pub source_step: String,
    
    /// The name of the output from the source step
    pub output_name: String,
}

/// Represents a connection between workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConnection {
    /// The source step ID
    pub source_step_id: String,
    
    /// The source step output
    pub source_output: String,
    
    /// The target step ID
    pub target_step_id: String,
    
    /// The target input name
    pub target_input: String,
}

/// Represents an input parameter for a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    /// The unique identifier for this input
    pub id: String,
    
    /// A label describing this input
    pub label: String,
    
    /// The type of this input
    pub input_type: WorkflowInputType,
    
    /// A default value, if any
    pub default: Option<ParameterValue>,
    
    /// Optional help text
    pub help: Option<String>,
}

/// The types of inputs a workflow can accept
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowInputType {
    /// A dataset input
    #[serde(rename = "dataset")]
    Dataset,
    
    /// A dataset collection input
    #[serde(rename = "dataset_collection")]
    Collection,
    
    /// A string parameter
    #[serde(rename = "string")]
    String,
    
    /// An integer parameter
    #[serde(rename = "integer")]
    Integer,
    
    /// A floating point parameter
    #[serde(rename = "float")]
    Float,
    
    /// A boolean parameter
    #[serde(rename = "boolean")]
    Boolean,
    
    /// A custom parameter type
    #[serde(untagged)]
    Custom(String),
}

/// Represents a workflow output parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    /// The label for this output
    pub label: String,
    
    /// The step ID that produces this output
    pub source_step_id: String,
    
    /// The output name from the source step
    pub output_name: String,
}

/// Execution request for a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionRequest {
    /// The ID of the workflow to execute
    pub workflow_id: String,
    
    /// The ID of the history to store results in (or creates a new one if not provided)
    pub history_id: Option<String>,
    
    /// The input parameters for the workflow
    pub inputs: HashMap<String, GalaxyDataReference>,
    
    /// Parameter values for tool steps
    pub parameters: Option<HashMap<String, HashMap<String, ParameterValue>>>,
    
    /// Whether to create a new history
    pub new_history: bool,
    
    /// A name for the history if creating a new one
    pub history_name: Option<String>,
    
    /// Whether to import the workflow before execution (for shared workflows)
    pub import_inputs_to_history: bool,
}

/// Response from a workflow execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResponse {
    /// The invocation ID for this execution
    pub invocation_id: String,
    
    /// The history ID where outputs are stored
    pub history_id: String,
    
    /// The current state of the workflow execution
    pub state: WorkflowState,
    
    /// The outputs produced by this workflow execution
    pub outputs: Option<HashMap<String, GalaxyDataReference>>,
}

/// The possible states of a workflow execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowState {
    /// The workflow is being scheduled
    #[serde(rename = "new")]
    New,
    
    /// The workflow is ready to run
    #[serde(rename = "ready")]
    Ready,
    
    /// The workflow is currently running
    #[serde(rename = "running")]
    Running,
    
    /// The workflow has completed successfully
    #[serde(rename = "completed")]
    Completed,
    
    /// The workflow execution failed
    #[serde(rename = "failed")]
    Failed,
    
    /// The workflow execution was cancelled
    #[serde(rename = "cancelled")]
    Cancelled,
    
    /// A custom state not covered by the standard states
    #[serde(untagged)]
    Custom(String),
}

/// Represents a workflow invocation (an instance of a workflow execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInvocation {
    /// The ID of this invocation
    pub id: String,
    
    /// The ID of the workflow being executed
    pub workflow_id: String,
    
    /// The current state of the invocation
    pub state: WorkflowState,
    
    /// The history where outputs are stored
    pub history_id: String,
    
    /// The time the invocation was created
    pub create_time: String,
    
    /// The time the invocation was last updated
    pub update_time: String,
    
    /// The ID of the user who ran this workflow
    pub user_id: Option<String>,
    
    /// The individual steps and their states
    pub steps: Vec<InvocationStep>,
}

/// Represents a step in a workflow invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationStep {
    /// The ID of this step
    pub id: String,
    
    /// The step definition from the workflow
    pub workflow_step_id: String,
    
    /// The current state of this step
    pub state: WorkflowStepState,
    
    /// The job associated with this step, if any
    pub job_id: Option<String>,
    
    /// Output datasets from this step
    pub outputs: Option<HashMap<String, GalaxyDataReference>>,
    
    /// When this step was created
    pub create_time: String,
    
    /// When this step was last updated
    pub update_time: String,
}

/// The possible states of a workflow step execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStepState {
    /// The step is waiting for inputs
    #[serde(rename = "new")]
    New,
    
    /// The step is ready to run
    #[serde(rename = "ready")]
    Ready,
    
    /// The step is currently running
    #[serde(rename = "running")]
    Running,
    
    /// The step has completed successfully
    #[serde(rename = "ok")]
    Ok,
    
    /// The step execution failed
    #[serde(rename = "error")]
    Error,
    
    /// The step is paused
    #[serde(rename = "paused")]
    Paused,
    
    /// The step was skipped
    #[serde(rename = "skipped")]
    Skipped,
    
    /// A custom state not covered by the standard states
    #[serde(untagged)]
    Custom(String),
}

impl GalaxyWorkflow {
    /// Create a new Galaxy workflow with the given name
    pub fn new(name: &str) -> Self {
        Self {
            metadata: ResourceMetadata::new(name),
            steps: Vec::new(),
            connections: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            tags: Vec::new(),
            published: false,
            annotation: None,
            format_version: "0.1".to_string(),
            license: None,
            creator: None,
        }
    }
    
    /// Add a step to this workflow
    pub fn add_step(&mut self, step: WorkflowStep) -> &mut Self {
        self.steps.push(step);
        self
    }
    
    /// Add a connection between steps
    pub fn add_connection(&mut self, connection: WorkflowConnection) -> &mut Self {
        self.connections.push(connection);
        self
    }
    
    /// Add an input to this workflow
    pub fn add_input(&mut self, input: WorkflowInput) -> &mut Self {
        self.inputs.push(input);
        self
    }
    
    /// Add an output to this workflow
    pub fn add_output(&mut self, output: WorkflowOutput) -> &mut Self {
        self.outputs.push(output);
        self
    }
    
    /// Set the annotation for this workflow
    pub fn with_annotation(&mut self, annotation: &str) -> &mut Self {
        self.annotation = Some(annotation.to_string());
        self
    }
    
    /// Publish or unpublish this workflow
    pub fn set_published(&mut self, published: bool) -> &mut Self {
        self.published = published;
        self
    }
    
    /// Add a tag to this workflow
    pub fn add_tag(&mut self, tag: &str) -> &mut Self {
        self.tags.push(tag.to_string());
        self
    }
}

/// Helper function to create a tool step
pub fn create_tool_step(id: &str, tool_id: &str, position: StepPosition) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        step_type: WorkflowStepType::Tool,
        tool: Some(tool_id.to_string()),
        tool_version: None,
        input_connections: HashMap::new(),
        position,
        parameters: HashMap::new(),
        subworkflow_id: None,
        label: None,
        uuid: Some(Uuid::new_v4()),
    }
}

/// Helper function to create a data input step
pub fn create_input_step(id: &str, label: &str, position: StepPosition) -> WorkflowStep {
    WorkflowStep {
        id: id.to_string(),
        step_type: WorkflowStepType::DataInput,
        tool: None,
        tool_version: None,
        input_connections: HashMap::new(),
        position,
        parameters: HashMap::new(),
        subworkflow_id: None,
        label: Some(label.to_string()),
        uuid: Some(Uuid::new_v4()),
    }
}

/// Helper function to connect two steps
pub fn connect_steps(
    source_step: &str,
    source_output: &str,
    target_step: &str,
    target_input: &str,
) -> WorkflowConnection {
    WorkflowConnection {
        source_step_id: source_step.to_string(),
        source_output: source_output.to_string(),
        target_step_id: target_step.to_string(),
        target_input: target_input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_workflow() {
        let mut workflow = GalaxyWorkflow::new("Test Workflow");
        
        // Add input step
        let input_step = create_input_step("0", "Input Dataset", StepPosition { left: 10.0, top: 10.0 });
        workflow.add_step(input_step);
        
        // Add tool step
        let tool_step = create_tool_step("1", "toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.72+galaxy1", 
                                        StepPosition { left: 200.0, top: 10.0 });
        workflow.add_step(tool_step);
        
        // Connect steps
        let connection = connect_steps("0", "output", "1", "input_file");
        workflow.add_connection(connection);
        
        // Add output
        let output = WorkflowOutput {
            label: "FastQC Report".to_string(),
            source_step_id: "1".to_string(),
            output_name: "html_file".to_string(),
        };
        workflow.add_output(output);
        
        // Set annotation
        workflow.with_annotation("A simple test workflow that runs FastQC on an input dataset");
        
        // Add tags
        workflow.add_tag("test");
        workflow.add_tag("fastqc");
        
        // Verify workflow
        assert_eq!(workflow.metadata.name, "Test Workflow");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.connections.len(), 1);
        assert_eq!(workflow.outputs.len(), 1);
        assert_eq!(workflow.tags, vec!["test", "fastqc"]);
        assert_eq!(workflow.published, false);
        assert!(workflow.annotation.is_some());
    }
} 