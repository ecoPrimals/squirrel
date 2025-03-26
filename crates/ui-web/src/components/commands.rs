use crate::components::{Component, ComponentType};
use uuid::Uuid;

/// Command data structure
#[derive(Debug, Clone)]
pub struct CommandData {
    /// Command ID
    pub id: String,
    /// Command name
    pub name: String,
    /// Command description
    pub description: Option<String>,
    /// Command tags
    pub tags: Vec<String>,
    /// Command parameters 
    pub parameters: Vec<CommandParameter>,
}

/// Command parameter
#[derive(Debug, Clone)]
pub struct CommandParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Parameter description
    pub description: Option<String>,
    /// Whether parameter is required
    pub required: bool,
}

/// Command list component
pub struct CommandList {
    /// Component ID
    id: Uuid,
    /// Component title
    title: String,
    /// List of commands
    commands: Vec<CommandData>,
    /// Selected command ID
    selected_command: Option<String>,
    /// Commands loaded indicator
    loaded: bool,
}

impl CommandList {
    /// Create a new command list
    pub fn new(id: impl Into<Uuid>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            commands: Vec::new(),
            selected_command: None,
            loaded: false,
        }
    }

    /// Set commands for the list
    pub fn set_commands(&mut self, commands: Vec<CommandData>) {
        self.commands = commands;
        self.loaded = true;
    }

    /// Set selected command
    pub fn select_command(&mut self, command_id: impl Into<String>) {
        self.selected_command = Some(command_id.into());
    }

    /// Get the selected command
    pub fn selected_command(&self) -> Option<&CommandData> {
        if let Some(id) = &self.selected_command {
            self.commands.iter().find(|cmd| &cmd.id == id)
        } else {
            None
        }
    }

    /// Check if commands are loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

impl Component for CommandList {
    /// Get component ID
    fn id(&self) -> Uuid {
        self.id
    }

    /// Get component name
    fn name(&self) -> String {
        format!("CommandList: {}", self.title)
    }

    /// Get component type
    fn component_type(&self) -> ComponentType {
        ComponentType::Content
    }

    /// Render component to HTML
    fn render_html(&self) -> String {
        let command_cards = if self.commands.is_empty() {
            if self.loaded {
                "<div class=\"empty-message\">No commands available.</div>".to_string()
            } else {
                "<div class=\"loading\">Loading commands...</div>".to_string()
            }
        } else {
            let mut html = String::new();
            for command in &self.commands {
                let selected_class = if let Some(selected_id) = &self.selected_command {
                    if selected_id == &command.id {
                        " selected"
                    } else {
                        ""
                    }
                } else {
                    ""
                };

                let tags_html = if command.tags.is_empty() {
                    String::new()
                } else {
                    command.tags.iter()
                        .map(|tag| format!("<span class=\"tag\">{}</span>", htmlescape::encode_minimal(tag)))
                        .collect::<Vec<_>>()
                        .join("")
                };

                html.push_str(&format!(
                    "<div class=\"command-card{}\" data-command-id=\"{}\">\
                        <h4>{}</h4>\
                        <p>{}</p>\
                        <div class=\"tags\">{}</div>\
                    </div>",
                    selected_class,
                    htmlescape::encode_minimal(&command.id),
                    htmlescape::encode_minimal(&command.name),
                    htmlescape::encode_minimal(&command.description.as_deref().unwrap_or("No description")),
                    tags_html
                ));
            }
            html
        };

        format!(
            "<div class=\"commands-list\" id=\"{}\">{}</div>",
            self.id, command_cards
        )
    }
}

/// Command execution component
pub struct CommandExecution {
    /// Component ID
    id: Uuid,
    /// Command data
    command: Option<CommandData>,
    /// Parameters JSON
    parameters: String,
    /// Result of execution
    result: Option<CommandResult>,
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Result ID
    pub id: String,
    /// Execution status
    pub status: String,
    /// Result content
    pub content: String,
}

