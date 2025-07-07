---
title: Multi-Mode Component Development Guide
version: 1.0.0
date: 2024-10-20
status: active
---

# Multi-Mode Component Development Guide

## Overview

This guide provides practical instructions for developing UI components that work seamlessly across all three modes of the Squirrel UI framework:

1. **Desktop GUI Mode**: Rich graphical interface for local monitoring
2. **Web Mode**: Browser-based interface for remote access
3. **TUI (Terminal) Mode**: Terminal-based interface for headless systems

By following these guidelines, you'll create components that adapt to their environment while maintaining consistent functionality and data handling.

## Core Principles

When developing multi-mode components, follow these key principles:

1. **Single Source of Truth**: Share data models and business logic across all modes
2. **Mode-Specific Presentation**: Adapt the presentation layer based on the detected mode
3. **Responsive Design**: Components should adapt to available space in all environments
4. **Fallback Patterns**: Implement fallbacks for features not available in all modes
5. **Progressive Enhancement**: Add rich features in GUI modes while maintaining core functionality in TUI

## Component Structure

### Folder Organization

```
src/components/[feature-name]/
├── index.ts                  # Main export (mode-adaptive)
├── [FeatureName].tsx         # Shared component logic 
├── [FeatureName]GUI.tsx      # Desktop/Web GUI implementation
├── [FeatureName]TUI.tsx      # Terminal UI implementation
├── [FeatureName]Store.ts     # Shared state management
├── types.ts                  # Shared type definitions
└── utils.ts                  # Shared utility functions
```

### Shared Logic Pattern

Start with a base class or hook that implements the shared business logic:

```typescript
// src/components/metrics/useMetrics.ts
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { SystemMetrics } from './types';

export function useMetrics() {
  const [data, setData] = useState<SystemMetrics | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  
  const fetchMetrics = async () => {
    try {
      setIsLoading(true);
      const response = await invoke('get_system_metrics');
      setData(response as SystemMetrics);
      setError(null);
    } catch (err) {
      setError(err as Error);
    } finally {
      setIsLoading(false);
    }
  };
  
  useEffect(() => {
    fetchMetrics();
    const interval = setInterval(fetchMetrics, 5000);
    return () => clearInterval(interval);
  }, []);
  
  return { data, isLoading, error, refresh: fetchMetrics };
}
```

### Mode-Specific Implementations

#### GUI Component

```tsx
// src/components/metrics/MetricsGUI.tsx
import React from 'react';
import { useMetrics } from './useMetrics';
import { CircularProgress, Card, CardContent, Typography } from '@mui/material';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip } from 'recharts';

export const MetricsGUI: React.FC = () => {
  const { data, isLoading, error } = useMetrics();
  
  if (isLoading) return <CircularProgress />;
  if (error) return <Typography color="error">Error: {error.message}</Typography>;
  if (!data) return <Typography>No data available</Typography>;
  
  return (
    <Card>
      <CardContent>
        <Typography variant="h5" component="div">
          System Metrics
        </Typography>
        
        <LineChart width={500} height={300} data={data.history}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="timestamp" />
          <YAxis />
          <Tooltip />
          <Line type="monotone" dataKey="cpu" stroke="#8884d8" />
          <Line type="monotone" dataKey="memory" stroke="#82ca9d" />
        </LineChart>
        
        <Typography>
          CPU: {data.cpu.toFixed(1)}% | Memory: {data.memory.toFixed(1)}%
        </Typography>
        
        <Typography>
          Status: {data.status === 'ok' ? '✅ Healthy' : '⚠️ Warning'}
        </Typography>
      </CardContent>
    </Card>
  );
};
```

#### TUI Component

