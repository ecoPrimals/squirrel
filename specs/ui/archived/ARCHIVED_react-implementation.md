# Squirrel UI - React Implementation Specification

## Implementation Status

### Completed Components
- **Core Structure**
  - AppShell layout with header, main content, and footer
  - TabNavigation for multiple dashboard views
  - StatusBar for system status and alerts
  - Theme toggle with light/dark/system mode support

- **Widgets**
  - HealthWidget: System health monitoring
  - MetricsWidget: CPU, memory, and disk usage metrics
  - ChartWidget: Data visualization
  - NetworkWidget: Network interface visualization
  - AlertsWidget: System alerts and notifications
  - ProtocolWidget: Protocol status tracking
  - SystemWidget: System information display

- **Services and Utilities**
  - API service for Tauri backend integration
  - Theme utilities for dark/light mode management
  - Dashboard data fetching hook with mock fallback

### Technical Implementation
- **Frontend Stack**
  - React 18 with TypeScript
  - Tailwind CSS for styling
  - React Query for data fetching
  - Tauri API for desktop integration

- **Data Flow**
  - Custom hooks for data management
  - API services for backend communication
  - Fallback mock data for development

- **Theming**
  - Dark/light mode toggle
  - System preference detection
  - Theme persistence in localStorage
  - Tailwind dark mode classes

## Testing Strategy

### Unit Testing
- Implement Jest + React Testing Library for component testing
- Test all core components:
  ```typescript
  // Example test for ThemeToggle
  describe('ThemeToggle', () => {
    it('renders light, dark and system theme buttons', () => {
      render(<ThemeToggle />);
      expect(screen.getByLabelText('Light mode')).toBeInTheDocument();
      expect(screen.getByLabelText('Dark mode')).toBeInTheDocument();
      expect(screen.getByLabelText('System theme')).toBeInTheDocument();
    });
    
    it('changes theme when buttons are clicked', () => {
      render(<ThemeToggle />);
      fireEvent.click(screen.getByLabelText('Dark mode'));
      // Test that the theme was changed
    });
  });
  ```

- Test utilities and hooks:
  ```typescript
  // Example test for useDashboardData hook
  describe('useDashboardData', () => {
    it('fetches dashboard data', async () => {
      const mockData = { health: { status: 'healthy' } };
      jest.spyOn(api, 'fetchDashboardData').mockResolvedValue(mockData);
      
      const { result, waitFor } = renderHook(() => useDashboardData());
      
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toEqual(mockData);
    });
    
    it('falls back to mock data on error', async () => {
      jest.spyOn(api, 'fetchDashboardData').mockRejectedValue(new Error('API error'));
      
      const { result, waitFor } = renderHook(() => useDashboardData());
      
      await waitFor(() => expect(result.current.data).toBeTruthy());
      // Verify mock data is returned
    });
  });
  ```

### Integration Testing
- Test interactions between components
- Test theme persistence and changes
- Test tab navigation functionality

### E2E Testing
- Implement Playwright for E2E testing
- Test full application flow:
  - Application startup
  - Data loading
  - Tab navigation
  - Theme switching
  - System interactions

## Build Error Resolution

### Current Errors
- TypeScript dependency errors for React and related libraries
- Missing JSX runtime configuration
- Module path resolution issues

### Resolution Plan
1. **Update package.json with missing dependencies**:
   ```bash
   npm install --save-dev @types/react @types/react-dom jest ts-jest @testing-library/react @testing-library/jest-dom @testing-library/user-event
   ```

2. **Fix TypeScript configuration**:
   - Update tsconfig.json to properly resolve React types:
   ```json
   {
     "compilerOptions": {
       "jsx": "react-jsx",
       "types": ["react/jsx-runtime", "jest", "node"]
     }
   }
   ```

3. **Add Testing Configuration**:
   - Create jest.config.js:
   ```js
   module.exports = {
     preset: 'ts-jest',
     testEnvironment: 'jsdom',
     setupFilesAfterEnv: ['<rootDir>/src/setupTests.ts']
   };
   ```
   
   - Create src/setupTests.ts:
   ```typescript
   import '@testing-library/jest-dom';
   ```

4. **Update Scripts in package.json**:
   ```json
   "scripts": {
     "dev": "vite",
     "build": "tsc && vite build",
     "preview": "vite preview",
     "test": "jest",
     "test:watch": "jest --watch"
   }
   ```

## Next Steps

1. **Complete Testing Implementation**:
   - Set up Jest and React Testing Library
   - Write tests for all components
   - Implement E2E tests with Playwright

2. **Fix Build Errors**:
   - Resolve TypeScript configuration issues
   - Update dependencies

3. **Final Integration with Tauri Backend**:
   - Connect API service to Rust functions
   - Test full data flow

4. **Additional UI Enhancements**:
   - Add animations for transitions
   - Implement responsive design optimizations
   - Add keyboard shortcuts 