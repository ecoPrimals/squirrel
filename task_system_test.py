#!/usr/bin/env python3
import asyncio
import logging
import unittest
import uuid
import os
import sys
import time

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("task_system_test")

# Try to import MCP bindings - required for these tests
try:
    import mcp_core_bindings as mcp
    HAS_BINDINGS = True
except ImportError:
    logger.error("mcp_core_bindings not found. Please build the bindings first.")
    logger.error("Run: cargo build -p mcp-pyo3-bindings")
    HAS_BINDINGS = False
    sys.exit(1)

class TaskSystemTestCase(unittest.IsolatedAsyncioTestCase):
    """End-to-end tests for the Task System in MCP"""
    
    async def asyncSetUp(self):
        """Initialize MCP context manager and task manager"""
        logger.info("Initializing MCP for tests")
        await mcp.initialize_manager_py()
        self.task_manager = mcp.task.PyTaskManager()
        
        # Create a test context for our tasks
        self.test_context_id = await mcp.create_context_py(
            "TaskSystemTest", 
            {"test_run": True, "timestamp": time.time()},
            None,
            None
        )
        logger.info(f"Created test context: {self.test_context_id}")
    
    async def test_basic_task_lifecycle(self):
        """Test basic task creation, updating, and completion"""
        # Create a new task
        task = mcp.task.PyTask("Test Basic Task")
        task.description = "A test task for basic lifecycle"
        task.input_data = {"test_param": "value1", "number": 42}
        task.context_id = self.test_context_id
        
        # Save task to the manager
        task_id = await self.task_manager.create_task(task)
        logger.info(f"Created task: {task_id}")
        
        # Retrieve the task
        retrieved_task = await self.task_manager.get_task(task_id)
        self.assertEqual(retrieved_task.name, "Test Basic Task")
        self.assertEqual(retrieved_task.context_id, self.test_context_id)
        
        # Assign the task
        await self.task_manager.assign_task(
            task_id, 
            "test-agent-1", 
            mcp.task.agent_type.SYSTEM
        )
        
        # Update progress
        await self.task_manager.update_progress(
            task_id,
            50,
            "Half-way through the task"
        )
        
        # Complete the task
        output_data = {"result": "success", "processed_items": 100}
        await self.task_manager.complete_task(
            task_id,
            output_data,
            {"execution_time_ms": 150}
        )
        
        # Verify final state
        completed_task = await self.task_manager.get_task(task_id)
        self.assertEqual(completed_task.status, mcp.task.status.COMPLETED)
        self.assertEqual(completed_task.progress_percent, 100)
        self.assertTrue(completed_task.is_complete())
    
    async def test_task_failure(self):
        """Test task failure handling"""
        # Create a task that will fail
        task = mcp.task.PyTask("Failing Task")
        task.description = "A task designed to fail"
        task.context_id = self.test_context_id
        
        # Save and start the task
        task_id = await self.task_manager.create_task(task)
        await self.task_manager.assign_task(
            task_id, 
            "test-agent-1", 
            mcp.task.agent_type.SYSTEM
        )
        
        # Update status to running
        retrieved_task = await self.task_manager.get_task(task_id)
        retrieved_task.mark_running()
        await self.task_manager.update_task(task_id, retrieved_task)
        
        # Fail the task
        error_message = "Task failed due to simulated error"
        await self.task_manager.fail_task(task_id, error_message)
        
        # Verify failure
        failed_task = await self.task_manager.get_task(task_id)
        self.assertEqual(failed_task.status, mcp.task.status.FAILED)
        self.assertEqual(failed_task.error_message, error_message)
        self.assertTrue(failed_task.is_failed())
    
    async def test_task_dependencies(self):
        """Test task dependencies and prerequisites"""
        # Create parent tasks
        parent_task1 = mcp.task.PyTask("Parent Task 1")
        parent_task1.context_id = self.test_context_id
        parent_id1 = await self.task_manager.create_task(parent_task1)
        
        parent_task2 = mcp.task.PyTask("Parent Task 2")
        parent_task2.context_id = self.test_context_id
        parent_id2 = await self.task_manager.create_task(parent_task2)
        
        # Create child task with prerequisites
        child_task = mcp.task.PyTask("Child Task")
        child_task.context_id = self.test_context_id
        child_task.prerequisite_task_ids = [parent_id1, parent_id2]
        child_id = await self.task_manager.create_task(child_task)
        
        # Check prerequisites (should be false since parents aren't complete)
        prerequisites_met = await self.task_manager.check_prerequisites(child_id)
        self.assertFalse(prerequisites_met)
        
        # Complete first parent
        await self.task_manager.complete_task(parent_id1, {"status": "done"}, None)
        
        # Check prerequisites again (should still be false)
        prerequisites_met = await self.task_manager.check_prerequisites(child_id)
        self.assertFalse(prerequisites_met)
        
        # Complete second parent
        await self.task_manager.complete_task(parent_id2, {"status": "done"}, None)
        
        # Check prerequisites again (should now be true)
        prerequisites_met = await self.task_manager.check_prerequisites(child_id)
        self.assertTrue(prerequisites_met)
        
        # Verify child task can now be assigned
        assignable_tasks = await self.task_manager.find_assignable_tasks()
        self.assertTrue(any(task.id == child_id for task in assignable_tasks))
    
    async def test_get_tasks_by_status(self):
        """Test retrieving tasks by status"""
        # Create several tasks with different statuses
        statuses = [
            mcp.task.status.CREATED,
            mcp.task.status.ASSIGNED,
            mcp.task.status.RUNNING,
            mcp.task.status.COMPLETED,
            mcp.task.status.FAILED
        ]
        
        task_ids = []
        for i, status in enumerate(statuses):
            task = mcp.task.PyTask(f"Status Test Task {i}")
            task.context_id = self.test_context_id
            task.status = status
            task_id = await self.task_manager.create_task(task)
            task_ids.append(task_id)
        
        # Get tasks for each status
        for status in statuses:
            status_tasks = await self.task_manager.get_tasks_by_status(status)
            # There should be at least one task with this status
            self.assertTrue(len(status_tasks) > 0)
            # All returned tasks should have the requested status
            for task in status_tasks:
                self.assertEqual(task.status, status)
    
    async def test_context_tasks(self):
        """Test retrieving tasks by context ID"""
        # Create several tasks in our test context
        for i in range(5):
            task = mcp.task.PyTask(f"Context Task {i}")
            task.context_id = self.test_context_id
            await self.task_manager.create_task(task)
        
        # Create a task in a different context
        other_context_id = await mcp.create_context_py(
            "OtherContext", 
            {"separate": True},
            None,
            None
        )
        other_task = mcp.task.PyTask("Other Context Task")
        other_task.context_id = other_context_id
        await self.task_manager.create_task(other_task)
        
        # Retrieve tasks for our test context
        context_tasks = await self.task_manager.get_context_tasks(self.test_context_id)
        
        # Verify we got the expected tasks
        self.assertGreaterEqual(len(context_tasks), 5)  # At least 5 from this test, maybe more from other tests
        for task in context_tasks:
            self.assertEqual(task.context_id, self.test_context_id)
        
        # Verify the other context task isn't included
        other_context_tasks = await self.task_manager.get_context_tasks(other_context_id)
        self.assertEqual(len(other_context_tasks), 1)
        self.assertEqual(other_context_tasks[0].context_id, other_context_id)

async def main():
    """Run the tests"""
    if not HAS_BINDINGS:
        return
    
    logger.info("Starting task system end-to-end tests")
    unittest.main(verbosity=2)

if __name__ == "__main__":
    asyncio.run(main()) 