```tsx
// src/components/metrics/MetricsTUI.tsx
import React from 'react';
import { useMetrics } from './useMetrics';
import { AsciiBox, AsciiSpinner, AsciiChart } from '../../tui/components';

export const MetricsTUI: React.FC = () => {
  const { data, isLoading, error } = useMetrics();
  
  if (isLoading) return <AsciiSpinner text="Loading metrics..." />;
  if (error) return <AsciiBox title="Error">Error: {error.message}</AsciiBox>;
  if (!data) return <AsciiBox title="Metrics">No data available</AsciiBox>;
  
  // Create ASCII representations
  const cpuBar = '#'.repeat(Math.round(data.cpu / 2));
  const memoryBar = '#'.repeat(Math.round(data.memory / 2));
  
  return (
    <AsciiBox title="System Metrics">
      {`CPU    : [${cpuBar.padEnd(50)}] ${data.cpu.toFixed(1)}%\n`}
      {`Memory : [${memoryBar.padEnd(50)}] ${data.memory.toFixed(1)}%\n`}
      {`Status : ${data.status === 'ok' ? '✓ Healthy' : '! Warning'}\n`}
      {`Updated: ${new Date().toLocaleTimeString()}`}
    </AsciiBox>
  );
};
```

### Mode-Adaptive Export

```typescript
// src/components/metrics/index.ts
import { isTuiMode } from '../../utils/modeDetection';
import { MetricsGUI } from './MetricsGUI';
import { MetricsTUI } from './MetricsTUI';

// Export the appropriate component based on the current mode
export const MetricsWidget = isTuiMode ? MetricsTUI : MetricsGUI;
```

## Handling Mode-Specific Features

Some features may only be available in certain modes. Use feature detection and fallbacks to handle these cases:

```tsx
// src/components/plugins/PluginManager.tsx
import React from 'react';
import { features } from '../../utils/features';
import { usePlugins } from './usePlugins';

export const PluginManager: React.FC = () => {
  const { plugins, installPlugin, removePlugin } = usePlugins();
  
  return (
    <div className="plugin-manager">
      <h2>Plugin Manager</h2>
      
      <div className="plugin-list">
        {plugins.map(plugin => (
          <div key={plugin.id} className="plugin-item">
            <span>{plugin.name}</span>
            
            {/* Show remove button in all modes */}
            <button onClick={() => removePlugin(plugin.id)}>Remove</button>
            
            {/* Advanced features only in GUI modes */}
            {features.fileSystemAccess && (
              <button onClick={() => plugin.openConfig()}>Edit Config</button>
            )}
            
            {/* Feature with fallback */}
            <button onClick={() => 
              features.nativeNotifications 
                ? plugin.testWithNotification() 
                : plugin.testSimple()
            }>
              Test
            </button>
          </div>
        ))}
      </div>
      
      {/* Install only available if file system access is supported */}
      {features.fileSystemAccess ? (
        <button onClick={() => installPlugin()}>Install New Plugin</button>
      ) : (
        <p>Plugin installation requires desktop mode</p>
      )}
    </div>
  );
};
```

## Best Practices

### 1. Shared State Management

Use stores or hooks that work consistently across all modes:

```typescript
// src/stores/dashboardStore.ts
import { create } from 'zustand';

interface DashboardState {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  refreshInterval: number;
  setRefreshInterval: (interval: number) => void;
  lastUpdated: Date | null;
  updateTimestamp: () => void;
}

export const useDashboardStore = create<DashboardState>((set) => ({
  activeTab: 'overview',
  setActiveTab: (tab) => set({ activeTab: tab }),
  refreshInterval: 5000,
  setRefreshInterval: (interval) => set({ refreshInterval: interval }),
  lastUpdated: null,
  updateTimestamp: () => set({ lastUpdated: new Date() }),
}));
```

### 2. Responsive Design Across Modes

Implement responsive layouts that adapt to available space:

