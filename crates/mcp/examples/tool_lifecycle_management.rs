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

#[async_trait::async_trait]
impl squirrel_mcp::tool::EnhancedRecoveryHandler for CustomRecoveryHandler {
    async fn handle_action(
        &self,
        tool_id: &str,
        action: &AdvancedRecoveryAction,
        _error: &ToolError,
        _tool_manager: &ToolManager,
    ) -> Result<bool, ToolError> {
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
    let tool = ToolBuilder::new()
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
    
    // Get valid next states
    let valid_states = validator.get_valid_next_states(&ToolState::Started).await;
    info!("Valid next states from Started: {:?}", valid_states);

    // Demo recovery by registering a failing tool
    info!("\nDemonstrating enhanced recovery:");
    
    // Create a failing tool
    let failing_tool = ToolBuilder::new()
        .id("failing-tool")
        .name("Failing Tool")
        .description("A tool that fails to demonstrate recovery")
        .capability(Capability {
            name: "failing-capability".to_string(),
            description: "A capability that fails".to_string(),
            parameters: vec![],
            return_type: None,
        })
        .security_level(5)
        .build();
    
    // Register with failing executor
    let failing_executor = ExampleToolExecutor {
        tool_id: failing_tool.id.clone(),
        capabilities: vec!["failing-capability".to_string()],
        should_fail: true,
    };
    
    tool_manager.register_tool(failing_tool.clone(), failing_executor).await?;
    info!("Failing tool registered");
    
    // Activate and start
    tool_manager.activate_tool(&failing_tool.id).await?;
    tool_manager.start_tool(&failing_tool.id).await?;
    
    // Try to execute - should fail
    match tool_manager
        .execute_tool(&failing_tool.id, "failing-capability", serde_json::json!({}), None)
        .await
    {
        Ok(_) => {
            info!("Failing tool execution succeeded (unexpected)");
        }
        Err(err) => {
            info!("Failing tool execution failed as expected: {}", err);
            
            // Perform recovery
            info!("Initiating enhanced recovery");
            match tool_manager.perform_enhanced_recovery(&failing_tool.id, &err, &recovery_hook).await {
                Ok(success) => {
                    info!("Recovery completed with success status: {}", success);
                }
                Err(recovery_err) => {
                    info!("Recovery failed: {}", recovery_err);
                }
            }
        }
    }
    
    // Demonstrate cleanup
    info!("\nDemonstrating comprehensive cleanup:");
    
    // Stop the tools
    tool_manager.stop_tool(&tool.id).await?;
    tool_manager.stop_tool(&failing_tool.id).await?;
    
    // Check for resource leaks
    let leaks = comprehensive_hook.check_for_leaks(&tool.id).await;
    info!("Detected resource leaks: {:?}", leaks);
    
    // Perform cleanup
    comprehensive_hook
        .cleanup_tool_resources(&tool.id, CleanupMethod::Normal)
        .await?;
    info!("Cleanup completed");
    
    // Verify all resources are cleaned up
    let active_resources = comprehensive_hook.get_active_resources(&tool.id).await;
    info!("Active resources after cleanup: {}", active_resources.len());
    
    // Unregister the tools
    tool_manager.unregister_tool(&tool.id).await?;
    tool_manager.unregister_tool(&failing_tool.id).await?;
    info!("Tools unregistered");
    
    info!("Example completed successfully");
    
    Ok(())
} 