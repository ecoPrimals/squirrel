---
title: UI Testing Patterns
version: 1.0.0
date: 2024-09-08
status: active
---

# UI Testing Patterns

## Overview

This document outlines the standardized testing patterns for the Squirrel UI components, focusing on best practices for React component testing with Jest. It addresses common issues encountered during the Vitest to Jest migration and provides solutions for improving test reliability.

## Test Utilities

We've created a set of standardized test utilities in `src/test-utils/test-helpers.ts` to improve test consistency and reduce code duplication.

### Key Utility Functions

#### `renderWithAct`

Solves the common React act() warning that occurs with async operations in tests.

```typescript
import { renderWithAct } from '../../test-utils/test-helpers';

it('renders the component', async () => {
  await renderWithAct(<MyComponent />);
  expect(screen.getByTestId('my-component')).toBeInTheDocument();
});
```

#### `createMockMcpStore`

Creates a standardized mock MCP store with sensible defaults:

```typescript
import { createMockMcpStore } from '../../test-utils/test-helpers';

// Basic usage with default connected state
(useMcpStore as any).mockReturnValue(createMockMcpStore());

// Override specific properties
(useMcpStore as any).mockReturnValue(createMockMcpStore({
  connectionStatus: 'disconnected',
  fetchTasks: mockFetchTasks
}));
```

#### `createMockTask` and Other Factories

Creates consistent test data:

```typescript
import { createMockTask } from '../../test-utils/test-helpers';

const mockTasks = [
  createMockTask({ id: 'task-1', command: 'test-command' }),
  createMockTask({ id: 'task-2', status: 'Completed' })
];
```

## Defensive Programming

### Handling Nullable Properties

All components should implement defensive programming to handle nullable or undefined values:

```typescript
// Defensive destructuring with default values
const store = useMcpStore() || {};
const { 
  activeTasks = [], 
  fetchTasks,
  connectionStatus = 'disconnected' 
} = store;

// Check function existence before calling
if (isConnected && typeof fetchTasks === 'function') {
  fetchTasks();
}
```

### Data Validation

Always validate data before using it:

```typescript
// Ensure activeTasks is an array
const formattedTasks = Array.isArray(activeTasks) 
  ? activeTasks.map(task => /* process task */)
  : [];

// Handle null/undefined values
if (!task) {
  return { id: 'unknown', name: 'Unknown Task' };
}

// Type validation
const progress = typeof task.progress === 'number' 
  ? Math.max(0, Math.min(100, task.progress)) 
  : 0;
```

## Testing Async Components

### Proper `act()` Usage

Always wrap component rendering and updates in `act()`:

```typescript
// Wrong - can cause act() warnings
render(<AsyncComponent />);
await waitFor(() => screen.getByTestId('loaded'));

// Correct - using our utility
await renderWithAct(<AsyncComponent />);
expect(screen.getByTestId('loaded')).toBeInTheDocument();
```

### Testing Edge Cases

Test components with various data states:

```typescript
it('handles null data gracefully', async () => {
  (useMcpStore as any).mockReturnValue(createMockMcpStore({
    activeTasks: null
  }));
  await renderWithAct(<McpTasks />);
  // Component should not crash
});
```

## Component Best Practices

### Data-Testid Attributes

Add testid attributes to critical elements:

```jsx
<div data-testid="mcp-tasks">
  <TaskList data-testid="task-list" />
</div>
```

### Error Boundaries

Implement error boundaries for components that might fail:

```jsx
<ErrorBoundary fallback={<div>Something went wrong</div>}>
  <McpTasks />
</ErrorBoundary>
```

## Migration Recommendations

When migrating or creating new tests:

1. Use the `renderWithAct` helper instead of manual act() wrapping
2. Use mock factories instead of inline mock objects
3. Test edge cases (null, undefined, unexpected types)
4. Add proper test-ids to components
5. Implement defensive programming in components

## Examples

See the following files for examples of these patterns:

- `src/components/__tests__/McpCommands.test.tsx`
- `src/components/__tests__/McpTasks.test.tsx`
- `src/components/McpTasks.tsx` 