#!/usr/bin/env python3
import asyncio
import logging
import os
import sys
import time
import json
import grpc
from typing import Dict, Any, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("task_client_minimal")

# Path to the generated protobuf code
proto_dir = os.path.join(os.path.dirname(__file__), "generated")
sys.path.append(proto_dir)

# Try to import the generated protobuf code
try:
    from generated import mcp_task_pb2, mcp_task_pb2_grpc
except ImportError:
    logger.error("Generated protobuf code not found. Generating it now...")
    
    # Create directory for generated code
    os.makedirs(proto_dir, exist_ok=True)
    
    # Generate protobuf code
    proto_path = os.path.join(os.path.dirname(__file__), "proto")
    cmd = f"python -m grpc_tools.protoc -I{proto_path} --python_out={proto_dir} --grpc_python_out={proto_dir} {proto_path}/mcp_task.proto"
    
    logger.info(f"Running: {cmd}")
    if os.system(cmd) != 0:
        logger.error("Failed to generate protobuf code. Make sure grpcio-tools is installed.")
        logger.error("Run: pip install grpcio-tools")
        sys.exit(1)
    
    # Fix imports in generated code
    for filename in os.listdir(proto_dir):
        if filename.endswith('.py'):
            with open(os.path.join(proto_dir, filename), 'r') as f:
                content = f.read()
            
            # Fix imports
            content = content.replace('import mcp_task_pb2', 'from generated import mcp_task_pb2')
            
            with open(os.path.join(proto_dir, filename), 'w') as f:
                f.write(content)
    
    # Now try importing again
    sys.path.append(proto_dir)
    try:
        from generated import mcp_task_pb2, mcp_task_pb2_grpc
    except ImportError:
        logger.error("Failed to import generated code after generation.")
        sys.exit(1)

