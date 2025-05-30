# UI Testing Patterns and Best Practices

**Version**: 1.0.0  
**Date**: 2024-08-30  
**Status**: Active

## Overview

This document outlines common testing patterns and best practices used across the Squirrel UI codebase. These patterns represent the accumulated knowledge from the testing implementation, framework migrations, and ongoing improvements to our test suite.

## Component Testing Patterns

### 1. Basic Component Rendering

```tsx
import { render, screen } from '@testing-library/react';
import Component from './Component';

describe('Component', () => {
  it('renders correctly', () => {
    render(<Component />);
    expect(screen.getByText('Expected Text')).toBeInTheDocument();
  });
});
```

### 2. Testing with Props

```tsx
import { render, screen } from '@testing-library/react';
import Component from './Component';

describe('Component', () => {
  it('renders with custom props', () => {
    const props = {
      title: 'Custom Title',
      description: 'Custom Description'
    };
    
    render(<Component {...props} />);
    
    expect(screen.getByText('Custom Title')).toBeInTheDocument();
    expect(screen.getByText('Custom Description')).toBeInTheDocument();
  });
});
```

### 3. Testing User Interactions

```tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Button from './Button';

describe('Button', () => {
  it('calls onClick when clicked', async () => {
    const handleClick = jest.fn();
    
    render(<Button onClick={handleClick}>Click Me</Button>);
    
    await userEvent.click(screen.getByRole('button', { name: 'Click Me' }));
    
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

### 4. Testing Async Components

```tsx
import { render, screen, waitFor } from '@testing-library/react';
import AsyncComponent from './AsyncComponent';

describe('AsyncComponent', () => {
  it('renders async data when loaded', async () => {
    render(<AsyncComponent />);
    
    // Wait for the async operation to complete
    await waitFor(() => {
      expect(screen.getByText('Data Loaded')).toBeInTheDocument();
    });
  });
});
```

### 5. Testing with React Hooks

```tsx
import { renderHook, act } from '@testing-library/react-hooks';
import useCounter from './useCounter';

describe('useCounter', () => {
  it('increments the counter', () => {
    const { result } = renderHook(() => useCounter());
    
    act(() => {
      result.current.increment();
    });
    
    expect(result.current.count).toBe(1);
  });
});
```

### 6. Testing Error States

```tsx
import { render, screen } from '@testing-library/react';
import ErrorComponent from './ErrorComponent';

describe('ErrorComponent', () => {
  it('displays error message when hasError is true', () => {
    render(<ErrorComponent hasError={true} errorMessage="Something went wrong" />);
    
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });
});
```

### 7. Testing Conditional Rendering

```tsx
import { render, screen } from '@testing-library/react';
import ConditionalComponent from './ConditionalComponent';

describe('ConditionalComponent', () => {
  it('renders content when condition is true', () => {
    render(<ConditionalComponent showContent={true} />);
    expect(screen.getByText('Conditional Content')).toBeInTheDocument();
  });
  
  it('does not render content when condition is false', () => {
    render(<ConditionalComponent showContent={false} />);
    expect(screen.queryByText('Conditional Content')).not.toBeInTheDocument();
  });
});
```

## Mocking Patterns

### 1. Mocking ES Modules

```tsx
// Mocking a default export
jest.mock('./module', () => ({
  __esModule: true,
  default: jest.fn().mockImplementation(() => ({
    property: 'value',
    method: jest.fn()
  }))
}));

// Mocking named exports
jest.mock('./module', () => ({
  __esModule: true,
  namedExport: jest.fn().mockReturnValue('mocked value'),
  anotherExport: jest.fn()
}));
```

### 2. Mocking React Components

```tsx
// Mocking a component
jest.mock('./Card', () => ({
  __esModule: true,
  default: ({ className, children }) => (
    <div data-testid="mocked-card" className={className}>{children}</div>
  )
}));

