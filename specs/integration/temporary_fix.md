# Temporary Fix for ToolHandler Send/Sync Issues

While the comprehensive refactoring plan is being implemented, this document outlines a temporary fix to address the immediate Send/Sync issues with the `ToolHandler` trait, focusing on making the tests work correctly.

## Current Issues

1. The `handle` method takes an `Arc<McpAiToolsAdapter>`, but `McpAiToolsAdapter` contains an `Arc<dyn MCPInterface>` which is not guaranteed to be `Send + Sync`.
2. This causes issues in async contexts, particularly in tests where `MockToolHandler` can't be sent between threads.

## Temporary Solutions

### Option 1: Make MockMCP explicitly Send + Sync in tests

```rust
// In tests/mcp_ai_tools/mock_tests.rs

#[derive(Debug)]
struct MockMCP {
    response: String,
}

// Explicitly implement Send + Sync
unsafe impl Send for MockMCP {}
unsafe impl Sync for MockMCP {}

impl MockMCP {
    fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

impl MCPInterface for MockMCP {
    // Implementations remain the same
}
```

### Option 2: Modify MockToolHandler to use Weak references

```rust
// In tests/mcp_ai_tools/mock_tests.rs

#[derive(Debug)]
struct MockToolHandler {
    last_invocation: Arc<Mutex<Option<AiToolInvocation>>>,
    response: AiToolResponse,
}

impl MockToolHandler {
    fn new(response: AiToolResponse) -> Self {
        Self {
            last_invocation: Arc::new(Mutex::new(None)),
            response,
        }
    }
    
    fn get_last_invocation(&self) -> Option<AiToolInvocation> {
        self.last_invocation.lock().unwrap().clone()
    }
}

#[async_trait]
impl ToolHandler for MockToolHandler {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>, // Ignore the adapter parameter
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Store the invocation but don't use the adapter
        let mut lock = self.last_invocation.lock().unwrap();
        *lock = Some(invocation);
        Ok(self.response.clone())
    }
}
```

### Option 3: Use a thread-local mock adapter in tests

```rust
// In tests/mcp_ai_tools/mock_tests.rs
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static MOCK_ADAPTER: RefCell<Option<Rc<McpAiToolsAdapter>>> = RefCell::new(None);
}

#[derive(Debug)]
struct MockToolHandler {
    last_invocation: Arc<Mutex<Option<AiToolInvocation>>>,
    response: AiToolResponse,
}

impl MockToolHandler {
    // ... same as before ...
}

#[async_trait]
impl ToolHandler for MockToolHandler {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>, // Ignore the parameter
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Store the invocation
        let mut lock = self.last_invocation.lock().unwrap();
        *lock = Some(invocation);
        
        // Use thread-local adapter if needed
        // MOCK_ADAPTER.with(|adapter| {
        //     if let Some(adapter) = &*adapter.borrow() {
        //         // Use adapter here if needed
        //     }
        // });
        
        Ok(self.response.clone())
    }
}

// Set up the thread-local adapter before tests
fn setup_test_adapter() -> Arc<McpAiToolsAdapter> {
    let mcp = Arc::new(MockMCP::new("mock_response"));
    let config = McpAiToolsConfig::default();
    let adapter = Arc::new(McpAiToolsAdapter::with_config(mcp, config));
    
    // Store in thread-local
    MOCK_ADAPTER.with(|cell| {
        *cell.borrow_mut() = Some(Rc::from(adapter.clone()));
    });
    
    adapter
}
```

## Recommended Approach

For the immediate fix, we recommend **Option 2** as it:

1. Is the least intrusive
2. Doesn't require unsafe code
3. Doesn't introduce thread-local state
4. Works with the current test structure

The implementation simply ignores the adapter parameter in the mock implementation, which allows the tests to compile and run correctly without changing the trait signature or introducing thread-safety issues.

## Implementation Steps

1. Update the `MockToolHandler` implementation to ignore the adapter parameter
2. Update any tests that expect the adapter to be used in the mock
3. Modify any other test handlers to follow the same pattern
4. Add a comment indicating this is a temporary fix pending the full refactoring

## Example Update for mock_tests.rs

```rust
// In tests/mcp_ai_tools/mock_tests.rs

// TEMPORARY FIX: Ignoring adapter in handle method to avoid Send/Sync issues
// See specs/integration/tool_handler_refactoring.md for the long-term solution
#[async_trait]
impl ToolHandler for MockToolHandler {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>, // Adapter ignored to avoid Send/Sync issues
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Store the invocation
        let mut lock = self.last_invocation.lock().unwrap();
        *lock = Some(invocation);
        Ok(self.response.clone())
    }
}
```

This approach allows the tests to run correctly while the more comprehensive solution is being developed and implemented.

## Limitations

This temporary fix:

1. Only addresses the immediate testing issue
2. Doesn't solve the underlying design problem
3. May not work for all types of tool handlers
4. Should be replaced with the comprehensive solution outlined in the refactoring plan

The fix should be considered a short-term solution only, with the understanding that the full refactoring plan will be implemented to properly address the design issues. 