```tsx
// GUI implementation with responsive design
const ResponsiveWidget: React.FC = () => {
  return (
    <div className="responsive-widget">
      <div className="widget-grid" 
           style={{ 
             display: 'grid', 
             gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
             gap: '1rem'
           }}>
        <MetricsWidget />
        <StatusWidget />
        <AlertsWidget />
      </div>
    </div>
  );
};

// TUI implementation with dynamic sizing
const ResponsiveWidgetTUI: React.FC = () => {
  const { width } = useTerminalSize();
  
  // Adjust layout based on terminal width
  return width > 100 ? (
    <AsciiGrid columns={3}>
      <MetricsWidget />
      <StatusWidget />
      <AlertsWidget />
    </AsciiGrid>
  ) : (
    <AsciiStack>
      <MetricsWidget />
      <StatusWidget />
      <AlertsWidget />
    </AsciiStack>
  );
};
```

### 3. Feature Detection and Fallbacks

Always detect features and provide fallbacks for unsupported capabilities:

```typescript
// Notification with fallback
const sendNotification = async (title: string, message: string) => {
  if (features.nativeNotifications) {
    await invoke('send_native_notification', { title, message });
  } else if (features.richDataVisualization) {
    showToast(title, message); // In-app toast notification
  } else {
    console.log(`${title}: ${message}`); // Terminal fallback
  }
};
```

### 4. Use Mode-Appropriate Input Methods

Handle input based on available input methods:

```tsx
// src/components/commands/CommandInput.tsx
import React, { useState } from 'react';
import { features } from '../../utils/features';
import { isTuiMode } from '../../utils/modeDetection';

export const CommandInput: React.FC = () => {
  const [command, setCommand] = useState('');
  
  const executeCommand = async () => {
    // Execute command logic
  };
  
  // TUI Mode: Focus on keyboard input
  if (isTuiMode) {
    return (
      <AsciiBox title="Command Input">
        {`> ${command}_\n`}
        {`Press Enter to execute, Esc to cancel`}
      </AsciiBox>
    );
  }
  
  // GUI Mode: Rich input with autocomplete
  return (
    <div className="command-input">
      <input 
        type="text" 
        value={command}
        onChange={(e) => setCommand(e.target.value)}
        placeholder="Enter command..."
      />
      
      {features.richDataVisualization && (
        <div className="suggestions">
          {/* Command suggestions */}
        </div>
      )}
      
      <button onClick={executeCommand}>Execute</button>
    </div>
  );
};
```

### 5. Consistent Data Interactions

Ensure data operations work identically across all modes:

```typescript
// src/components/alerts/useAlerts.ts
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Alert } from './types';

export function useAlerts() {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  
  const fetchAlerts = async () => {
    const response = await invoke('get_alerts');
    setAlerts(response as Alert[]);
  };
  
  const acknowledgeAlert = async (id: string) => {
    await invoke('acknowledge_alert', { id });
    // Update local state
    setAlerts(alerts.map(alert => 
      alert.id === id ? { ...alert, acknowledged: true } : alert
    ));
  };
  
  useEffect(() => {
    fetchAlerts();
    const interval = setInterval(fetchAlerts, 10000);
    return () => clearInterval(interval);
  }, []);
  
  return { alerts, acknowledgeAlert, refresh: fetchAlerts };
}
```

## Advanced Component Examples

### 1. Real-Time Dashboard

