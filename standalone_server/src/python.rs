#[cfg(feature = "python")]
use pyo3::prelude::*;
use anyhow::Result;
use std::time::Duration;
use uuid::Uuid;

#[cfg(feature = "python")]
#[pymodule]
fn taskserver_standalone(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TaskClient>()?;
    m.add_class::<PyTask>()?;
    m.add_class::<TaskStatus>()?;
    m.add_class::<TaskPriority>()?;
    m.add_class::<AgentType>()?;
    m.add_class::<TaskWatcher>()?;
    Ok(())
}

#[cfg(feature = "python")]
#[pyclass(name = "TaskStatus")]
#[derive(Clone, Debug)]
pub enum TaskStatus {
    #[pyo3(name = "UNSPECIFIED")]
    Unspecified = 0,
    #[pyo3(name = "CREATED")]
    Created = 1,
    #[pyo3(name = "ASSIGNED")]
    Assigned = 2,
    #[pyo3(name = "RUNNING")]
    Running = 3,
    #[pyo3(name = "COMPLETED")]
    Completed = 4,
    #[pyo3(name = "FAILED")]
    Failed = 5,
    #[pyo3(name = "CANCELLED")]
    Cancelled = 6,
    #[pyo3(name = "PENDING")]
    Pending = 7,
}

#[cfg(feature = "python")]
#[pyclass(name = "TaskPriority")]
#[derive(Clone, Debug)]
pub enum TaskPriority {
    #[pyo3(name = "UNSPECIFIED")]
    Unspecified = 0,
    #[pyo3(name = "LOW")]
    Low = 1,
    #[pyo3(name = "MEDIUM")]
    Medium = 2,
    #[pyo3(name = "HIGH")]
    High = 3,
    #[pyo3(name = "CRITICAL")]
    Critical = 4,
}

#[cfg(feature = "python")]
#[pyclass(name = "AgentType")]
#[derive(Clone, Debug)]
pub enum AgentType {
    #[pyo3(name = "UNSPECIFIED")]
    Unspecified = 0,
    #[pyo3(name = "LOCAL_PYTHON")]
    LocalPython = 1,
    #[pyo3(name = "REMOTE_API")]
    RemoteApi = 2,
    #[pyo3(name = "UI")]
    Ui = 3,
    #[pyo3(name = "SYSTEM")]
    System = 4,
    #[pyo3(name = "CUSTOM")]
    Custom = 5,
}

#[cfg(feature = "python")]
#[pyclass(name = "Task")]
#[derive(Clone, Debug)]
pub struct PyTask {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub status: u32,
    #[pyo3(get)]
    pub priority: u32,
    #[pyo3(get)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[pyo3(get)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[pyo3(get)]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[pyo3(get)]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[pyo3(get)]
    pub agent_id: String,
    #[pyo3(get)]
    pub agent_type: u32,
    #[pyo3(get)]
    pub progress_percent: i32,
    #[pyo3(get)]
    pub progress_message: String,
    #[pyo3(get)]
    pub error_message: String,
    #[pyo3(get)]
    pub context_id: String,
}

#[cfg(feature = "python")]
impl From<crate::mcp_task::Task> for PyTask {
    fn from(task: crate::mcp_task::Task) -> Self {
        fn timestamp_to_datetime(ts: Option<prost_types::Timestamp>) -> Option<chrono::DateTime<chrono::Utc>> {
            ts.map(|t| {
                let seconds = t.seconds;
                let nanos = t.nanos as u32;
                chrono::DateTime::from_timestamp(seconds, nanos)
            }).flatten()
        }

        Self {
            id: task.id,
            name: task.name,
            description: task.description,
            status: task.status as u32,
            priority: task.priority as u32,
            created_at: timestamp_to_datetime(task.created_at),
            updated_at: timestamp_to_datetime(task.updated_at),
            started_at: timestamp_to_datetime(task.started_at),
            completed_at: timestamp_to_datetime(task.completed_at),
            agent_id: task.agent_id,
            agent_type: task.agent_type as u32,
            progress_percent: task.progress_percent,
            progress_message: task.progress_message,
            error_message: task.error_message,
            context_id: task.context_id,
        }
    }
}

#[cfg(feature = "python")]
#[pyclass(name = "TaskClient")]
pub struct TaskClient {
    address: String,
}

#[cfg(feature = "python")]
#[pyclass(name = "TaskWatcher")]
pub struct TaskWatcher {
    address: String,
    task_id: String,
    include_initial_state: bool,
    timeout_seconds: i32,
    only_watchable: bool,
    filter_updates: bool,
}

#[cfg(feature = "python")]
#[pymethods]
impl TaskClient {
    #[new]
    fn new(address: Option<String>) -> Self {
        Self {
            address: address.unwrap_or_else(|| "http://[::1]:50052".to_string()),
        }
    }

    fn create_task(&self, 
        py: Python,
        name: String, 
        description: String, 
        priority: u32,
        input_data: Option<String>,
        metadata: Option<String>,
        context_id: Option<String>,
        agent_id: Option<String>,
        agent_type: Option<u32>
    ) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            // Convert input data and metadata to bytes
            let input_bytes = if let Some(data) = input_data {
                data.as_bytes().to_vec()
            } else {
                Vec::new()
            };

            let metadata_bytes = if let Some(data) = metadata {
                data.as_bytes().to_vec()
            } else {
                Vec::new()
            };

            let request = tonic::Request::new(crate::mcp_task::CreateTaskRequest {
                name,
                description,
                priority,
                input_data: input_bytes,
                metadata: metadata_bytes,
                prerequisite_task_ids: Vec::new(),
                context_id: context_id.unwrap_or_default(),
                agent_id: agent_id.unwrap_or_default(),
                agent_type: agent_type.unwrap_or(0),
            });

