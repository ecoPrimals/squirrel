// Import required modules
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyList};
use pyo3::PyResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use serde_json::Value;
use chrono::{DateTime, Utc};

// Import our utility functions
use crate::types::{py_object_to_json_value, json_value_to_py_object};

// Import the MCP task module
use squirrel_mcp::task::Task as RustTask;
use squirrel_mcp::task::TaskManager;
use squirrel_mcp::task::types::{TaskStatus, TaskPriority, AgentType};
use squirrel_mcp::error::MCPError;
use pyo3_asyncio::tokio::future_into_py;

// Error mapping
fn map_error(e: MCPError) -> PyErr {
    PyValueError::new_err(format!("Task operation failed: {}", e))
}

#[pyclass]
#[derive(Clone)]
struct PyTask {
    #[pyo3(get, set)]
    id: String,
    
    #[pyo3(get, set)]
    name: String,
    
    #[pyo3(get, set)]
    description: String,
    
    #[pyo3(get, set)]
    status: u32,
    
    #[pyo3(get, set)]
    priority: u32,
    
    #[pyo3(get, set)]
    created_at: DateTime<Utc>,
    
    #[pyo3(get, set)]
    updated_at: DateTime<Utc>,
    
    #[pyo3(get, set)]
    started_at: Option<DateTime<Utc>>,
    
    #[pyo3(get, set)]
    completed_at: Option<DateTime<Utc>>,
    
    #[pyo3(get, set)]
    agent_id: Option<String>,
    
    #[pyo3(get, set)]
    agent_type: u32,
    
    #[pyo3(get, set)]
    input_data: PyObject,
    
    #[pyo3(get, set)]
    output_data: Option<PyObject>,
    
    #[pyo3(get, set)]
    metadata: Option<PyObject>,
    
    #[pyo3(get, set)]
    error_message: Option<String>,
    
    #[pyo3(get, set)]
    prerequisite_task_ids: Vec<String>,
    
    #[pyo3(get, set)]
    dependent_task_ids: Vec<String>,
    
    #[pyo3(get, set)]
    progress_percent: i32,
    
    #[pyo3(get, set)]
    progress_message: Option<String>,
    
    #[pyo3(get, set)]
    context_id: Option<String>,
}

impl PyTask {
    // Convert from Rust Task to Python PyTask
    fn from_rust(py: Python<'_>, task: RustTask) -> PyResult<Self> {
        // Convert input_data to a Python object
        let input_data = if let Some(input_map) = &task.input_data {
            json_value_to_py_object(py, serde_json::to_value(input_map).unwrap_or(serde_json::Value::Null))?
        } else {
            py.None()
        };
        
        // Convert output_data to a Python object
        let output_data = if let Some(output_map) = &task.output_data {
            Some(json_value_to_py_object(py, serde_json::to_value(output_map).unwrap_or(serde_json::Value::Null))?)
        } else {
            None
        };
        
        // Convert metadata to a Python object
        let metadata = if let Some(meta_map) = &task.metadata {
            Some(json_value_to_py_object(py, serde_json::to_value(meta_map).unwrap_or(serde_json::Value::Null))?)
        } else {
            None
        };
        
        Ok(PyTask {
            id: task.id,
            name: task.name,
            description: task.description,
            status: task.status_code as u32,
            priority: task.priority_code as u32,
            created_at: task.created_at,
            updated_at: task.updated_at,
            // Task doesn't have started_at field, but we'll keep it in the binding
            started_at: None, 
            completed_at: task.completed_at,
            agent_id: task.agent_id,
            agent_type: task.agent_type as u32,
            input_data,
            output_data,
            metadata,
            error_message: task.error_message,
            // These fields are not directly in the Task struct, adapt them
            prerequisite_task_ids: task.prerequisites,
            dependent_task_ids: Vec::new(), // Task doesn't have this field
            progress_percent: task.progress as i32,
            progress_message: task.status_message,
            context_id: task.context_id,
        })
    }
    