// Testing with the mocked component
it('renders with mocked Card component', () => {
  render(<ComponentWithCard />);
  expect(screen.getByTestId('mocked-card')).toBeInTheDocument();
});
```

### 3. Mocking Hooks

```tsx
// Mocking a custom hook
jest.mock('./useCustomHook', () => ({
  __esModule: true,
  default: jest.fn().mockReturnValue({
    data: 'mock data',
    loading: false,
    error: null
  })
}));

// Mocking a Redux hook
jest.mock('react-redux', () => ({
  ...jest.requireActual('react-redux'),
  useSelector: jest.fn().mockImplementation(selector => 
    selector({ 
      counter: { value: 10 }
    })
  ),
  useDispatch: () => jest.fn()
}));
```

### 4. Mocking Services

```tsx
// Mocking a service class
jest.mock('../services/ApiService', () => {
  return {
    ApiService: jest.fn().mockImplementation(() => ({
      getData: jest.fn().mockResolvedValue({ result: 'success' }),
      postData: jest.fn().mockResolvedValue({ id: '123' })
    }))
  };
});

// Using the mocked service
const apiService = new ApiService();
await apiService.getData(); // Returns { result: 'success' }
```

### 5. Mocking Promises

```tsx
// Mocking resolved promises
const mockFunction = jest.fn().mockResolvedValue({ data: 'mock data' });

// Mocking rejected promises
const mockFailure = jest.fn().mockRejectedValue(new Error('Failed to fetch'));

// Mocking multiple sequential calls
const mockApiCall = jest.fn()
  .mockResolvedValueOnce({ data: 'first call' })
  .mockResolvedValueOnce({ data: 'second call' })
  .mockRejectedValueOnce(new Error('third call fails'));
```

### 6. Mocking Tauri API

```tsx
// Mocking Tauri invoke
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn().mockImplementation((command, args) => {
    switch (command) {
      case 'get_data':
        return Promise.resolve({ result: 'mocked data' });
      case 'save_data':
        return Promise.resolve({ success: true });
      default:
        return Promise.reject(new Error(`Unknown command: ${command}`));
    }
  })
}));
```

## Safe Testing Patterns

### 1. Defensive Component Testing

```tsx
// Component with defensive programming
function SafeComponent({ data }) {
  // Safely handle potentially undefined props
  const safeData = data || {};
  const { items = [], count = 0 } = safeData;
  
  return (
    <div>
      <h2>Items ({count})</h2>
      <ul>
        {Array.isArray(items) && items.map(item => (
          <li key={item.id}>{item.name}</li>
        ))}
      </ul>
    </div>
  );
}

// Testing the defensive component
it('handles missing data gracefully', () => {
  render(<SafeComponent data={undefined} />);
  expect(screen.getByText('Items (0)')).toBeInTheDocument();
});
```

### 2. Safe Store Access

```tsx
// Component with safe store access
function StoreComponent() {
  const { 
    connectionStatus = 'disconnected', 
    data = [] 
  } = useStore() || {};
  
  return (
    <div>
      <p>Status: {connectionStatus}</p>
      <p>Items: {data.length}</p>
    </div>
  );
}

// Testing with mocked store
jest.mock('../store', () => ({
  useStore: jest.fn()
}));

it('handles undefined store gracefully', () => {
  jest.spyOn(storeModule, 'useStore').mockReturnValue(undefined);
  render(<StoreComponent />);
  expect(screen.getByText('Status: disconnected')).toBeInTheDocument();
  expect(screen.getByText('Items: 0')).toBeInTheDocument();
});
```

### 3. Null Checking in Tests

```tsx
import { render, screen } from '@testing-library/react';
import UserList from './UserList';