class TaskClient:
    """A client for interacting with the MCP Task Service."""
    
    def __init__(self, server_address="[::1]:50052"):
        """Initialize the task client.
        
        Args:
            server_address: The address of the task server.
        """
        self.server_address = server_address
        self.channel = None
        self.stub = None
    
    async def connect(self):
        """Connect to the task service."""
        self.channel = grpc.aio.insecure_channel(self.server_address)
        self.stub = mcp_task_pb2_grpc.TaskServiceStub(self.channel)
        logger.info(f"Connected to task service at {self.server_address}")
    
    async def create_task(
        self, 
        name: str, 
        description: str = "", 
        priority: int = 2,  # MEDIUM
        input_data: Optional[Dict[str, Any]] = None,
        metadata: Optional[Dict[str, Any]] = None,
        context_id: Optional[str] = None,
        prerequisite_task_ids: Optional[list] = None
    ) -> str:
        """Create a new task.
        
        Args:
            name: The name of the task.
            description: A description of the task.
            priority: Task priority (1=LOW, 2=MEDIUM, 3=HIGH, 4=CRITICAL).
            input_data: Input data for the task.
            metadata: Metadata for the task.
            context_id: The context ID to associate with the task.
            prerequisite_task_ids: List of task IDs that must be completed before this one.
            
        Returns:
            The ID of the created task.
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.CreateTaskRequest(
            name=name,
            description=description,
            priority=priority
        )
        
        # Add input data if provided
        if input_data:
            request.input_data = json.dumps(input_data).encode('utf-8')
        
        # Add metadata if provided
        if metadata:
            request.metadata = json.dumps(metadata).encode('utf-8')
        
        # Add context ID if provided
        if context_id:
            request.context_id = context_id
        
        # Add prerequisite task IDs if provided
        if prerequisite_task_ids:
            request.prerequisite_task_ids.extend(prerequisite_task_ids)
        
        # Send the request
        response = await self.stub.CreateTask(request)
        
        if not response.success:
            raise Exception(f"Failed to create task: {response.error_message}")
        
        logger.info(f"Created task: {response.task_id}")
        return response.task_id
    
    async def get_task(self, task_id: str) -> Dict[str, Any]:
        """Get a task by ID.
        
        Args:
            task_id: The ID of the task to get.
            
        Returns:
            The task data.
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.GetTaskRequest(
            task_id=task_id
        )
        
        # Send the request
        response = await self.stub.GetTask(request)
        
        if not response.success:
            raise Exception(f"Failed to get task: {response.error_message}")
        
        # Convert to dictionary with parsed JSON data
        task = response.task
        
        # Parse input/output data if present
        input_data = None
        if task.input_data:
            try:
                input_data = json.loads(task.input_data.decode('utf-8'))
            except:
                input_data = task.input_data
        
        output_data = None
        if task.output_data:
            try:
                output_data = json.loads(task.output_data.decode('utf-8'))
            except:
                output_data = task.output_data
        
        # Parse metadata if present
        metadata = None
        if task.metadata:
            try:
                metadata = json.loads(task.metadata.decode('utf-8'))
            except:
                metadata = task.metadata
        
        # Convert timestamps to ISO format
        created_at = task.created_at.ToDatetime().isoformat() if task.created_at else None
        updated_at = task.updated_at.ToDatetime().isoformat() if task.updated_at else None
        started_at = task.started_at.ToDatetime().isoformat() if task.started_at else None
        completed_at = task.completed_at.ToDatetime().isoformat() if task.completed_at else None
        
        # Build the task dictionary
        task_dict = {
            "id": task.id,
            "name": task.name,
            "description": task.description,
            "status": task.status,
            "priority": task.priority,
            "created_at": created_at,
            "updated_at": updated_at,
            "started_at": started_at,
            "completed_at": completed_at,
            "agent_id": task.agent_id,
            "agent_type": task.agent_type,
            "input_data": input_data,
            "output_data": output_data,
            "metadata": metadata,
            "error_message": task.error_message,
            "prerequisite_task_ids": list(task.prerequisite_task_ids),
            "dependent_task_ids": list(task.dependent_task_ids),
            "progress_percent": task.progress_percent,
            "progress_message": task.progress_message,
            "context_id": task.context_id
        }
        
        return task_dict
    
    async def assign_task(self, task_id: str, agent_id: str, agent_type: int = 4) -> None:
        """Assign a task to an agent.
        
        Args:
            task_id: The ID of the task to assign.
            agent_id: The ID of the agent to assign the task to.
            agent_type: The type of agent (4=SYSTEM by default).
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.AssignTaskRequest(
            task_id=task_id,
            agent_id=agent_id,
            agent_type=agent_type
        )
        
        # Send the request
        response = await self.stub.AssignTask(request)
        
        if not response.success:
            raise Exception(f"Failed to assign task: {response.error_message}")
        
        logger.info(f"Assigned task {task_id} to agent {agent_id}")
    
    async def report_progress(
        self, 
        task_id: str, 
        progress_percent: int, 
        progress_message: Optional[str] = None, 
        interim_results: Optional[Dict[str, Any]] = None
    ) -> None:
        """Report progress on a task.
        
        Args:
            task_id: The ID of the task to report progress on.
            progress_percent: The progress percentage (0-100).
            progress_message: A message describing the progress.
            interim_results: Interim results data.
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.ReportProgressRequest(
            task_id=task_id,
            progress_percent=progress_percent
        )
        
        # Add progress message if provided
        if progress_message:
            request.progress_message = progress_message
        
        # Add interim results if provided
        if interim_results:
            request.interim_results = json.dumps(interim_results).encode('utf-8')
        
        # Send the request
        response = await self.stub.ReportProgress(request)
        
        if not response.success:
            raise Exception(f"Failed to report progress: {response.error_message}")
        
        logger.info(f"Reported progress {progress_percent}% on task {task_id}")
    
    async def complete_task(
        self, 
        task_id: str, 
        output_data: Optional[Dict[str, Any]] = None, 
        metadata: Optional[Dict[str, Any]] = None
    ) -> None:
        """Complete a task with results.
        
        Args:
            task_id: The ID of the task to complete.
            output_data: Output data for the task.
            metadata: Updated metadata for the task.
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.CompleteTaskRequest(
            task_id=task_id
        )
        
        # Add output data if provided
        if output_data:
            request.output_data = json.dumps(output_data).encode('utf-8')
        
        # Add metadata if provided
        if metadata:
            request.metadata = json.dumps(metadata).encode('utf-8')
        
        # Send the request
        response = await self.stub.CompleteTask(request)
        
        if not response.success:
            raise Exception(f"Failed to complete task: {response.error_message}")
        
        logger.info(f"Completed task {task_id}")
    
    async def cancel_task(self, task_id: str, reason: Optional[str] = None) -> None:
        """Cancel a task.
        
        Args:
            task_id: The ID of the task to cancel.
            reason: The reason for cancellation.
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.CancelTaskRequest(
            task_id=task_id
        )
        
        # Add reason if provided
        if reason:
            request.reason = reason
        
        # Send the request
        response = await self.stub.CancelTask(request)
        
        if not response.success:
            raise Exception(f"Failed to cancel task: {response.error_message}")
        
        logger.info(f"Cancelled task {task_id}")
    
    async def list_tasks(
        self, 
        status: Optional[int] = None, 
        agent_id: Optional[str] = None, 
        agent_type: Optional[int] = None, 
        context_id: Optional[str] = None, 
        limit: int = 100, 
        offset: int = 0
    ) -> list:
        """List tasks matching filter criteria.
        
        Args:
            status: Filter by task status.
            agent_id: Filter by agent ID.
            agent_type: Filter by agent type.
            context_id: Filter by context ID.
            limit: Maximum number of tasks to return.
            offset: Offset for pagination.
            
        Returns:
            List of task dictionaries.
        """
        if self.stub is None:
            await self.connect()
        
        # Create the request
        request = mcp_task_pb2.ListTasksRequest(
            limit=limit,
            offset=offset
        )
        
        # Add filters if provided
        if status is not None:
            request.status = status
        
        if agent_id:
            request.agent_id = agent_id
        
        if agent_type is not None:
            request.agent_type = agent_type
        
        if context_id:
            request.context_id = context_id
        
        # Send the request
        response = await self.stub.ListTasks(request)
        
        if not response.success:
            raise Exception(f"Failed to list tasks: {response.error_message}")
        
        # Convert tasks to dictionaries
        tasks = []
        for task in response.tasks:
            # Parse input/output data if present
            input_data = None
            if task.input_data:
                try:
                    input_data = json.loads(task.input_data.decode('utf-8'))
                except:
                    input_data = task.input_data
            
            output_data = None
            if task.output_data:
                try:
                    output_data = json.loads(task.output_data.decode('utf-8'))
                except:
                    output_data = task.output_data
            
            # Parse metadata if present
            metadata = None
            if task.metadata:
                try:
                    metadata = json.loads(task.metadata.decode('utf-8'))
                except:
                    metadata = task.metadata
            
            # Convert timestamps to ISO format
            created_at = task.created_at.ToDatetime().isoformat() if task.created_at else None
            updated_at = task.updated_at.ToDatetime().isoformat() if task.updated_at else None
            started_at = task.started_at.ToDatetime().isoformat() if task.started_at else None
            completed_at = task.completed_at.ToDatetime().isoformat() if task.completed_at else None
            
            # Build the task dictionary
            task_dict = {
                "id": task.id,
                "name": task.name,
                "description": task.description,
                "status": task.status,
                "priority": task.priority,
                "created_at": created_at,
                "updated_at": updated_at,
                "started_at": started_at,
                "completed_at": completed_at,
                "agent_id": task.agent_id,
                "agent_type": task.agent_type,
                "input_data": input_data,
                "output_data": output_data,
                "metadata": metadata,
                "error_message": task.error_message,
                "prerequisite_task_ids": list(task.prerequisite_task_ids),
                "dependent_task_ids": list(task.dependent_task_ids),
                "progress_percent": task.progress_percent,
                "progress_message": task.progress_message,
                "context_id": task.context_id
            }
            
            tasks.append(task_dict)
        
        logger.info(f"Listed {len(tasks)} tasks (total count: {response.total_count})")
        return tasks
        
    async def close(self):
        """Close the channel."""
        if self.channel:
            await self.channel.close()
            logger.info("Closed channel")