    // Convert from Python PyTask to Rust Task
    fn to_rust(&self, py: Python<'_>) -> PyResult<RustTask> {
        // Convert input_data from Python to HashMap
        let input_data_value = py_object_to_json_value(py, self.input_data.clone())?;
        let input_data = if input_data_value.is_null() {
            None
        } else {
            match serde_json::from_value(input_data_value) {
                Ok(map) => Some(map),
                Err(_) => return Err(PyValueError::new_err("Failed to convert input_data to HashMap"))
            }
        };
        
        // Convert output_data from Python to HashMap
        let output_data = match &self.output_data {
            Some(data) => {
                let output_value = py_object_to_json_value(py, data.clone())?;
                if output_value.is_null() {
                    None
                } else {
                    match serde_json::from_value(output_value) {
                        Ok(map) => Some(map),
                        Err(_) => return Err(PyValueError::new_err("Failed to convert output_data to HashMap"))
                    }
                }
            },
            None => None,
        };
        
        // Convert metadata from Python to HashMap
        let metadata = match &self.metadata {
            Some(data) => {
                let meta_value = py_object_to_json_value(py, data.clone())?;
                if meta_value.is_null() {
                    None
                } else {
                    match serde_json::from_value(meta_value) {
                        Ok(map) => Some(map),
                        Err(_) => return Err(PyValueError::new_err("Failed to convert metadata to HashMap"))
                    }
                }
            },
            None => None,
        };
        
        let status_code = TaskStatus::from(self.status as i32);
        let priority_code = TaskPriority::from(self.priority as i32);
        let agent_type = AgentType::from(self.agent_type as i32);
        
        Ok(RustTask {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            status_code,
            priority_code,
            created_at: self.created_at,
            updated_at: self.updated_at,
            completed_at: self.completed_at,
            agent_id: self.agent_id.clone(),
            agent_type,
            input_data,
            output_data,
            metadata,
            error_message: self.error_message.clone(),
            prerequisites: self.prerequisite_task_ids.clone(),
            progress: self.progress_percent as f32,
            status_message: self.progress_message.clone(),
            context_id: self.context_id.clone(),
            // Fields with default values (that exist in RustTask but not in PyTask)
            parent_id: None,
            deadline: None,
            watchable: false,
            retry_count: 0,
            max_retries: 3,
        })
    }
}

#[pymethods]
impl PyTask {
    #[new]
    fn new(name: String) -> Self {
        Python::with_gil(|py| {
            PyTask {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                description: String::new(),
                status: TaskStatus::Pending as u32,
                priority: TaskPriority::Medium as u32,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                started_at: None,
                completed_at: None,
                agent_id: None,
                agent_type: AgentType::Unspecified as u32,
                input_data: py.None(),
                output_data: None,
                metadata: None,
                error_message: None,
                prerequisite_task_ids: Vec::new(),
                dependent_task_ids: Vec::new(),
                progress_percent: 0,
                progress_message: None,
                context_id: None,
            }
        })
    }
    
    fn is_complete(&self) -> bool {
        self.status == TaskStatus::Completed as u32
    }
    
    fn is_failed(&self) -> bool {
        self.status == TaskStatus::Failed as u32
    }
    
    fn mark_running(&mut self) {
        self.status = TaskStatus::Running as u32;
        self.updated_at = Utc::now();
        self.started_at = Some(Utc::now());
    }
    
    fn mark_completed(&mut self, _py: Python<'_>, output_data: Option<PyObject>) {
        self.status = TaskStatus::Completed as u32;
        self.updated_at = Utc::now();
        self.completed_at = Some(Utc::now());
        self.output_data = output_data;
        self.progress_percent = 100;
    }
    
    fn mark_failed(&mut self, error_message: String) {
        self.status = TaskStatus::Failed as u32;
        self.updated_at = Utc::now();
        self.error_message = Some(error_message);
    }
    
    fn update_progress(&mut self, percent: i32, message: Option<String>) {
        self.progress_percent = percent;
        self.progress_message = message;
    }
    