impl CommandExecution {
    /// Create a new command execution component
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self {
            id: id.into(),
            command: None,
            parameters: String::new(),
            result: None,
        }
    }

    /// Set the command to execute
    pub fn set_command(&mut self, command: CommandData) {
        // Generate default parameter template based on command parameters
        let mut params = std::collections::BTreeMap::new();
        for param in &command.parameters {
            let default_value = match param.param_type.as_str() {
                "string" => serde_json::Value::String("".to_string()),
                "number" | "integer" => serde_json::Value::Number(serde_json::Number::from(0)),
                "boolean" => serde_json::Value::Bool(false),
                "array" => serde_json::Value::Array(Vec::new()),
                "object" => serde_json::Value::Object(serde_json::Map::new()),
                _ => serde_json::Value::Null,
            };
            params.insert(param.name.clone(), default_value);
        }

        self.parameters = serde_json::to_string_pretty(&params).unwrap_or_else(|_| "{}".to_string());
        self.command = Some(command);
        self.result = None;
    }

    /// Set the parameters JSON
    pub fn set_parameters(&mut self, parameters: impl Into<String>) {
        self.parameters = parameters.into();
    }

    /// Get the parameters as a string
    pub fn parameters(&self) -> &str {
        &self.parameters
    }

    /// Set the execution result
    pub fn set_result(&mut self, result: CommandResult) {
        self.result = Some(result);
    }

    /// Get the current result
    pub fn result(&self) -> Option<&CommandResult> {
        self.result.as_ref()
    }

    /// Clear the current result
    pub fn clear_result(&mut self) {
        self.result = None;
    }
}

impl Component for CommandExecution {
    /// Get component ID
    fn id(&self) -> Uuid {
        self.id
    }

    /// Get component name
    fn name(&self) -> String {
        "Command Execution".to_string()
    }

    /// Get component type
    fn component_type(&self) -> ComponentType {
        ComponentType::Input
    }

    /// Render component to HTML
    fn render_html(&self) -> String {
        let command_select = if let Some(command) = &self.command {
            format!(
                "<div class=\"selected-command\">\
                    <h3>{}</h3>\
                    <p>{}</p>\
                </div>",
                htmlescape::encode_minimal(&command.name),
                htmlescape::encode_minimal(&command.description.as_deref().unwrap_or("No description"))
            )
        } else {
            "<select id=\"command-select\">\
                <option value=\"\">Select a command...</option>\
            </select>".to_string()
        };

        let result_html = if let Some(result) = &self.result {
            format!(
                "<div class=\"command-result\" id=\"command-result\">\
                    <h3>Command Result</h3>\
                    <div class=\"result-details\">\
                        <div class=\"result-header\">\
                            <span class=\"result-id\">ID: <span id=\"result-id\">{}</span></span>\
                            <span class=\"result-status\">Status: <span id=\"result-status\">{}</span></span>\
                        </div>\
                        <pre id=\"result-content\" class=\"result-content\">{}</pre>\
                    </div>\
                </div>",
                htmlescape::encode_minimal(&result.id),
                htmlescape::encode_minimal(&result.status),
                htmlescape::encode_minimal(&result.content)
            )
        } else {
            "<div class=\"command-result hidden\" id=\"command-result\">\
                <h3>Command Result</h3>\
                <div class=\"result-details\">\
                    <div class=\"result-header\">\
                        <span class=\"result-id\">ID: <span id=\"result-id\"></span></span>\
                        <span class=\"result-status\">Status: <span id=\"result-status\"></span></span>\
                    </div>\
                    <pre id=\"result-content\" class=\"result-content\"></pre>\
                </div>\
            </div>".to_string()
        };

        format!(
            "<div class=\"command-execution\" id=\"{}\">\
                <h3>Execute Command</h3>\
                <div class=\"form-group\">\
                    <label for=\"command-select\">Command:</label>\
                    {}\
                </div>\
                <div class=\"form-group\">\
                    <label for=\"command-parameters\">Parameters (JSON):</label>\
                    <textarea id=\"command-parameters\" rows=\"5\" placeholder='{{ \"param1\": \"value1\", \"param2\": \"value2\" }}'>{}</textarea>\
                </div>\
                <button id=\"execute-command\" class=\"primary-button\">Execute</button>\
            </div>\
            {}",
            self.id,
            command_select,
            htmlescape::encode_minimal(&self.parameters),
            result_html
        )
    }
} 