async def demo():
    """Run a demonstration of the TaskClient."""
    client = TaskClient()
    await client.connect()
    
    try:
        # Create a context ID for this demo
        context_id = f"demo-{int(time.time())}"
        logger.info(f"Using context ID: {context_id}")
        
        # Create a task
        input_data = {
            "parameters": {
                "url": "https://example.com",
                "depth": 2
            },
            "settings": {
                "timeout": 30,
                "retry": True
            }
        }
        
        metadata = {
            "source": "task_client_minimal.py",
            "tags": ["demo", "test"]
        }
        
        task_id = await client.create_task(
            name="Demo Task",
            description="A demonstration task created by the minimal client",
            priority=3,  # HIGH
            input_data=input_data,
            metadata=metadata,
            context_id=context_id
        )
        
        # Get the task
        task = await client.get_task(task_id)
        print(f"\nCreated Task:")
        print(f"  ID: {task['id']}")
        print(f"  Name: {task['name']}")
        print(f"  Status: {task['status']}")
        print(f"  Priority: {task['priority']}")
        print(f"  Created At: {task['created_at']}")
        
        # Assign the task
        await client.assign_task(task_id, "demo-agent", 4)  # SYSTEM agent
        
        # Report progress
        for i in range(1, 5):
            progress = i * 20
            await client.report_progress(
                task_id,
                progress,
                f"Step {i} of 5 completed",
                {"items_processed": i * 10}
            )
            await asyncio.sleep(0.5)
        
        # Complete the task
        output_data = {
            "results": {
                "items_found": 42,
                "processing_time_ms": 1234,
                "success": True
            },
            "summary": "Task completed successfully"
        }
        
        updated_metadata = {
            "source": "task_client_minimal.py",
            "tags": ["demo", "test", "completed"],
            "completion_timestamp": time.time()
        }
        
        await client.complete_task(task_id, output_data, updated_metadata)
        
        # Get the completed task
        completed_task = await client.get_task(task_id)
        print(f"\nCompleted Task:")
        print(f"  ID: {completed_task['id']}")
        print(f"  Status: {completed_task['status']}")
        print(f"  Progress: {completed_task['progress_percent']}%")
        print(f"  Completed At: {completed_task['completed_at']}")
        print(f"  Output Data: {json.dumps(completed_task['output_data'], indent=2)}")
        
        # List tasks in the context
        tasks = await client.list_tasks(context_id=context_id)
        print(f"\nTasks in context {context_id}: {len(tasks)}")
        for i, task in enumerate(tasks):
            print(f"  {i+1}. {task['name']} (Status: {task['status']})")
        
        print("\nDemo completed successfully!")
    finally:
        await client.close()

if __name__ == "__main__":
    asyncio.run(demo()) 