    fn assign(&mut self, agent_id: String, agent_type: u32) -> PyResult<()> {
        self.agent_id = Some(agent_id);
        self.agent_type = agent_type;
        Ok(())
    }
    
    fn __repr__(&self) -> String {
        format!("PyTask(id={}, name={}, status={})", 
                self.id, self.name, self.status)
    }
}

#[pyclass]
struct PyTaskManager {
    task_manager: Arc<TaskManager>,
}

#[pymethods]
impl PyTaskManager {
    #[new]
    fn new() -> Self {
        PyTaskManager {
            task_manager: Arc::new(TaskManager::new()),
        }
    }
    
    fn create_task<'p>(&self, py: Python<'p>, task: &PyTask) -> PyResult<&'p PyAny> {
        let rust_task = task.to_rust(py)?;
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.create_task(rust_task).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn get_task<'p>(&self, py: Python<'p>, task_id: String) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.get_task(&task_id).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn update_task<'p>(&self, py: Python<'p>, task_id: String, task: &PyTask) -> PyResult<&'p PyAny> {
        let rust_task = task.to_rust(py)?;
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.update_task(rust_task).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn assign_task<'p>(
        &self,
        py: Python<'p>,
        task_id: String,
        agent_id: String,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.assign_task(&task_id, &agent_id).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn update_progress<'p>(
        &self,
        py: Python<'p>,
        task_id: String,
        progress_percent: i32,
        progress_message: Option<String>,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.update_progress(
                &task_id, 
                progress_percent as f32, 
                progress_message
            ).await
            .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn complete_task<'p>(
        &self,
        py: Python<'p>,
        task_id: String,
        output_data: Option<PyObject>,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        // Convert output_data from Python to HashMap<String, String>
        let output_data_map: Option<HashMap<String, String>> = if let Some(data) = output_data {
            let output_value = py_object_to_json_value(py, data)?;
            if output_value.is_null() {
                None
            } else {
                match output_value {
                    serde_json::Value::Object(map) => {
                        let mut string_map = HashMap::new();
                        for (key, value) in map {
                            string_map.insert(key, value.to_string());
                        }
                        Some(string_map)
                    },
                    _ => return Err(PyValueError::new_err("Output data must be a dictionary"))
                }
            }
        } else {
            None
        };
        
        future_into_py(py, async move {
            let result = task_manager.complete_task(&task_id, output_data_map).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn fail_task<'p>(
        &self,
        py: Python<'p>,
        task_id: String,
        error_message: String,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.fail_task(&task_id, &error_message).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn cancel_task<'p>(
        &self,
        py: Python<'p>,
        task_id: String,
        reason: String,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.cancel_task(&task_id, &reason).await
                .map_err(map_error)?;
            Ok(Python::with_gil(|py| PyTask::from_rust(py, result).unwrap().into_py(py)))
        })
    }
    
    fn get_agent_tasks<'p>(
        &self,
        py: Python<'p>,
        agent_id: String,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.get_agent_tasks(&agent_id).await
                .map_err(map_error)?;
            
            // Convert Vec<RustTask> to Vec<PyObject>
            Ok(Python::with_gil(|py| {
                let py_tasks: Vec<PyObject> = result.into_iter()
                    .map(|task| PyTask::from_rust(py, task).unwrap().into_py(py))
                    .collect();
                py_tasks
            }))
        })
    }
    
    fn get_context_tasks<'p>(
        &self,
        py: Python<'p>,
        context_id: String,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.get_context_tasks(&context_id).await
                .map_err(map_error)?;
            
            // Convert Vec<RustTask> to Vec<PyObject>
            Ok(Python::with_gil(|py| {
                let py_tasks: Vec<PyObject> = result.into_iter()
                    .map(|task| PyTask::from_rust(py, task).unwrap().into_py(py))
                    .collect();
                py_tasks
            }))
        })
    }
    
    fn get_tasks_by_status<'p>(
        &self,
        py: Python<'p>,
        status: u32,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        let status_enum = TaskStatus::from(status as i32);
        
        future_into_py(py, async move {
            let result = task_manager.get_tasks_by_status(status_enum).await
                .map_err(map_error)?;
            
            // Convert Vec<RustTask> to Vec<PyObject>
            Ok(Python::with_gil(|py| {
                let py_tasks: Vec<PyObject> = result.into_iter()
                    .map(|task| PyTask::from_rust(py, task).unwrap().into_py(py))
                    .collect();
                py_tasks
            }))
        })
    }
    
    fn check_prerequisites<'p>(
        &self,
        py: Python<'p>,
        task_id: String,
    ) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            // First get the task by id
            let task = task_manager.get_task(&task_id).await
                .map_err(map_error)?;
            
            // Then check prerequisites on the task
            let result = task_manager.check_prerequisites(&task).await
                .map_err(map_error)?;
            
            // Return a bool
            Ok(Python::with_gil(|py| result.into_py(py)))
        })
    }
    
    fn find_assignable_tasks<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let task_manager = self.task_manager.clone();
        
        future_into_py(py, async move {
            let result = task_manager.find_assignable_tasks().await
                .map_err(map_error)?;
            
            // Convert Vec<RustTask> to Vec<PyObject>
            Ok(Python::with_gil(|py| {
                let py_tasks: Vec<PyObject> = result.into_iter()
                    .map(|task| PyTask::from_rust(py, task).unwrap().into_py(py))
                    .collect();
                py_tasks
            }))
        })
    }
}

