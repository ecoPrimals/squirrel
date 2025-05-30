# Vitest to Jest Migration Guide

**Version**: 1.4.0  
**Date**: 2024-05-11  
**Status**: In Progress

## Overview

This document outlines the process for migrating tests from Vitest to Jest in the Squirrel project. The migration is currently in progress, with several key components already successfully migrated.

## Migration Strategy

We are taking a systematic approach to migrate all tests from Vitest to Jest:

1. First, update all test utilities to use Jest instead of Vitest
2. Then update all test files to use Jest imports and syntax
3. Create compatibility layers where needed for smooth transition
4. Remove Vitest configuration files once migration is complete
5. Remove the compatibility layer once all tests are updated

## Migration Script

We've created an automated migration script at `scripts/migrate-vitest-to-jest.js` that can help automate the conversion process. The script:

1. Scans directories for test files
2. Identifies files using Vitest patterns
3. Automatically converts common patterns like:
   - Removing Vitest imports
   - Replacing `vi.*` calls with `jest.*` equivalents
   - Converting environment variable handling

### Usage

```bash
# Run a dry run (shows changes without applying them)
node scripts/migrate-vitest-to-jest.js --dry-run

# Convert a specific path
node scripts/migrate-vitest-to-jest.js --path crates/ui-tauri-react/src/components

# Convert a specific file
node scripts/migrate-vitest-to-jest.js --path crates/ui-tauri-react/src/services/WebApiClient.test.ts

# Convert all files in the default directory (crates/ui-tauri-react/src)
node scripts/migrate-vitest-to-jest.js
```

### Migration Statistics

The script will output statistics about the migration process:
- Files scanned
- Files modified
- Vitest imports removed
- vi.* calls replaced
- Environment variable replacements

## Common Migration Changes

### Import Statements

Replace Vitest imports with Jest equivalents:

```typescript
// Before - Vitest
import { describe, it, expect, vi, beforeEach } from 'vitest';

// After - Jest
// No explicit imports needed as Jest makes these available globally
```

### Mocking Functions

Replace Vitest's `vi` namespace with Jest's `jest` namespace:

```typescript
// Before - Vitest
const mockFn = vi.fn();
vi.mock('./someModule');
vi.spyOn(object, 'method');

// After - Jest
const mockFn = jest.fn();
jest.mock('./someModule');
jest.spyOn(object, 'method');
```

### Timer Mocks

```typescript
// Before - Vitest
vi.useFakeTimers();
vi.advanceTimersByTime(1000);
vi.useRealTimers();

// After - Jest
jest.useFakeTimers();
jest.advanceTimersByTime(1000);
jest.useRealTimers();
```

### Environment Variable Handling

```typescript
// Before - Vitest
vi.stubEnv('VITE_USE_WEB_API', 'true');
// ...test code...
vi.unstubAllEnvs();

// After - Jest
// Option 1: Use jest-env-utils.js compatibility layer
jest.stubEnv('VITE_USE_WEB_API', 'true');
// ...test code...
jest.unstubAllEnvs();

// Option 2: Direct process.env manipulation (recommended)
const originalValue = process.env.VITE_USE_WEB_API;
process.env.VITE_USE_WEB_API = 'true';
// ...test code...
process.env.VITE_USE_WEB_API = originalValue;
```

### Module Mocking

```typescript
// Before - Vitest
vi.mock('moduleName', () => {
  return {
    default: vi.fn(),
    namedExport: vi.fn()
  };
});

// After - Jest
jest.mock('moduleName', () => {
  return {
    __esModule: true,
    default: jest.fn(),
    namedExport: jest.fn()
  };
});
```

### Import Mocking Differences

Vitest's `importActual` needed to be replaced:

```typescript
// Before - Vitest
vi.mock('recharts', async () => {
  const actual = await vi.importActual('recharts');
  return {
    ...actual,
    // Overrides
  };
});

// After - Jest
// For Jest, we used a simpler approach
jest.mock('recharts', () => {
  return {
    // Mock all required components directly
    ResponsiveContainer: ({ children }) => <div data-testid="mock-container">{children}</div>,
    // Other components...
  };
});
```

## Configuration Updates

1. The Vitest configuration in `vite.config.ts` has been removed
2. Jest configuration is defined in `jest.config.mjs`
3. Added `jest-env-utils.js` for Vitest compatibility functions

## Compatibility Layers

To ease the transition, we've implemented:

1. **jest-env-utils.js**: Adds Vitest-compatible functions to Jest:
   - `jest.stubEnv` 
   - `jest.unstubEnv`
   - `jest.unstubAllEnvs`

These utilities help bridge the gap during migration, but should be replaced with direct `process.env` manipulation where possible.

## Current Progress

### Successfully Migrated

