# Remaining UI Test Issues

After our initial test fixes, we've made significant progress, but there are still several test issues remaining. Here's a comprehensive breakdown of what needs to be addressed next.

## 1. Component Tests

### McpPanel Component Issues
- **Problem**: McpPanel component tests are failing with "Element type is invalid" errors.
- **Root Cause**: The McpPanel component is likely imported incorrectly or not properly mocked.
- **Solution**: 
  - Check the component's import path
  - Verify the component is exported correctly
  - Create a proper mock for any dependencies the component uses
  - Update the tests to wrap the component in necessary providers

### WebAuth Component Issues
- **Problem**: Test expects "Protected Content" text that isn't rendered.
- **Root Cause**: The component's behavior has likely changed but the test wasn't updated.
- **Solution**:
  - Review the actual component output in the test
  - Update the test expectations to match the current component behavior
  - Add proper test data to ensure the component renders as expected

## 2. Store Tests

### mcpStore.test.ts Issues
- **Problem**: Several tests are failing because the store's state isn't updating as expected.
- **Root Cause**: The store implementation has likely changed, or the mocked functions aren't being called.
- **Solution**:
  - Update the test to properly initialize the store's state before each test
  - Fix the mock implementations to ensure they're correctly setting state
  - Update test expectations to match actual implementation

### dashboardStore.test.ts Issues
- **Problem**: Multiple test failures related to WebApiClient mocking and invoke calls.
- **Root Cause**: The mocking approach for WebApiClient and Tauri's invoke function isn't working.
- **Solution**:
  - Update the mocking approach to use the pattern that works with WebApiClient.test.ts
  - Create a proper factory function for creating mocked clients
  - Consistently use jest.mocked() for type safety
  - Fix the WebSocket subscription tests

## 3. Mock Implementation Issues

### General Mocking Problems
- **Problem**: Inconsistent mocking patterns across different test files.
- **Root Cause**: Different approaches to mocking the same dependencies.
- **Solution**:
  - Create standardized mock factory functions in test-utils
  - Use consistent patterns for mocking Tauri APIs
  - Ensure mocks are reset between tests
  - Add proper type safety to all mocks

### WebSocket Testing
- **Problem**: WebSocket connection tests are still failing in some components.
- **Root Cause**: Inconsistent approach to testing WebSocket connections.
- **Solution**:
  - Standardize the WebSocket testing approach using the pattern from WebApiClient.test.ts
  - Create helper functions for simulating WebSocket messages
  - Properly handle async behavior in WebSocket tests

## 4. Recommended Priority Order

1. **Fix mcpStore.test.ts** - This is a foundational store that other components depend on
2. **Fix dashboardStore.test.ts** - Focus on standardizing the mocking approach
3. **Fix McpPanel component tests** - Address the component rendering issues
4. **Fix WebAuth component tests** - Update test expectations to match component behavior

## 5. Long-term Improvements

1. **Create More Factory Functions**:
   - `createMockInvoke()` - For consistent Tauri invoke mocking
   - `createMockListen()` - For consistent Tauri event listening mocking
   - `createMockSubscription()` - For WebSocket subscription testing

2. **Standardize Test Patterns**:
   - Use consistent patterns for component rendering
   - Follow the same approach for async testing
   - Establish conventions for mocking stores in component tests

3. **Improve Test Utils**:
   - Enhance `renderWithAct` to handle more edge cases
   - Add snapshot testing capabilities
   - Create a standard test wrapper with all required providers

By addressing these issues systematically, we can bring the entire test suite to a consistent, passing state. 