// Define the module and expose task status, priority, and agent types
fn task_status(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add("STATUS_PENDING", TaskStatus::Pending as u32)?;
    m.add("STATUS_RUNNING", TaskStatus::Running as u32)?;
    m.add("STATUS_COMPLETED", TaskStatus::Completed as u32)?;
    m.add("STATUS_FAILED", TaskStatus::Failed as u32)?;
    m.add("STATUS_CANCELED", TaskStatus::Cancelled as u32)?;
    m.add("STATUS_WAITING", TaskStatus::Waiting as u32)?;
    Ok(())
}

fn task_priority(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Add standard priority levels
    m.add("PRIORITY_LOW", TaskPriority::Low as u32)?;
    m.add("PRIORITY_MEDIUM", TaskPriority::Medium as u32)?;
    m.add("PRIORITY_HIGH", TaskPriority::High as u32)?;
    m.add("PRIORITY_CRITICAL", TaskPriority::Critical as u32)?;
    
    // Since the enum doesn't have Lowest and Highest, let's add them with the closest values
    m.add("PRIORITY_LOWEST", TaskPriority::Low as u32)?; // Use Low instead
    m.add("PRIORITY_HIGHEST", TaskPriority::Critical as u32)?; // Use Critical instead
    
    Ok(())
}

fn agent_type(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add("AGENT_UNSPECIFIED", AgentType::Unspecified as u32)?;
    m.add("AGENT_HUMAN", AgentType::Human as u32)?;
    m.add("AGENT_AI", AgentType::AI as u32)?;
    m.add("AGENT_SYSTEM", AgentType::System as u32)?;
    m.add("AGENT_GENERAL", AgentType::General as u32)?;
    m.add("AGENT_DATA_PROCESSOR", AgentType::DataProcessor as u32)?;
    m.add("AGENT_FILE_HANDLER", AgentType::FileHandler as u32)?;
    Ok(())
}

// Main module initialization function
pub fn task(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // First make sure Python is initialized with correct library paths 
    crate::init_python()?;
    
    // Add the task submodule
    let task_module = PyModule::new(_py, "task")?;
    
    // Add task status constants
    task_status(_py, task_module)?;
    
    // Add task priority constants
    task_priority(_py, task_module)?;
    
    // Add agent type constants
    agent_type(_py, task_module)?;
    
    // Add task classes
    task_module.add_class::<PyTask>()?;
    task_module.add_class::<PyTaskManager>()?;
    
    // Add the task submodule to the main module
    m.add_submodule(task_module)?;
    
    Ok(())
} 