1. ✅ All the following test files have been converted from Vitest to Jest:
   - McpTasksPanel.test.tsx
   - ConnectionStatus.test.tsx
   - AIChat.test.tsx (with fixes to component selectors)
   - LanguageSwitcher.test.tsx
   - PerformanceMonitor.test.tsx 
   - EnhancedPluginManager.test.tsx
   - AIService.test.ts
   - WebApiClient.test.ts

2. ✅ Setup configuration in setupTests.ts to support Jest
3. ✅ Migration script created for automated conversions
4. ✅ All Vitest imports have been removed
5. ✅ All `vi.*` function calls have been replaced with Jest equivalents

### Still In Progress

1. ⏳ Fixing implementation-related test failures:
   - LanguageSwitcher component tests failing due to component implementation changes
   - EnhancedPluginManager tests failing due to null plugin handling 
   - Performance Monitor tests failing due to error message format
   - Some test failures appear to be unrelated to the testing framework

2. ⏳ Final cleanup:
   - Remove Vitest from package.json
   - Remove compatibility layers once all tests pass

## Examples

### Before (Vitest):

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import MyComponent from './MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });

  it('calls the API', async () => {
    const mockApi = vi.fn().mockResolvedValue({ data: 'test' });
    vi.mock('../api', () => ({
      fetchData: mockApi
    }));
    
    render(<MyComponent />);
    await vi.waitFor(() => {
      expect(mockApi).toHaveBeenCalled();
    });
  });
});
```

### After (Jest):

```typescript
// No imports needed for Jest globals
import { render, screen, waitFor } from '@testing-library/react';
import MyComponent from './MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });

  it('calls the API', async () => {
    const mockApi = jest.fn().mockResolvedValue({ data: 'test' });
    jest.mock('../api', () => ({
      fetchData: mockApi
    }));
    
    render(<MyComponent />);
    await waitFor(() => {
      expect(mockApi).toHaveBeenCalled();
    });
  });
});
```

## Implementation Plan

Here's our detailed implementation plan for completing the migration:

1. **Week 1: Infrastructure and Core Components (Current - Completed)**
   - ✅ Update package.json scripts
   - ✅ Create migration script
   - ✅ Add Jest compatibility layer for Vitest functions
   - ✅ Migrate critical components
   - ✅ Identify and fix failing tests

2. **Week 2: Fix Component Tests (Current)**
   - ✅ Run migration script on all test files
   - ⏳ Update component tests to work with the latest component implementations:
     - Fix LanguageSwitcher tests to match new component structure
     - Update EnhancedPluginManager to handle null plugins
     - Fix error message assertions in Performance Monitor tests

3. **Week 3: Cleanup and Documentation (Next)**
   - Remove Vitest dependencies
   - Remove compatibility layers
   - Update documentation
   - Add linting rules to prevent new Vitest code

## Next Steps

1. **Fix Specific Component Tests**:
   - EnhancedPluginManager.test.tsx: Update to handle null plugins by adding a check in the component:
     ```tsx
     // In EnhancedPluginManager.tsx
     {plugins && Array.isArray(plugins) && plugins.map((plugin) => (
       // Plugin rendering code
     ))}
     ```

   - LanguageSwitcher.test.tsx: Update tests to match the current component implementation, which may have changed since the tests were written.

   - PerformanceMonitor.test.tsx: Fix error message assertions to match the actual component output.

2. **Remove Vitest Dependencies**:
   - Update `package.json` to remove Vitest dependencies:
     ```diff
     "devDependencies": {
       // ...
     -  "vitest": "^x.x.x",
     }
     ```

3. **Remove Compatibility Layers**:
   - Once all tests pass, replace compatibility layer usage with direct `process.env` manipulation.

4. **Final Verification**:
   - Run all tests to ensure they pass with Jest.
   - Update documentation to reflect Jest-only approach.

## Troubleshooting

Common issues encountered:

1. **Environment Variables**: Vitest's `stubEnv` doesn't exist in Jest. Use `jest-env-utils.js` compatibility layer or direct `process.env` manipulation.

2. **Mock Timers**: When using Jest's timer mocks, make sure to properly wrap timer-related code in `act()` to prevent React testing warnings:
   ```javascript
   act(() => {
     jest.advanceTimersByTime(1000);
   });
   ```

3. **Asynchronous Testing**: Use `waitFor()` from '@testing-library/react' instead of Vitest's `vi.waitFor()`.

4. **Test Failures**: Many failures after migration relate to component implementation issues or incorrect mocks, not the testing framework itself.

5. **Component Selectors**: Some tests were written with incorrect assumptions about the component structure. Update selectors to match the actual components (e.g., using `getByRole` with proper aria attributes).

---

Last Updated: 2024-05-11 