```tsx
// src/components/dashboard/Dashboard.tsx
import React from 'react';
import { isTuiMode } from '../../utils/modeDetection';
import { useDashboardStore } from '../../stores/dashboardStore';
import { MetricsWidget } from '../metrics';
import { AlertsWidget } from '../alerts';
import { HealthWidget } from '../health';
import { NetworkWidget } from '../network';

export const Dashboard: React.FC = () => {
  const { activeTab, setActiveTab } = useDashboardStore();
  
  // TUI-specific rendering
  if (isTuiMode) {
    return (
      <AsciiLayout>
        <AsciiTabs
          tabs={['Overview', 'Metrics', 'Network', 'Alerts']}
          activeTab={activeTab}
          onTabChange={setActiveTab}
        />
        
        {activeTab === 'Overview' && (
          <AsciiGrid columns={2}>
            <MetricsWidget />
            <AlertsWidget />
            <HealthWidget />
            <NetworkWidget />
          </AsciiGrid>
        )}
        
        {activeTab === 'Metrics' && <MetricsWidget detailed={true} />}
        {activeTab === 'Network' && <NetworkWidget detailed={true} />}
        {activeTab === 'Alerts' && <AlertsWidget detailed={true} />}
        
        <AsciiFooter>
          Press 1-4 to switch tabs, R to refresh, Q to quit
        </AsciiFooter>
      </AsciiLayout>
    );
  }
  
  // GUI rendering (desktop & web)
  return (
    <div className="dashboard">
      <div className="tabs">
        <button 
          className={activeTab === 'Overview' ? 'active' : ''} 
          onClick={() => setActiveTab('Overview')}>
          Overview
        </button>
        <button 
          className={activeTab === 'Metrics' ? 'active' : ''} 
          onClick={() => setActiveTab('Metrics')}>
          Metrics
        </button>
        <button 
          className={activeTab === 'Network' ? 'active' : ''} 
          onClick={() => setActiveTab('Network')}>
          Network
        </button>
        <button 
          className={activeTab === 'Alerts' ? 'active' : ''} 
          onClick={() => setActiveTab('Alerts')}>
          Alerts
        </button>
      </div>
      
      <div className="content">
        {activeTab === 'Overview' && (
          <div className="grid-layout">
            <MetricsWidget />
            <AlertsWidget />
            <HealthWidget />
            <NetworkWidget />
          </div>
        )}
        
        {activeTab === 'Metrics' && <MetricsWidget detailed={true} />}
        {activeTab === 'Network' && <NetworkWidget detailed={true} />}
        {activeTab === 'Alerts' && <AlertsWidget detailed={true} />}
      </div>
    </div>
  );
};
```

### 2. Plugin Management System

```tsx
// src/components/plugins/PluginManagement.tsx
import React, { useState } from 'react';
import { isTuiMode } from '../../utils/modeDetection';
import { usePlugins } from './usePlugins';
import { features } from '../../utils/features';
import { PluginDetails } from './types';

export const PluginManagement: React.FC = () => {
  const { plugins, installPlugin, removePlugin, updatePlugin } = usePlugins();
  const [selectedPlugin, setSelectedPlugin] = useState<string | null>(null);
  
  // Get details of currently selected plugin
  const selectedDetails = selectedPlugin 
    ? plugins.find(p => p.id === selectedPlugin) || null
    : null;
  
  if (isTuiMode) {
    // TUI implementation with navigation instructions
    return (
      <AsciiBox title="Plugin Management">
        <AsciiList
          items={plugins.map(p => ({ 
            id: p.id, 
            label: `${p.name} v${p.version} [${p.status}]` 
          }))}
          selectedItem={selectedPlugin}
          onSelect={setSelectedPlugin}
        />
        
        {selectedDetails && (
          <AsciiBox title={selectedDetails.name}>
            {`ID: ${selectedDetails.id}\n`}
            {`Version: ${selectedDetails.version}\n`}
            {`Status: ${selectedDetails.status}\n`}
            {`Author: ${selectedDetails.author}\n`}
            {`Description: ${selectedDetails.description}\n`}
          </AsciiBox>
        )}
        
        <AsciiFooter>
          {`Arrow keys: Navigate | Enter: Select | R: Refresh\n`}
          {`I: Install | U: Update | D: Delete | Q: Back`}
        </AsciiFooter>
      </AsciiBox>
    );
  }
  
  // GUI implementation with rich interactions
  return (
    <div className="plugin-management">
      <h2>Plugin Management</h2>
      
      <div className="plugin-grid">
        {plugins.map(plugin => (
          <div 
            key={plugin.id} 
            className={`plugin-card ${selectedPlugin === plugin.id ? 'selected' : ''}`}
            onClick={() => setSelectedPlugin(plugin.id)}
          >
            <div className="plugin-header">
              <h3>{plugin.name}</h3>
              <span className={`status ${plugin.status}`}>{plugin.status}</span>
            </div>
            
            <p className="version">v{plugin.version}</p>
            <p className="description">{plugin.description}</p>
            
            <div className="plugin-actions">
              <button onClick={() => updatePlugin(plugin.id)}>Update</button>
              <button onClick={() => removePlugin(plugin.id)}>Remove</button>
              
              {features.fileSystemAccess && (
                <button onClick={() => showPluginConfig(plugin.id)}>Configure</button>
              )}
            </div>
          </div>
        ))}
      </div>
      
      {features.fileSystemAccess && (
        <button className="install-button" onClick={() => installPlugin()}>
          Install New Plugin
        </button>
      )}
    </div>
  );
};
```

