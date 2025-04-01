// Example demonstrating the enhanced tool lifecycle management features
//
// This example shows how to use the enhanced tool lifecycle management
// components including state validation, enhanced recovery, and comprehensive cleanup.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use squirrel_mcp::tool::{
    AdvancedBackoffStrategy, AdvancedRecoveryAction, Capability, CleanupMethod, ComprehensiveCleanupHook,
    CompositeLifecycleHook, EnhancedRecoveryHook, EnhancedRecoveryStrategy, Parameter, ParameterType,
    ResourceType, StateTransitionValidator, StateValidationHook, ToolBuilder, ToolError,
    ToolExecutor, ToolLifecycleHook, ToolManager, ToolManagerBuilder, ToolManagerRecoveryExt,
    ToolState,
};

// Create a simple tool executor that can simulate errors
#[derive(Debug)]
struct ExampleToolExecutor {
    tool_id: String,
    capabilities: Vec<String>,
    should_fail: bool,
}

#[async_trait::async_trait]
impl ToolExecutor for ExampleToolExecutor {
    fn get_tool_id(&self) -> String {
        self.tool_id.clone()
    }

    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }

    async fn execute(
        &self,
        context: squirrel_mcp::tool::ToolContext,
    ) -> Result<squirrel_mcp::tool::ToolExecutionResult, ToolError> {
        if self.should_fail {
            return Err(ToolError::ExecutionFailed {
                tool_id: self.tool_id.clone(),
                reason: "Simulated failure".to_string(),
            });
        }

        Ok(squirrel_mcp::tool::ToolExecutionResult {
            tool_id: self.tool_id.clone(),
            capability: context.capability,
            request_id: context.request_id,
            status: squirrel_mcp::tool::ExecutionStatus::Success,
            output: Some(serde_json::json!({ "result": "success" })),
            error_message: None,
            execution_time_ms: 50,
            timestamp: chrono::Utc::now(),
        })
    }
}

// Create a custom recovery handler
#[derive(Debug)]
struct CustomRecoveryHandler;