            let response = client.create_task(request).await?;
            let task_id = response.into_inner().task_id;
            Ok(Python::with_gil(|_py| task_id))
        })
    }

    fn get_task(&self, py: Python, task_id: String) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            let request = tonic::Request::new(crate::mcp_task::GetTaskRequest {
                task_id,
            });

            let response = client.get_task(request).await?;
            let task = response.into_inner().task
                .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

            let py_task = PyTask::from(task);
            Ok(Python::with_gil(|py| py_task.into_py(py)))
        })
    }

    fn list_tasks(&self, 
        py: Python,
        status: Option<u32>,
        agent_id: Option<String>,
        agent_type: Option<u32>,
        context_id: Option<String>,
        limit: Option<i32>,
        offset: Option<i32>
    ) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            let request = tonic::Request::new(crate::mcp_task::ListTasksRequest {
                status: status.unwrap_or(0) as i32,
                agent_id: agent_id.unwrap_or_default(),
                agent_type: agent_type.unwrap_or(0) as i32,
                context_id: context_id.unwrap_or_default(),
                limit: limit.unwrap_or(100),
                offset: offset.unwrap_or(0),
            });

            let response = client.list_tasks(request).await?;
            let tasks = response.into_inner().tasks;
            
            let py_tasks: Vec<PyTask> = tasks.into_iter()
                .map(PyTask::from)
                .collect();

            Ok(Python::with_gil(|py| py_tasks.into_py(py)))
        })
    }

    fn assign_task(&self, 
        py: Python,
        task_id: String, 
        agent_id: String, 
        agent_type: u32
    ) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            let request = tonic::Request::new(crate::mcp_task::AssignTaskRequest {
                task_id,
                agent_id,
                agent_type: agent_type as i32,
            });

            let response = client.assign_task(request).await?;
            let success = response.into_inner().success;
            Ok(Python::with_gil(|_py| success))
        })
    }

    fn report_progress(&self, 
        py: Python,
        task_id: String, 
        progress_percent: i32, 
        progress_message: String
    ) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            let request = tonic::Request::new(crate::mcp_task::ReportProgressRequest {
                task_id,
                progress_percent,
                progress_message,
                interim_results: Vec::new(),
            });

            let response = client.report_progress(request).await?;
            let success = response.into_inner().success;
            Ok(Python::with_gil(|_py| success))
        })
    }

    fn complete_task(&self, 
        py: Python,
        task_id: String, 
        output_data: Option<String>,
        metadata: Option<String>
    ) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            // Convert output data and metadata to bytes
            let output_bytes = if let Some(data) = output_data {
                data.as_bytes().to_vec()
            } else {
                Vec::new()
            };

            let metadata_bytes = if let Some(data) = metadata {
                data.as_bytes().to_vec()
            } else {
                Vec::new()
            };

            let request = tonic::Request::new(crate::mcp_task::CompleteTaskRequest {
                task_id,
                output_data: output_bytes,
                metadata: metadata_bytes,
            });

            let response = client.complete_task(request).await?;
            let success = response.into_inner().success;
            Ok(Python::with_gil(|_py| success))
        })
    }

    fn cancel_task(&self, 
        py: Python,
        task_id: String, 
        reason: String
    ) -> PyResult<&PyAny> {
        let address = self.address.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;

            let request = tonic::Request::new(crate::mcp_task::CancelTaskRequest {
                task_id,
                reason,
            });

            let response = client.cancel_task(request).await?;
            let success = response.into_inner().success;
            Ok(Python::with_gil(|_py| success))
        })
    }

    fn watch_task(&self, 
        py: Python,
        task_id: String, 
        include_initial_state: Option<bool>,
        timeout_seconds: Option<i32>,
        only_watchable: Option<bool>,
        filter_updates: Option<bool>
    ) -> PyResult<Py<TaskWatcher>> {
        let watcher = TaskWatcher::new(
            self.address.clone(),
            task_id,
            include_initial_state.unwrap_or(true),
            timeout_seconds.unwrap_or(0),
            only_watchable.unwrap_or(false),
            filter_updates.unwrap_or(false)
        );

        Py::new(py, watcher)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl TaskWatcher {
    #[new]
    fn new(address: String, task_id: String, include_initial_state: bool, timeout_seconds: i32, only_watchable: bool, filter_updates: bool) -> Self {
        Self {
            address,
            task_id,
            include_initial_state,
            timeout_seconds,
            only_watchable,
            filter_updates,
        }
    }

    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'a>(slf: PyRef<'a, Self>, py: Python<'a>) -> PyResult<Option<&'a PyAny>> {
        let address = slf.address.clone();
        let task_id = slf.task_id.clone();
        let include_initial_state = slf.include_initial_state;
        let timeout_seconds = slf.timeout_seconds;
        let only_watchable = slf.only_watchable;
        let filter_updates = slf.filter_updates;

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut client = crate::client::connect(&address).await?;
            let mut stream = crate::client::watch_task(
                &mut client, 
                &task_id, 
                include_initial_state, 
                timeout_seconds,
                only_watchable,
                filter_updates
            ).await?;

            match stream.message().await? {
                Some(response) => {
                    if let Some(task) = response.task {
                        let py_task = PyTask::from(task);
                        Ok(Some(Python::with_gil(|py| py_task.into_py(py))))
                    } else {
                        // No task in response means we're done
                        Ok(None)
                    }
                },
                None => Ok(None) // Stream ended
            }
        })
    }
} 