## Testing Multi-Mode Components

### 1. Unit Testing Each Mode

```typescript
// src/components/metrics/__tests__/Metrics.test.tsx
import React from 'react';
import { render, screen } from '@testing-library/react';
import { MetricsGUI } from '../MetricsGUI';
import { MetricsTUI } from '../MetricsTUI';
import { mockInvoke } from '../../../test-utils/tauri-mocks';

// Mock data
const mockMetricsData = {
  cpu: 45.5,
  memory: 62.3,
  disk: 78.1,
  status: 'ok',
  uptime: 362145
};

// Test GUI component
describe('MetricsGUI Component', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(mockMetricsData);
  });
  
  test('renders loading state initially', () => {
    render(<MetricsGUI />);
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });
  
  test('displays metrics data when loaded', async () => {
    render(<MetricsGUI />);
    
    // Wait for data to load
    expect(await screen.findByText(/CPU: 45.5%/)).toBeInTheDocument();
    expect(screen.getByText(/Memory: 62.3%/)).toBeInTheDocument();
    expect(screen.getByText(/✅ Healthy/)).toBeInTheDocument();
  });
});

// Test TUI component
describe('MetricsTUI Component', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(mockMetricsData);
  });
  
  test('renders loading state initially', () => {
    render(<MetricsTUI />);
    expect(screen.getByText(/Loading metrics/)).toBeInTheDocument();
  });
  
  test('displays ASCII metrics when loaded', async () => {
    render(<MetricsTUI />);
    
    // Wait for data to load
    const output = await screen.findByText(/CPU.*45.5%/);
    expect(output).toBeInTheDocument();
    expect(output.textContent).toContain('#'.repeat(22)); // 45.5 / 2 ≈ 22
  });
});
```

### 2. Mode-Switching Tests

```typescript
// src/components/metrics/__tests__/MetricsIntegration.test.tsx
import React from 'react';
import { render, screen } from '@testing-library/react';
import { MetricsWidget } from '../index';
import * as modeDetection from '../../../utils/modeDetection';
import { mockInvoke } from '../../../test-utils/tauri-mocks';

// Mock the mode detection module
jest.mock('../../../utils/modeDetection', () => ({
  isTuiMode: false, // Default to GUI mode
  isDesktopMode: true,
  isWebMode: false,
  currentMode: 'desktop'
}));

const mockMetricsData = {
  cpu: 45.5,
  memory: 62.3,
  disk: 78.1,
  status: 'ok',
  uptime: 362145
};

describe('MetricsWidget Integration', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(mockMetricsData);
  });
  
  test('renders GUI component when in desktop mode', async () => {
    // Ensure GUI mode
    jest.spyOn(modeDetection, 'isTuiMode', 'get').mockReturnValue(false);
    
    render(<MetricsWidget />);
    expect(await screen.findByText(/System Metrics/)).toBeInTheDocument();
    // Check for GUI-specific elements
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });
  
  test('renders TUI component when in terminal mode', async () => {
    // Switch to TUI mode
    jest.spyOn(modeDetection, 'isTuiMode', 'get').mockReturnValue(true);
    
    render(<MetricsWidget />);
    
    // Check for TUI-specific elements (ASCII art)
    const output = await screen.findByText(/CPU.*45.5%/);
    expect(output.textContent).toContain('#'.repeat(22));
  });
});
```