describe('UserList', () => {
  it('renders empty state when users is null', () => {
    render(<UserList users={null} />);
    expect(screen.getByText('No users found')).toBeInTheDocument();
  });
  
  it('renders empty state when users is undefined', () => {
    render(<UserList users={undefined} />);
    expect(screen.getByText('No users found')).toBeInTheDocument();
  });
  
  it('renders empty state when users is an empty array', () => {
    render(<UserList users={[]} />);
    expect(screen.getByText('No users found')).toBeInTheDocument();
  });
});
```

## Testing Complex Scenarios

### 1. Testing Component Side Effects

```tsx
// Testing useEffect
it('calls API on mount', async () => {
  const mockFetch = jest.fn().mockResolvedValue({ data: [] });
  jest.spyOn(apiModule, 'fetchData').mockImplementation(mockFetch);
  
  render(<DataComponent />);
  
  await waitFor(() => {
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });
});

// Testing cleanup
it('performs cleanup on unmount', () => {
  const mockCleanup = jest.fn();
  jest.spyOn(apiModule, 'cleanup').mockImplementation(mockCleanup);
  
  const { unmount } = render(<CleanupComponent />);
  unmount();
  
  expect(mockCleanup).toHaveBeenCalledTimes(1);
});
```

### 2. Testing Custom Events

```tsx
it('handles custom events', () => {
  const handleCustomEvent = jest.fn();
  
  render(<EventComponent onCustomEvent={handleCustomEvent} />);
  
  // Dispatch a custom event
  const customEvent = new CustomEvent('customEvent', { detail: { id: '123' } });
  window.dispatchEvent(customEvent);
  
  expect(handleCustomEvent).toHaveBeenCalledWith({ id: '123' });
});
```

### 3. Testing Complex Forms

```tsx
it('submits form with correct values', async () => {
  const handleSubmit = jest.fn();
  
  render(<FormComponent onSubmit={handleSubmit} />);
  
  // Fill out the form
  await userEvent.type(screen.getByLabelText('Name'), 'John Doe');
  await userEvent.type(screen.getByLabelText('Email'), 'john@example.com');
  await userEvent.selectOptions(screen.getByLabelText('Country'), ['US']);
  await userEvent.click(screen.getByLabelText('Accept Terms'));
  
  // Submit the form
  await userEvent.click(screen.getByRole('button', { name: 'Submit' }));
  
  // Check that the form was submitted with the correct values
  expect(handleSubmit).toHaveBeenCalledWith({
    name: 'John Doe',
    email: 'john@example.com',
    country: 'US',
    acceptTerms: true
  });
});
```

## Jest-Specific Patterns

### 1. Timer Mocking

```tsx
// Mocking timers
jest.useFakeTimers();

it('updates after 1 second', () => {
  render(<TimerComponent />);
  
  // Fast-forward time
  jest.advanceTimersByTime(1000);
  
  expect(screen.getByText('1 second elapsed')).toBeInTheDocument();
});

// Clean up
afterEach(() => {
  jest.useRealTimers();
});
```

### 2. Snapshot Testing

```tsx
it('matches the snapshot', () => {
  const { container } = render(<Component />);
  expect(container).toMatchSnapshot();
});
```

### 3. Custom Matchers

```tsx
// Define a custom matcher
expect.extend({
  toBeWithinRange(received, floor, ceiling) {
    const pass = received >= floor && received <= ceiling;
    if (pass) {
      return {
        message: () => `expected ${received} not to be within range ${floor} - ${ceiling}`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be within range ${floor} - ${ceiling}`,
        pass: false,
      };
    }
  },
});

// Use the custom matcher
it('has a value within range', () => {
  const result = calculateValue();
  expect(result).toBeWithinRange(10, 20);
});
```

## Conclusion

These patterns and best practices form the foundation of our testing approach for the Squirrel UI. By following these patterns consistently, we ensure that our tests are reliable, maintainable, and effective at catching issues before they reach production.

New patterns can be added to this document as they emerge through the development process. The goal is to maintain a living document that captures our evolving understanding of testing best practices.

---

**Last Updated**: 2024-08-30 