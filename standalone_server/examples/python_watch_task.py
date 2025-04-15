#!/usr/bin/env python3
import asyncio
import sys
import uuid
import json
import time
from taskserver_standalone import TaskClient, TaskStatus, TaskPriority, AgentType

async def main():
    # Connect to the task server
    client = TaskClient("http://[::1]:50052")
    
    # Create a test task
    test_id = str(uuid.uuid4())
    task_id = await client.create_task(
        name=f"Python Watch Test {test_id}",
        description="Task for testing watch functionality from Python",
        priority=TaskPriority.MEDIUM,
        input_data=json.dumps({
            "test": True,
            "timestamp": time.time()
        }),
        metadata=json.dumps({
            "test": "python_watch_task",
        }),
        context_id="python-watch-test",
        agent_id="python-agent",
        agent_type=AgentType.LOCAL_PYTHON
    )
    
    print(f"Created task with ID: {task_id}")
    
    # Start watching the task in a separate task
    watch_task = asyncio.create_task(watch_task_updates(client, task_id))
    
    # Wait a moment for the watcher to start
    await asyncio.sleep(1)
    
    # Update task progress
    print("Reporting progress...")
    await client.report_progress(
        task_id=task_id,
        progress_percent=25,
        progress_message="Starting work"
    )
    
    # Wait a bit
    await asyncio.sleep(2)
    
    # Update more progress
    print("Reporting more progress...")
    await client.report_progress(
        task_id=task_id,
        progress_percent=75,
        progress_message="Almost done"
    )
    
    # Wait a bit
    await asyncio.sleep(2)
    
    # Complete the task
    print("Completing the task...")
    await client.complete_task(
        task_id=task_id,
        output_data=json.dumps({
            "result": "success",
            "completion_time": time.time()
        })
    )
    
    # Wait for the watcher to finish
    await watch_task
    
    print("Python watch task test completed successfully!")

async def watch_task_updates(client, task_id):
    print(f"Watching task {task_id} for updates...")
    
    update_count = 0
    async for task in client.watch_task(
        task_id, 
        include_initial_state=True, 
        timeout_seconds=30,
        only_watchable=False,
        filter_updates=False
    ):
        update_count += 1
        
        print(f"Task update {update_count}: Status={task.status}, Progress={task.progress_percent}%, Message='{task.progress_message}'")
        
        # If task is completed, we're done
        if task.status == TaskStatus.COMPLETED:
            print("Task completed, ending watch")
            break
    
    print(f"Watch stream ended. Received {update_count} updates")

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1) 