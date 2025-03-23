/*!
 * Galaxy tool models.
 * 
 * This module defines the data models for Galaxy tools.
 */

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{ParameterDefinition, ParameterValue, ResourceMetadata};

/// Galaxy tool representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyTool {
    /// Tool metadata
    pub metadata: ResourceMetadata,
    
    /// Tool ID
    pub id: String,
    
    /// Tool version
    pub version: String,
    
    /// Tool type
    pub tool_type: ToolType,
    
    /// Tool inputs
    pub inputs: Vec<ParameterDefinition>,
    
    /// Tool outputs
    pub outputs: Vec<OutputDefinition>,
    
    /// Tool help text
    pub help: Option<String>,
    
    /// Tool requirements
    pub requirements: Vec<ToolRequirement>,
    
    /// Tool citations
    pub citations: Vec<Citation>,
    
    /// Tool panel section
    pub panel_section: Option<PanelSection>,
    
    /// Is the tool hidden
    pub hidden: bool,
}

/// Galaxy tool type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// Standard tool
    Tool,
    /// Data source tool
    DataSource,
    /// Data manager tool
    DataManager,
    /// Interactive tool
    Interactive,
    /// Custom tool type
    Custom(String),
}

/// Galaxy tool output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDefinition {
    /// Output name
    pub name: String,
    
    /// Output label
    pub label: Option<String>,
    
    /// Output format
    pub format: String,
    
    /// Output type
    pub output_type: OutputType,
    
    /// Whether the output is a collection
    pub is_collection: bool,
    
    /// Collection type if is_collection is true
    pub collection_type: Option<String>,
    
    /// Discover datasets pattern if is_collection is true
    pub discover_pattern: Option<String>,
}

/// Galaxy output type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    /// Standard output
    Data,
    /// Collection output
    Collection,
    /// Dataset output
    Dataset,
    /// Custom output type
    Custom(String),
}

/// Galaxy tool requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequirement {
    /// Requirement type
    pub requirement_type: String,
    
    /// Requirement version
    pub version: String,
    
    /// Requirement name
    pub name: String,
}

/// Galaxy tool citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Citation type
    pub citation_type: String,
    
    /// Citation value
    pub value: String,
}

/// Galaxy tool panel section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSection {
    /// Section ID
    pub id: String,
    
    /// Section name
    pub name: String,
    
    /// Section description
    pub description: Option<String>,
}

/// Galaxy tool execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionRequest {
    /// Tool ID
    pub tool_id: String,
    
    /// History ID to store results
    pub history_id: String,
    
    /// Tool parameters
    pub parameters: HashMap<String, ParameterValue>,
    
    /// Whether to create a new history
    pub create_history: bool,
}

/// Galaxy tool execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResponse {
    /// Job ID
    pub job_id: String,
    
    /// Outputs from the tool
    pub outputs: Vec<ToolOutput>,
    
    /// History ID where results are stored
    pub history_id: String,
    
    /// Job state
    pub state: JobState,
}

/// Galaxy tool output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Output name
    pub name: String,
    
    /// Output ID
    pub id: String,
    
    /// Output format
    pub format: String,
    
    /// Output URL
    pub url: Option<String>,
}

/// Galaxy job state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    /// Job is waiting to run
    Waiting,
    /// Job is queued
    Queued,
    /// Job is running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was deleted
    Deleted,
    /// Job state is unknown
    Unknown,
}

impl JobState {
    /// Check if the job state is terminal (completed, failed, deleted)
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Deleted)
    }
    
    /// Check if the job completed successfully
    pub fn is_successful(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Galaxy tool parameter mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameterMapping {
    /// Galaxy parameter name
    pub galaxy_name: String,
    
    /// MCP parameter name
    pub mcp_name: String,
    
    /// Parameter type for conversion
    pub param_type: String,
    
    /// Default value
    pub default: Option<ParameterValue>,
    
    /// Transformation function
    pub transform: Option<String>,
}

impl GalaxyTool {
    /// Create a new Galaxy tool
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            metadata: ResourceMetadata {
                name: name.into(),
                ..ResourceMetadata::default()
            },
            id: id.into(),
            version: version.into(),
            tool_type: ToolType::Tool,
            inputs: Vec::new(),
            outputs: Vec::new(),
            help: None,
            requirements: Vec::new(),
            citations: Vec::new(),
            panel_section: None,
            hidden: false,
        }
    }
    
    /// Add an input parameter to the tool
    pub fn add_input(&mut self, param: ParameterDefinition) -> &mut Self {
        self.inputs.push(param);
        self
    }
    
    /// Add an output to the tool
    pub fn add_output(&mut self, output: OutputDefinition) -> &mut Self {
        self.outputs.push(output);
        self
    }
    
    /// Set the tool help text
    pub fn with_help(&mut self, help: impl Into<String>) -> &mut Self {
        self.help = Some(help.into());
        self
    }
    
    /// Add a requirement to the tool
    pub fn add_requirement(
        &mut self,
        name: impl Into<String>,
        version: impl Into<String>,
        requirement_type: impl Into<String>,
    ) -> &mut Self {
        self.requirements.push(ToolRequirement {
            name: name.into(),
            version: version.into(),
            requirement_type: requirement_type.into(),
        });
        self
    }
    
    /// Add a citation to the tool
    pub fn add_citation(
        &mut self,
        citation_type: impl Into<String>,
        value: impl Into<String>,
    ) -> &mut Self {
        self.citations.push(Citation {
            citation_type: citation_type.into(),
            value: value.into(),
        });
        self
    }
} 