impl squirrel_mcp::tool::EnhancedRecoveryHandler for CustomRecoveryHandler {
    // Manually implement the signature to match the trait
    fn handle_action<'a>(
        &'a self,
        tool_id: &'a str,
        action: &'a AdvancedRecoveryAction,
        _error: &'a ToolError,
        _tool_manager: &'a ToolManager,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            if let AdvancedRecoveryAction::Custom { name, params } = action {
                info!(
                    "Handling custom recovery action '{}' for tool {} with params: {:?}",
                    name, tool_id, params
                );
                // Simulate successful custom recovery
                Ok(true)
            } else {
                // We only handle custom actions
                Ok(false)
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting enhanced tool lifecycle management example");

    // Create the enhanced lifecycle management components
    
    // 1. State validation
    let state_validator = Arc::new(StateTransitionValidator::new());
    let state_hook = StateValidationHook::with_validator(state_validator.clone());
    
    // 2. Enhanced recovery
    let recovery_strategy = EnhancedRecoveryStrategy {
        max_attempts: 3,
        backoff_strategy: AdvancedBackoffStrategy::Exponential {
            base_ms: 100,  // Fast for demo purposes
            max_ms: 1000,
            jitter: 0.2,
        },
        recovery_actions: vec![
            AdvancedRecoveryAction::Reset,
            AdvancedRecoveryAction::Restart,
            AdvancedRecoveryAction::Custom {
                name: "specialized-recovery".to_string(),
                params: HashMap::new(),
            },
        ],
        stop_on_success: true,
        max_recovery_time_seconds: Some(5),
    };
    
    let recovery_hook = EnhancedRecoveryHook::new()
        .with_default_strategy(recovery_strategy)
        .add_handler(CustomRecoveryHandler);
    
    // 3. Comprehensive cleanup
    let cleanup_hook = ComprehensiveCleanupHook::new()
        .with_cleanup_timeout(2000);  // 2 seconds timeout
    
    // Create a composite lifecycle hook that includes all components
    let mut composite_hook = CompositeLifecycleHook::new();
    composite_hook.add_hook(state_hook);
    composite_hook.add_hook(cleanup_hook);
    
    // Create the tool manager with our enhanced hooks
    let tool_manager = ToolManagerBuilder::new()
        .lifecycle_hook(composite_hook)
        .build();

    info!("Tool manager created with enhanced lifecycle components");

    // Create a test tool
    let tool_result = ToolBuilder::new()
        .id("example-tool")
        .name("Example Tool")
        .description("A tool for demonstrating lifecycle management")
        .capability(Capability {
            name: "example-capability".to_string(),
            description: "An example capability".to_string(),
            parameters: vec![Parameter {
                name: "input".to_string(),
                description: "Input parameter".to_string(),
                parameter_type: ParameterType::String,
                required: true,
            }],
            return_type: None,
        })
        .security_level(5)
        .build();

    // Unwrap the Result to get the Tool
    let tool = tool_result?;
    
    info!("Registering tool: {}", tool.name);

    // Register the tool with a normal executor
    let executor = ExampleToolExecutor {
        tool_id: tool.id.clone(),
        capabilities: vec!["example-capability".to_string()],
        should_fail: false,
    };

    tool_manager.register_tool(tool.clone(), executor).await?;
    info!("Tool registered successfully");

    // Register some resources for the tool
    let comprehensive_hook = ComprehensiveCleanupHook::new();
    
    // Register memory resource
    comprehensive_hook
        .register_resource(
            &tool.id,
            ResourceType::Memory,
            "example-memory",
            1024 * 1024, // 1MB
            HashMap::new(),
        )
        .await;
    
    // Register file resource
    comprehensive_hook
        .register_resource(
            &tool.id,
            ResourceType::File,
            "example-file",
            1024,
            HashMap::new(),
        )
        .await;
    
    info!("Resources registered for tool");

    // Activate the tool
    tool_manager.activate_tool(&tool.id).await?;
    info!("Tool activated");

    // Start the tool
    tool_manager.start_tool(&tool.id).await?;
    info!("Tool started");

    // Execute the tool
    let params = serde_json::json!({
        "input": "test input"
    });
    
    match tool_manager
        .execute_tool(&tool.id, "example-capability", params, None)
        .await
    {
        Ok(result) => {
            info!("Tool execution successful: {:?}", result.output);
        }
        Err(err) => {
            info!("Tool execution failed: {}", err);
        }
    }

    // Demonstrate state transition validation by attempting invalid transitions
    let validator = state_validator.clone();
    
    info!("Current tool state: {:?}", tool_manager.get_tool_state(&tool.id).await);
    
    // Try an invalid transition
    match validator
        .validate_transition(
            &tool.id,
            &ToolState::Started,
            &ToolState::Registered,
            Some("Demo invalid transition".to_string()),
        )
        .await
    {
        Ok(_) => {
            info!("Transition validation succeeded (should not happen)");
        }
        Err(err) => {
            info!("Transition validation failed as expected: {}", err);
        }
    }

    // Try a valid transition
    match validator
        .validate_transition(
            &tool.id,
            &ToolState::Started,
            &ToolState::Stopped,
            Some("Demo valid transition".to_string()),
        )
        .await
    {
        Ok(_) => {
            info!("Transition validation succeeded as expected");
        }
        Err(err) => {
            info!("Transition validation failed: {}", err);
        }
    }

    // Stop the tool
    tool_manager.stop_tool(&tool.id).await?;
    info!("Tool stopped");

    // Test the tool recovery flow with a failing executor
    info!("\nTesting recovery flows...");
    
    // Create a new tool with a failing executor
    let failing_tool_result = ToolBuilder::new()
        .id("failing-tool")
        .name("Failing Tool")
        .description("A tool that fails for recovery testing")
        .capability(Capability {
            name: "fail-capability".to_string(),
            description: "A capability that always fails".to_string(),
            parameters: vec![],
            return_type: None,
        })
        .security_level(1)
        .build();
        
    // Unwrap the Result to get the Tool
    let failing_tool = failing_tool_result?;

    let failing_executor = ExampleToolExecutor {
        tool_id: failing_tool.id.clone(),
        capabilities: vec!["fail-capability".to_string()],
        should_fail: true,  // This will cause execution to fail
    };

    // Register with enhanced recovery hook
    let tool_manager_with_recovery = ToolManagerBuilder::new()
        .lifecycle_hook(recovery_hook)
        .build();

    tool_manager_with_recovery.register_tool(failing_tool.clone(), failing_executor).await?;
    tool_manager_with_recovery.activate_tool(&failing_tool.id).await?;
    tool_manager_with_recovery.start_tool(&failing_tool.id).await?;

    info!("Registered and started failing tool. Testing execution with recovery...");

    // Execute the failing tool - this should trigger recovery
    match tool_manager_with_recovery
        .execute_tool(
            &failing_tool.id,
            "fail-capability",
            serde_json::json!({}),
            None,
        )
        .await
    {
        Ok(result) => {
            info!("Tool execution somehow succeeded despite failure: {:?}", result);
        }
        Err(err) => {
            info!("Tool execution failed as expected (after recovery attempts): {}", err);
        }
    }

    // Test the cleanup flow
    info!("\nTesting comprehensive cleanup...");
    
    // Create a tool with registered resources
    let cleanup_tool_result = ToolBuilder::new()
        .id("cleanup-test-tool")
        .name("Cleanup Test Tool")
        .description("A tool for testing comprehensive cleanup")
        .capability(Capability {
            name: "test-capability".to_string(),
            description: "Test capability".to_string(),
            parameters: vec![],
            return_type: None,
        })
        .security_level(1)
        .build();
        
    // Unwrap the Result to get the Tool
    let cleanup_tool = cleanup_tool_result?;

    let tool_manager_with_cleanup = ToolManagerBuilder::new()
        .lifecycle_hook(ComprehensiveCleanupHook::new())
        .build();

    // Register a simple executor
    let basic_executor = ExampleToolExecutor {
        tool_id: cleanup_tool.id.clone(),
        capabilities: vec!["test-capability".to_string()],
        should_fail: false,
    };

    // Register tool and resources
    tool_manager_with_cleanup.register_tool(cleanup_tool.clone(), basic_executor).await?;
    
    // Create our own cleanup hook since ToolManager doesn't expose its hooks
    let cleanup_hook = ComprehensiveCleanupHook::new();
    
    // Register various resources
    cleanup_hook
        .register_resource(
            &cleanup_tool.id,
            ResourceType::Memory,
            "test-memory",
            2048 * 1024,  // 2MB
            HashMap::new(),
        )
        .await;
        
    cleanup_hook
        .register_resource(
            &cleanup_tool.id,
            ResourceType::File,
            "test-file-1",
            4096,
            HashMap::new(),
        )
        .await;
        
    cleanup_hook
        .register_resource(
            &cleanup_tool.id,
            ResourceType::Network,
            "test-connection",
            0,
            HashMap::new(),
        )
        .await;
    
    info!("Registered cleanup test tool with various resources");
    
    // Test listing resources
    let resources = cleanup_hook.get_active_resources(&cleanup_tool.id).await;
    info!("Registered resources: {:?}", resources);
    
    // Test cleanup with different methods
    info!("Testing cleanup with normal method...");
    let result = cleanup_hook
        .cleanup_tool_resources(&cleanup_tool.id, CleanupMethod::Normal)
        .await;
    info!("Normal cleanup result: {:?}", result);
    
    // Register more resources
    cleanup_hook
        .register_resource(
            &cleanup_tool.id,
            ResourceType::File,
            "temp-file",
            1024,
            HashMap::new(),
        )
        .await;
    
    info!("Testing cleanup with forced method...");
    let result = cleanup_hook
        .cleanup_tool_resources(&cleanup_tool.id, CleanupMethod::Forced)
        .await;
    info!("Forced cleanup result: {:?}", result);
    
    // Verify all resources are cleaned up
    let remaining = cleanup_hook.get_active_resources(&cleanup_tool.id).await;
    info!("Remaining resources after cleanup: {:?}", remaining);
    
    // Test the uninstall flow with a clean workflow
    info!("\nTesting complete tool uninstall workflow...");
    
    // The final cleanup should handle removing all resources and deregistering the tool
    match tool_manager_with_cleanup.unregister_tool(&cleanup_tool.id).await {
        Ok(_) => {
            info!("Tool unregistered successfully");
        }
        Err(err) => {
            info!("Tool unregistration failed: {}", err);
        }
    }
    
    info!("Enhanced tool lifecycle example completed successfully");
    
    Ok(())
} 