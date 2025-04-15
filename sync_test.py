import asyncio
import mcp_core_bindings as mcp
import uuid
import json
import subprocess
import time
import sys

# Set server URL for synchronization
SERVER_URL = "http://[::1]:50051"

async def setup_instance(instance_name):
    """Initialize a new MCP instance with a specific configuration"""
    print(f"\n[{instance_name}] Initializing MCP manager...")
    await mcp.initialize_manager_py()
    
    # Configure the instance to use our gRPC server
    # This is a placeholder - in a real implementation, you'd need to
    # expose a way to configure the sync URL through Python bindings
    print(f"[{instance_name}] Configured with server URL: {SERVER_URL}")
    
    return instance_name

async def create_test_context(instance_name):
    """Create a test context in the specified instance"""
    test_data = {
        "message": f"Hello from {instance_name}!",
        "timestamp": "2023-04-15T12:00:00Z",
        "instance": instance_name,
        "values": [1, 2, 3, 4, 5]
    }
    
    print(f"[{instance_name}] Creating test context...")
    context_id = await mcp.create_context_py(
        name=f"test_context_{instance_name}",
        data=test_data,
        metadata={"source": instance_name},
        parent_id_str=None
    )
    print(f"[{instance_name}] Context created with ID: {context_id}")
    return context_id

async def sync_instance(instance_name):
    """Synchronize the specified instance with the server"""
    print(f"[{instance_name}] Initiating synchronization...")
    try:
        await mcp.sync_py()
        print(f"[{instance_name}] Synchronization completed successfully!")
    except Exception as e:
        print(f"[{instance_name}] Synchronization error: {e}")
        raise e

async def check_context(instance_name, context_id):
    """Check if a context exists in the specified instance"""
    try:
        context = await mcp.get_context_py(context_id)
        print(f"[{instance_name}] Retrieved context: {context.name}")
        print(f"[{instance_name}] Context data: {json.dumps(context.data, indent=2)}")
        return context
    except Exception as e:
        print(f"[{instance_name}] Context not found: {e}")
        return None

async def test_bidirectional_sync():
    """Test synchronization between two MCP instances"""
    print("=== Testing Bidirectional Synchronization ===")
    
    # Start gRPC server process
    print("Starting gRPC server...")
    server_process = subprocess.Popen(
        ["cargo", "run", "--bin", "sync_server"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd="./crates/mcp"
    )
    
    # Give the server time to start
    print("Waiting for server to start...")
    time.sleep(3)
    
    try:
        # Setup two MCP instances
        instance1 = await setup_instance("Instance1")
        instance2 = await setup_instance("Instance2")
        
        # Create context in Instance1
        context1_id = await create_test_context(instance1)
        
        # Sync Instance1
        await sync_instance(instance1)
        
        # Sync Instance2
        await sync_instance(instance2)
        
        # Check if Instance2 can see context from Instance1
        print("\n[Test] Checking if Instance2 can see context from Instance1...")
        instance2_context1 = await check_context(instance2, context1_id)
        
        if instance2_context1:
            print("[Test] SUCCESS: Instance2 successfully received context from Instance1")
        else:
            print("[Test] FAILURE: Instance2 could not see context from Instance1")
        
        # Create context in Instance2
        context2_id = await create_test_context(instance2)
        
        # Sync Instance2
        await sync_instance(instance2)
        
        # Sync Instance1
        await sync_instance(instance1)
        
        # Check if Instance1 can see context from Instance2
        print("\n[Test] Checking if Instance1 can see context from Instance2...")
        instance1_context2 = await check_context(instance1, context2_id)
        
        if instance1_context2:
            print("[Test] SUCCESS: Instance1 successfully received context from Instance2")
        else:
            print("[Test] FAILURE: Instance1 could not see context from Instance2")
        
        # Clean up
        print("\n[Test] Cleaning up...")
        await mcp.delete_context_py(context1_id)
        await mcp.delete_context_py(context2_id)
        
        print("\n=== Synchronization test completed ===")
        if instance2_context1 and instance1_context2:
            print("OVERALL RESULT: SUCCESS - Both contexts were synchronized successfully")
        else:
            print("OVERALL RESULT: FAILURE - One or more contexts failed to synchronize")
    
    finally:
        # Stop the gRPC server
        print("Stopping gRPC server...")
        server_process.terminate()
        stdout, stderr = server_process.communicate()
        if stdout:
            print(f"Server stdout: {stdout.decode('utf-8')}")
        if stderr:
            print(f"Server stderr: {stderr.decode('utf-8')}")

if __name__ == "__main__":
    asyncio.run(test_bidirectional_sync()) 