## Common Pitfalls and Solutions

### 1. Inconsistent Data Handling

**Problem**: Different implementations have different data handling logic.

**Solution**: Extract all data handling into shared hooks or services:

```typescript
// BAD: Different implementation in each component
// In GUI component
useEffect(() => {
  const fetchData = async () => {
    try {
      const data = await invoke('get_metrics');
      setMetrics(data);
    } catch (err) {
      console.error(err);
    }
  };
  fetchData();
}, []);

// In TUI component
useEffect(() => {
  const fetchData = async () => {
    try {
      const data = await invoke('get_metrics');
      setMetrics(data);
    } catch (err) {
      setError(err);
    }
  };
  fetchData();
}, []);

// GOOD: Shared logic in hook
function useMetrics() {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  
  const fetchData = async () => {
    try {
      setIsLoading(true);
      const response = await invoke('get_metrics');
      setData(response);
      setError(null);
    } catch (err) {
      setError(err);
    } finally {
      setIsLoading(false);
    }
  };
  
  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, []);
  
  return { data, error, isLoading, refresh: fetchData };
}

// Then use this hook in both components
const { data, error, isLoading } = useMetrics();
```

### 2. Duplicate Event Handling

**Problem**: Event handling logic duplicated across mode implementations.

**Solution**: Create shared handlers in a common module:

```typescript
// src/components/alerts/alertHandlers.ts
import { invoke } from '@tauri-apps/api/tauri';
import { Alert } from './types';

export const acknowledgeAlert = async (id: string) => {
  return await invoke('acknowledge_alert', { id });
};

export const dismissAlert = async (id: string) => {
  return await invoke('dismiss_alert', { id });
};

export const snoozeAlert = async (id: string, minutes: number) => {
  return await invoke('snooze_alert', { id, minutes });
};

// Then in components:
import { acknowledgeAlert, dismissAlert, snoozeAlert } from './alertHandlers';

// GUI component
<button onClick={() => acknowledgeAlert(alert.id)}>Acknowledge</button>

// TUI component
if (key === 'a') acknowledgeAlert(selectedAlert);
```

### 3. Incompatible DOM Elements in TUI Mode

**Problem**: Using DOM elements that don't translate well to terminal interfaces.

**Solution**: Create abstract components with mode-specific implementations:

```typescript
// src/components/ui/Button.tsx
import React from 'react';
import { isTuiMode } from '../../utils/modeDetection';

interface ButtonProps {
  onClick: () => void;
  children: React.ReactNode;
  variant?: 'primary' | 'secondary' | 'danger';
}

// GUI Button
const GUIButton: React.FC<ButtonProps> = ({ 
  onClick, 
  children, 
  variant = 'primary' 
}) => (
  <button 
    className={`btn btn-${variant}`}
    onClick={onClick}>
    {children}
  </button>
);

// TUI Button (visual representation only, actual interaction via keyboard)
const TUIButton: React.FC<ButtonProps> = ({ 
  children, 
  variant = 'primary' 
}) => {
  const style = variant === 'danger' ? 'red' 
    : variant === 'primary' ? 'blue' 
    : 'gray';
    
  return (
    <span className={`tui-button tui-button-${style}`}>
      [{children}]
    </span>
  );
};

// Export the appropriate component
export const Button = isTuiMode ? TUIButton : GUIButton;
```

## Conclusion

By following these guidelines and patterns, you can create UI components that work seamlessly across all three modes of the Squirrel UI framework. This approach allows for maximum code reuse while still providing optimized experiences for each environment.

Remember the key principles:
- Share business logic and data handling
- Adapt the presentation layer to each mode
- Provide appropriate fallbacks
- Test each mode independently and with mode switching

This multi-mode approach ensures that users have access to the same core functionality whether they're using the rich desktop interface, accessing remotely via the web, or working in a terminal environment. 