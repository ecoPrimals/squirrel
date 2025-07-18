---
title: Squirrel React Component Specifications
version: 1.0.0
date: 2024-04-09
status: active
---

# Squirrel React Component Specifications

## Overview

This document outlines the specifications for React components in the Tauri + React implementation of Squirrel's UI system. It defines the component structure, props, state management, and how they map to the equivalent Terminal UI components.

## Component Architecture

The React component architecture follows these principles:

1. **Component-Based Design**: UI built from reusable, composable components
2. **Typed Interface**: All components have TypeScript interfaces for props and state
3. **Presentation/Logic Separation**: Container and presentation component pattern
4. **Consistent Styling**: TailwindCSS for styling with consistent design tokens
5. **Accessibility-First**: All components meet WCAG 2.1 AA standards
6. **Responsive Design**: Components adapt to different screen sizes

## Core Layout Components

### AppShell

The main application container that establishes the layout structure.

```tsx
interface AppShellProps {
  children: React.ReactNode;
  isLoading?: boolean;
  sidebar?: React.ReactNode;
  statusBar?: React.ReactNode;
}

const AppShell: React.FC<AppShellProps> = ({ 
  children, 
  isLoading,
  sidebar,
  statusBar
}) => {
  // Implementation
};
```

### TabNavigation

Handles tab switching and navigation similar to the Terminal UI tabs.

```tsx
interface TabItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  content: React.ReactNode;
}

interface TabNavigationProps {
  tabs: TabItem[];
  activeTab: string;
  onTabChange: (tabId: string) => void;
}

const TabNavigation: React.FC<TabNavigationProps> = ({
  tabs,
  activeTab,
  onTabChange
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `ui.rs` tab rendering logic and tab switching

### StatusBar

Displays application status, connection info, and key metrics.

```tsx
interface StatusBarProps {
  connectionStatus: ConnectionStatus;
  lastUpdated?: Date;
  errors?: string[];
  version?: string;
}

const StatusBar: React.FC<StatusBarProps> = ({
  connectionStatus,
  lastUpdated,
  errors,
  version
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: Footer rendering in `ui.rs`

## Dashboard Widgets

### HealthWidget

Displays system health indicators similar to the Terminal UI HealthWidget.

```tsx
interface HealthCheck {
  name: string;
  status: 'ok' | 'warning' | 'error' | 'unknown';
  message?: string;
}

interface HealthWidgetProps {
  healthChecks: HealthCheck[];
  loading?: boolean;
}

const HealthWidget: React.FC<HealthWidgetProps> = ({
  healthChecks,
  loading
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/health.rs`

### MetricsWidget

Displays system metrics similar to the Terminal UI MetricsWidget.

```tsx
interface Metrics {
  cpu: number;
  memory: number;
  disk: number;
  uptime: number;
  load?: number[];
}

interface MetricsWidgetProps {
  metrics: Metrics;
  loading?: boolean;
}

const MetricsWidget: React.FC<MetricsWidgetProps> = ({
  metrics,
  loading
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/metrics.rs`

### ChartWidget

Displays time-series data in charts similar to the Terminal UI ChartWidget.

```tsx
interface DataPoint {
  timestamp: Date;
  value: number;
}

interface ChartWidgetProps {
  title: string;
  data: DataPoint[];
  maxItems?: number;
  yAxisLabel?: string;
  loading?: boolean;
  color?: string;
}

const ChartWidget: React.FC<ChartWidgetProps> = ({
  title,
  data,
  maxItems,
  yAxisLabel,
  loading,
  color
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/chart.rs`

### NetworkWidget

Displays network status and metrics similar to the Terminal UI NetworkWidget.

```tsx
interface NetworkInterface {
  name: string;
  ipAddress: string;
  rxBytes: number;
  txBytes: number;
  status: 'up' | 'down';
}

interface NetworkWidgetProps {
  interfaces: NetworkInterface[];
  loading?: boolean;
}

const NetworkWidget: React.FC<NetworkWidgetProps> = ({
  interfaces,
  loading
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/network.rs`

### AlertsWidget

Displays system alerts similar to the Terminal UI AlertsWidget.

```tsx
interface Alert {
  id: string;
  severity: 'info' | 'warning' | 'error' | 'critical';
  timestamp: Date;
  message: string;
  source: string;
  acknowledged: boolean;
}

interface AlertsWidgetProps {
  alerts: Alert[];
  onAcknowledge?: (id: string) => void;
  loading?: boolean;
}

const AlertsWidget: React.FC<AlertsWidgetProps> = ({
  alerts,
  onAcknowledge,
  loading
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/alerts.rs`

### ProtocolWidget

Displays protocol status similar to the Terminal UI ProtocolWidget.

```tsx
interface ProtocolStatus {
  status: 'connected' | 'disconnected' | 'connecting';
  latency?: number;
  uptime?: number;
  version?: string;
  errors?: string[];
}

interface ProtocolWidgetProps {
  protocol: ProtocolStatus;
  loading?: boolean;
}

const ProtocolWidget: React.FC<ProtocolWidgetProps> = ({
  protocol,
  loading
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/protocol.rs`

### SystemWidget

Displays system information similar to the Terminal UI SystemWidget.

```tsx
interface Process {
  pid: number;
  name: string;
  cpu: number;
  memory: number;
  status: string;
}

interface SystemInfo {
  hostname: string;
  os: string;
  kernel: string;
  processes: Process[];
}

interface SystemWidgetProps {
  systemInfo: SystemInfo;
  loading?: boolean;
}

const SystemWidget: React.FC<SystemWidgetProps> = ({
  systemInfo,
  loading
}) => {
  // Implementation
};
```

**Maps to Terminal UI**: `widgets/system.rs`

## State Management

The React UI uses Zustand for state management with a structure that parallels the App state in the Terminal UI.

```tsx
interface AppState {
  // Dashboard Data
  metrics: Metrics | null;
  alerts: Alert[];
  networkInterfaces: NetworkInterface[];
  protocol: ProtocolStatus | null;
  systemInfo: SystemInfo | null;
  
  // Derived State
  connectionStatus: ConnectionStatus;
  healthChecks: HealthCheck[];
  cpuHistory: DataPoint[];
  memoryHistory: DataPoint[];
  
  // UI State
  activeTab: string;
  isLoading: boolean;
  lastUpdated: Date | null;
  errors: string[];
  showHelp: boolean;
  
  // Actions
  fetchDashboardData: () => Promise<void>;
  setActiveTab: (tab: string) => void;
  acknowledgeAlert: (id: string) => void;
  toggleHelp: () => void;
}

const useAppStore = create<AppState>((set, get) => ({
  // Initial state and actions implementation
}));
```

**Maps to Terminal UI**: `app.rs::AppState` structure

## Data Fetching

React components use React Query for data fetching, which maps to the DashboardService integration in the Terminal UI.

```tsx
export const useDashboardData = () => {
  return useQuery({
    queryKey: ['dashboardData'],
    queryFn: async () => {
      const data = await invoke<DashboardData>('get_dashboard_data');
      return data;
    },
    refetchInterval: 5000, // Refetch every 5 seconds
  });
};
```

**Maps to Terminal UI**: `app.update()` method that calls `provider.get_dashboard_data()`

## Layout Structure

The overall layout structure mirrors the Terminal UI layout:

```tsx
<AppShell 
  statusBar={<StatusBar />}
  sidebar={<SidePanel />}
>
  <TabNavigation
    tabs={[
      {
        id: 'overview',
        label: 'Overview',
        content: (
          <div className="grid grid-cols-2 gap-4">
            <HealthWidget />
            <MetricsWidget />
            <ChartWidget title="CPU Usage" />
            <ChartWidget title="Memory Usage" />
          </div>
        )
      },
      {
        id: 'network',
        label: 'Network',
        content: <NetworkWidget />
      },
      {
        id: 'alerts',
        label: 'Alerts',
        content: <AlertsWidget />
      },
      {
        id: 'protocol',
        label: 'Protocol',
        content: <ProtocolWidget />
      },
      {
        id: 'system',
        label: 'System',
        content: <SystemWidget />
      }
    ]}
    activeTab={activeTab}
    onTabChange={setActiveTab}
  />
</AppShell>
```

**Maps to Terminal UI**: Overall layout in `ui.rs` with the 2x2 grid for Overview tab and individual tabs for others

## Platform-Specific Components

These components are unique to the Tauri + React implementation:

### TitleBar

Custom window titlebar for the desktop application (Tauri specific).

```tsx
interface TitleBarProps {
  title: string;
  minimizable?: boolean;
  maximizable?: boolean;
  closable?: boolean;
}

const TitleBar: React.FC<TitleBarProps> = ({
  title,
  minimizable,
  maximizable,
  closable
}) => {
  // Implementation
};
```

### SystemTray

System tray integration for the desktop application (Tauri specific).

```tsx
interface TrayMenuItem {
  id: string;
  label: string;
  icon?: string;
  enabled?: boolean;
  onClick: () => void;
}

interface SystemTrayProps {
  items: TrayMenuItem[];
  icon: string;
  tooltip?: string;
}

const SystemTray: React.FC<SystemTrayProps> = ({
  items,
  icon,
  tooltip
}) => {
  // Implementation
};
```

### FileDialog

Native file dialog wrapper (Tauri specific).

```tsx
interface FileDialogProps {
  open: boolean;
  onSelect: (path: string | string[] | null) => void;
  onCancel: () => void;
  multiple?: boolean;
  directory?: boolean;
  filters?: { name: string; extensions: string[] }[];
}

const FileDialog: React.FC<FileDialogProps> = ({
  open,
  onSelect,
  onCancel,
  multiple,
  directory,
  filters
}) => {
  // Implementation
};
```

## Component Testing

Each component should have a comprehensive test suite:

1. **Unit Tests**: Testing component rendering and props
2. **Integration Tests**: Testing component interactions
3. **Accessibility Tests**: Testing ARIA compliance
4. **Visual Tests**: Storybook stories for visual regression testing

Example unit test:

```tsx
import { render, screen } from '@testing-library/react';
import { HealthWidget } from './HealthWidget';

test('renders health checks correctly', () => {
  const healthChecks = [
    { name: 'CPU', status: 'ok', message: 'CPU usage normal' },
    { name: 'Memory', status: 'warning', message: 'Memory usage high' }
  ];
  
  render(<HealthWidget healthChecks={healthChecks} />);
  
  expect(screen.getByText('CPU')).toBeInTheDocument();
  expect(screen.getByText('Memory')).toBeInTheDocument();
  expect(screen.getByText('CPU usage normal')).toBeInTheDocument();
  expect(screen.getByText('Memory usage high')).toBeInTheDocument();
  
  const statusIndicators = screen.getAllByRole('status');
  expect(statusIndicators[0]).toHaveClass('status-ok');
  expect(statusIndicators[1]).toHaveClass('status-warning');
});
```

## References

- [Squirrel Tauri + React Architecture](./tauri-react-architecture.md)
- [Terminal UI Component Specs](./tui-component-specs.md)
- [Web UI Strategy](./web/web-ui-strategy.md)
- [Desktop UI Strategy](./desktop/desktop-ui-strategy.md)
- [React Documentation](https://reactjs.org/docs/components-and-props.html)
- [TypeScript Documentation](https://www.typescriptlang.org/docs/handbook/react.html) 