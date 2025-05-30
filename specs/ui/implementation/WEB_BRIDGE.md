# Web Bridge Implementation Pattern

**Version**: 1.0.0
**Date**: 2024-05-10
**Status**: Complete

## Overview

This document describes the bridge pattern implementation used to integrate the functionality from the standalone web UI into the unified Tauri + React application. The pattern enables the Tauri application to leverage the existing web implementation while providing a unified user experience.

## Architecture

The bridge pattern consists of several key components:

1. **Tauri Commands Layer**: Rust functions exposed to JavaScript via Tauri's command system
2. **API Client**: TypeScript wrapper for Tauri commands
3. **React Integration Components**: React components that consume the API client
4. **State Management Bridge**: Integration with Zustand stores
5. **Fallback Mechanism**: Support for web-only mode when Tauri APIs are unavailable

```
┌─────────────────────────────────────────────────────────┐
│                  React Components                       │
│                                                         │
│  ┌─────────────────────┐      ┌─────────────────────┐  │
│  │ Dashboard Components│      │Web Integration Panel│  │
│  └──────────┬──────────┘      └──────────┬──────────┘  │
│             │                            │             │
└─────────────│────────────────────────────│─────────────┘
              │                            │
              ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐
│  Dashboard Store    │      │    Web Bridge       │
└──────────┬──────────┘      └──────────┬──────────┘
           │                            │
           ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐
│   Tauri Commands    │◄────►│    Web API Client   │
└──────────┬──────────┘      └──────────┬──────────┘
           │                            │
           ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐
│ DashboardService    │      │    Web Server       │
└─────────────────────┘      └─────────────────────┘
```

## Bridge Components

### 1. Tauri Commands Layer

The Tauri commands layer provides the interface between the Rust backend and JavaScript frontend:

```rust
// In src-tauri/src/web/handlers.rs
#[tauri::command]
pub async fn list_web_commands(
    web_bridge: State<'_, WebBridgeState>
) -> Result<Vec<squirrel_web::AvailableCommand>, String> {
    let bridge = web_bridge.0.lock().await;
    bridge.list_commands()
        .await
        .map_err(|e| format!("Failed to list web commands: {}", e))
}

#[tauri::command]
pub async fn execute_web_command(
    web_bridge: State<'_, WebBridgeState>,
    name: String,
    args: Vec<String>
) -> Result<squirrel_web::CommandStatusResponse, String> {
    let bridge = web_bridge.0.lock().await;
    bridge.execute_command(&name, args)
        .await
        .map_err(|e| format!("Failed to execute web command: {}", e))
}

#[tauri::command]
pub async fn web_login(
    web_bridge: State<'_, WebBridgeState>,
    username: String,
    password: String
) -> Result<AuthTokens, String> {
    let bridge = web_bridge.0.lock().await;
    bridge.login(LoginCredentials { username, password })
        .await
        .map_err(|e| format!("Failed to login: {}", e))
}

// WebSocket commands
#[tauri::command]
pub async fn web_create_subscription(
    web_bridge: State<'_, WebBridgeState>,
    window: Window,
    channel: String,
    event_type: String
) -> Result<String, String> {
    let mut bridge = web_bridge.0.lock().await;
    
    // Create a unique event name for this subscription
    let event_name = format!("web-event-{}-{}", channel, event_type);
    let event_name_clone = event_name.clone();
    
    // Register a subscription with the web bridge
    let subscription_id = bridge.create_subscription(&channel, &event_type, Box::new(move |event_data| {
        if let Some(window) = window.get_window("main") {
            if let Err(e) = window.emit(&event_name_clone, event_data) {
                eprintln!("Failed to emit web event: {}", e);
            }
        }
    }))
    .await
    .map_err(|e| format!("Failed to create subscription: {}", e))?;
    
    Ok(subscription_id)
}
```

### 2. API Client

The API client provides a TypeScript wrapper for the Tauri commands:

```typescript
// src/services/WebApiClient.ts
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { AuthTokens, CommandResult, PluginInfo } from '../types';

export class WebApiClient {
    // Authentication
    async login(username: string, password: string): Promise<AuthTokens> {
        return await invoke('web_login', { username, password });
    }
    
    // Token validation and refresh
    async validateToken(token: string): Promise<boolean> {
        return await invoke('web_validate_token', { token });
    }
    
    async refreshToken(refreshToken: string): Promise<AuthTokens> {
        return await invoke('web_refresh_token', { refreshToken });
    }
    
    // Command execution
    async listCommands(): Promise<string[]> {
        return await invoke('list_web_commands');
    }
    
    async executeCommand(command: string, args: string[] = []): Promise<CommandResult> {
        return await invoke('execute_web_command', { name: command, args });
    }
    
    async getCommandStatus(commandId: string): Promise<CommandResult> {
        return await invoke('get_web_command_status', { commandId });
    }
    
    // Plugin management
    async listPlugins(): Promise<PluginInfo[]> {
        return await invoke('list_web_plugins');
    }
    
    async loadPlugin(path: string): Promise<PluginInfo> {
        return await invoke('load_web_plugin', { path });
    }
    
    // WebSocket interaction
    async createSubscription(channel: string, eventType: string): Promise<{
        subscriptionId: string,
        unsubscribe: () => Promise<void>
    }> {
        const eventName = `web-event-${channel}-${eventType}`;
        const subscriptionId = await invoke('web_create_subscription', { 
            channel, 
            eventType 
        });
        
        // Listen for events on this subscription
        const unlisten = await listen(eventName, (event) => {
            // Handle the event
            console.log('Received event:', event);
            if (this.eventHandlers[eventName]) {
                this.eventHandlers[eventName](event.payload);
            }
        });
        
        // Return subscription ID and unsubscribe function
        return {
            subscriptionId,
            unsubscribe: async () => {
                unlisten();
                await invoke('web_close_subscription', { subscriptionId });
            }
        };
    }
    
    // Event handling
    private eventHandlers: Record<string, (data: any) => void> = {};
    
    onEvent(channel: string, eventType: string, handler: (data: any) => void): void {
        const eventName = `web-event-${channel}-${eventType}`;
        this.eventHandlers[eventName] = handler;
    }
    
    async sendEvent(channel: string, eventType: string, data: any): Promise<void> {
        await invoke('web_send_event', { channel, eventType, data });
    }
}
```

### 3. React Integration Components

React components that consume the API client:

```tsx
// src/components/WebIntegrationPanel.tsx
import React, { useEffect, useState } from 'react';
import { WebApiClient } from '../services/WebApiClient';
import { loadTauriApis } from '../utils/tauriUtils';
import { CommandResult, PluginInfo } from '../types';

const WebIntegrationPanel: React.FC = () => {
    // State management
    const [apiClient, setApiClient] = useState<WebApiClient | null>(null);
    const [tauriApiLoaded, setTauriApiLoaded] = useState(false);
    const [commands, setCommands] = useState<string[]>([]);
    const [selectedCommand, setSelectedCommand] = useState('');
    const [commandArgs, setCommandArgs] = useState('');
    const [commandResult, setCommandResult] = useState<CommandResult | null>(null);
    const [plugins, setPlugins] = useState<PluginInfo[]>([]);
    const [eventData, setEventData] = useState<any[]>([]);
    const [activeTab, setActiveTab] = useState('commands');
    
    // Load Tauri APIs and initialize client
    useEffect(() => {
        const initClient = async () => {
            try {
                // Try to load Tauri APIs
                const result = await loadTauriApis(['@tauri-apps/api/tauri', '@tauri-apps/api/event']);
                setTauriApiLoaded(result.success);
                
                // Create API client
                const client = new WebApiClient();
                setApiClient(client);
                
                // Load initial data
                if (result.success) {
                    const commands = await client.listCommands();
                    setCommands(commands);
                    
                    const plugins = await client.listPlugins();
                    setPlugins(plugins);
                    
                    // Subscribe to system events
                    const { unsubscribe } = await client.createSubscription('system', 'status');
                    client.onEvent('system', 'status', (data) => {
                        setEventData(prev => [...prev, data]);
                    });
                    
                    // Clean up on unmount
                    return () => {
                        unsubscribe();
                    };
                }
            } catch (err) {
                console.error('Failed to initialize WebApiClient:', err);
            }
        };
        
        initClient();
    }, []);
    
    // Execute command
    const executeCommand = async () => {
        if (!apiClient) return;
        
        try {
            const args = commandArgs.split(',').map(arg => arg.trim());
            const result = await apiClient.executeCommand(selectedCommand, args);
            setCommandResult(result);
        } catch (err) {
            console.error('Command execution failed:', err);
            setCommandResult({
                id: 'error',
                status: 'error',
                message: `Error: ${err}`,
                result: null
            });
        }
    };
    
    // Render different tabs based on activeTab state
    const renderTabContent = () => {
        switch (activeTab) {
            case 'commands':
                return (
                    <div className="command-panel">
                        <h3>Available Commands</h3>
                        <select 
                            value={selectedCommand} 
                            onChange={(e) => setSelectedCommand(e.target.value)}
                        >
                            <option value="">Select a command</option>
                            {commands.map(cmd => (
                                <option key={cmd} value={cmd}>{cmd}</option>
                            ))}
                        </select>
                        
                        <h3>Command Arguments</h3>
                        <input 
                            type="text" 
                            value={commandArgs} 
                            onChange={(e) => setCommandArgs(e.target.value)}
                            placeholder="Comma-separated arguments"
                        />
                        
                        <button 
                            onClick={executeCommand}
                            disabled={!selectedCommand}
                        >
                            Execute Command
                        </button>
                        
                        {commandResult && (
                            <div className="command-result">
                                <h3>Result</h3>
                                <pre>{JSON.stringify(commandResult, null, 2)}</pre>
                            </div>
                        )}
                    </div>
                );
                
            case 'plugins':
                return (
                    <div className="plugins-panel">
                        <h3>Installed Plugins</h3>
                        <table>
                            <thead>
                                <tr>
                                    <th>Name</th>
                                    <th>Version</th>
                                    <th>Status</th>
                                </tr>
                            </thead>
                            <tbody>
                                {plugins.map(plugin => (
                                    <tr key={plugin.id}>
                                        <td>{plugin.name}</td>
                                        <td>{plugin.version}</td>
                                        <td>{plugin.status}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                );
                
            case 'events':
                return (
                    <div className="events-panel">
                        <h3>WebSocket Events</h3>
                        <div className="event-log">
                            {eventData.map((event, idx) => (
                                <div key={idx} className="event-item">
                                    <pre>{JSON.stringify(event, null, 2)}</pre>
                                </div>
                            ))}
                        </div>
                    </div>
                );
                
            default:
                return null;
        }
    };
    
    // If Tauri APIs not loaded, show warning
    if (!tauriApiLoaded) {
        return (
            <div className="web-bridge-error">
                <h2>Tauri APIs Not Available</h2>
                <p>This feature requires the Tauri desktop application.</p>
            </div>
        );
    }
    
    return (
        <div className="web-integration-panel">
            <div className="tab-navigation">
                <button 
                    className={activeTab === 'commands' ? 'active' : ''}
                    onClick={() => setActiveTab('commands')}
                >
                    Commands
                </button>
                <button 
                    className={activeTab === 'plugins' ? 'active' : ''}
                    onClick={() => setActiveTab('plugins')}
                >
                    Plugins
                </button>
                <button 
                    className={activeTab === 'events' ? 'active' : ''}
                    onClick={() => setActiveTab('events')}
                >
                    Events
                </button>
            </div>
            
            <div className="tab-content">
                {renderTabContent()}
            </div>
        </div>
    );
};

export default WebIntegrationPanel;
```

### 4. State Management Bridge

Integration with Zustand stores:

```typescript
// src/stores/dashboardStore.ts
import { create } from 'zustand';
import { loadTauriApis } from '../utils/tauriUtils';
import { DashboardData } from '../types';

interface DashboardState {
    dashboardData: DashboardData | null;
    status: 'Ready' | 'Fetching' | 'Error' | 'ListenerError';
    error: string | null;
    isLoading: boolean;
    useTauriMode: boolean;
    tauriApisLoaded: boolean;
    
    // Actions
    initialize: () => Promise<void>;
    triggerRefresh: () => Promise<void>;
    acknowledgeAlert: (alertId: string, userId: string) => Promise<void>;
    setUseTauriMode: (useTauri: boolean) => void;
    cleanupListener: () => void;
}

// Helper functions for different initialization modes
async function initializeTauriDashboard(set: any, get: any) {
    const { invoke } = await import('@tauri-apps/api/tauri');
    const { listen } = await import('@tauri-apps/api/event');
    
    try {
        set({ status: 'Fetching', isLoading: true });
        
        // Fetch initial dashboard data via Tauri command
        const data = await invoke('get_dashboard_data');
        set({ dashboardData: data, status: 'Ready', isLoading: false });
        
        // Listen for dashboard updates
        const unlisten = await listen('dashboard-update', (event) => {
            set({ dashboardData: event.payload, status: 'Ready' });
        });
        
        // Store unlisten function for cleanup
        set({ cleanup: unlisten });
    } catch (e) {
        console.error('Failed to initialize Tauri dashboard:', e);
        set({ 
            error: `Failed to initialize: ${e}`, 
            status: 'Error', 
            isLoading: false 
        });
    }
}

async function initializeWebDashboard(set: any, get: any) {
    try {
        set({ status: 'Fetching', isLoading: true });
        
        // Use WebApiClient for web mode
        const { WebApiClient } = await import('../services/WebApiClient');
        const client = new WebApiClient();
        
        // Fetch dashboard data via web API
        const data = await client.getDashboardData();
        set({ dashboardData: data, status: 'Ready', isLoading: false });
        
        // Set up WebSocket subscription for updates
        const { subscriptionId, unsubscribe } = await client.createSubscription('dashboard', 'update');
        client.onEvent('dashboard', 'update', (data) => {
            set({ dashboardData: data, status: 'Ready' });
        });
        
        // Store cleanup function
        set({ cleanup: unsubscribe });
    } catch (e) {
        console.error('Failed to initialize web dashboard:', e);
        set({ 
            error: `Failed to initialize: ${e}`, 
            status: 'Error', 
            isLoading: false 
        });
    }
}

export const useDashboardStore = create<DashboardState>((set, get) => ({
    // State
    dashboardData: null,
    status: 'Ready',
    error: null,
    isLoading: false,
    useTauriMode: false,
    tauriApisLoaded: false,
    cleanup: null,
    
    // Initialize dashboard
    initialize: async () => {
        const { useTauriMode } = get();
        
        // Try to load Tauri APIs
        const tauriResult = await loadTauriApis();
        set({ tauriApisLoaded: tauriResult.success });
        
        // If using Tauri mode and APIs are available, use them
        if (useTauriMode && tauriResult.success) {
            await initializeTauriDashboard(set, get);
        } else {
            // Otherwise, fall back to web API
            await initializeWebDashboard(set, get);
        }
    },
    
    // Trigger manual refresh
    triggerRefresh: async () => {
        const { useTauriMode, tauriApisLoaded } = get();
        
        set({ status: 'Fetching', isLoading: true });
        
        try {
            if (useTauriMode && tauriApisLoaded) {
                // Refresh via Tauri command
                const { invoke } = await import('@tauri-apps/api/tauri');
                const data = await invoke('get_dashboard_data');
                set({ dashboardData: data, status: 'Ready', isLoading: false });
            } else {
                // Refresh via web API
                const { WebApiClient } = await import('../services/WebApiClient');
                const client = new WebApiClient();
                const data = await client.getDashboardData();
                set({ dashboardData: data, status: 'Ready', isLoading: false });
            }
        } catch (e) {
            console.error('Failed to refresh dashboard:', e);
            set({ 
                error: `Refresh failed: ${e}`, 
                status: 'Error', 
                isLoading: false 
            });
        }
    },
    
    // Acknowledge alert
    acknowledgeAlert: async (alertId: string, userId: string) => {
        const { useTauriMode, tauriApisLoaded } = get();
        
        try {
            if (useTauriMode && tauriApisLoaded) {
                // Acknowledge via Tauri command
                const { invoke } = await import('@tauri-apps/api/tauri');
                await invoke('acknowledge_alert', { alertId, userId });
            } else {
                // Acknowledge via web API
                const { WebApiClient } = await import('../services/WebApiClient');
                const client = new WebApiClient();
                await client.acknowledgeAlert(alertId, userId);
            }
            
            // Trigger refresh to update UI
            get().triggerRefresh();
        } catch (e) {
            console.error('Failed to acknowledge alert:', e);
            set({ 
                error: `Failed to acknowledge alert: ${e}`, 
                status: 'Error' 
            });
        }
    },
    
    // Toggle between Tauri and web mode
    setUseTauriMode: (useTauri: boolean) => {
        set({ useTauriMode: useTauri });
        get().initialize(); // Re-initialize with new mode
    },
    
    // Clean up listeners
    cleanupListener: () => {
        const { cleanup } = get();
        if (cleanup) {
            cleanup();
        }
    }
}));
```

### 5. Fallback Mechanism

The bridge pattern includes fallback mechanisms to use the web API directly when Tauri APIs are unavailable:

```typescript
// src/utils/tauriUtils.ts
export async function loadTauriApis(modulePaths: string[] = [
    '@tauri-apps/api/tauri',
    '@tauri-apps/api/window',
    '@tauri-apps/api/event'
]): Promise<{success: boolean, modules: Record<string, any>}> {
    const result: Record<string, any> = {};
    
    try {
        for (const path of modulePaths) {
            const module = await import(/* @vite-ignore */ path);
            const moduleName = path.split('/').pop() || path;
            result[moduleName] = module;
        }
        
        return {
            success: true,
            modules: result
        };
    } catch (error) {
        console.error('Failed to load Tauri APIs:', error);
        return {
            success: false,
            modules: result
        };
    }
}

export function isTauriEnvironment(): boolean {
    return window.__TAURI__ !== undefined;
}

export async function getApiClient() {
    // Try to load Tauri APIs
    const isDesktop = isTauriEnvironment();
    
    if (isDesktop) {
        try {
            // Load TauriApiClient for desktop environment
            const { TauriApiClient } = await import('../services/TauriApiClient');
            return new TauriApiClient();
        } catch (error) {
            console.error('Failed to load TauriApiClient, falling back to WebApiClient:', error);
            // Fall back to WebApiClient
            const { WebApiClient } = await import('../services/WebApiClient');
            return new WebApiClient();
        }
    } else {
        // Use WebApiClient for browser environment
        const { WebApiClient } = await import('../services/WebApiClient');
        return new WebApiClient();
    }
}
```

## Integration Points

The bridge pattern integrates with the web UI at several key points:

### 1. Command Execution

The bridge enables command execution from the Tauri UI to the backend:

```typescript
// Execute command via Tauri
const result = await invoke('web_execute_command', {
    command: 'get_plugins',
    args: []
});

// or via direct web API when Tauri is unavailable
const result = await fetch('/api/commands/get_plugins', {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${token}` },
    body: JSON.stringify({})
});
```

### 2. Authentication

The bridge provides authentication functionality:

```typescript
// Via Tauri
const authResult = await invoke('web_login', {
    username,
    password
});

// or via direct web API
const authResult = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
});
```

### 3. WebSocket Communication

The bridge supports WebSocket communication for real-time updates:

```typescript
// Via Tauri
const subscriptionId = await invoke('web_create_subscription', {
    channel,
    event_type: eventType
});

// Listen for events
const unsubscribe = await event.listen(`web-event-${channel}-${eventType}`, (event) => {
    handleEvent(event.payload);
});

// or via direct web API
const ws = new WebSocket(`${wsUrl}?token=${token}`);
ws.onmessage = (event) => {
    handleEvent(JSON.parse(event.data));
};
```

## Testing Strategy

The bridge pattern is tested at multiple levels:

1. **Unit Tests**: Test individual components and functions
2. **Integration Tests**: Test the interaction between the bridge components
3. **End-to-End Tests**: Test the complete flow from UI to backend

```typescript
// Unit test for the WebApiClient
import { WebApiClient } from '../services/WebApiClient';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import * as tauriUtils from '../utils/tauriUtils';

// Mock Tauri's invoke function
vi.mock('@tauri-apps/api/tauri', () => ({
    invoke: vi.fn()
}));

// Mock Tauri's event listener
vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn().mockResolvedValue(() => {})
}));

// Mock our custom utility
vi.mock('../utils/tauriUtils', async () => {
    const actual = await vi.importActual('../utils/tauriUtils');
    return {
        ...actual,
        loadTauriApis: vi.fn()
    };
});

describe('WebApiClient', () => {
    let client: WebApiClient;
    
    beforeEach(() => {
        client = new WebApiClient();
        vi.clearAllMocks();
    });
    
    it('should execute a command via the WebApiClient', async () => {
        // Arrange
        const mockInvoke = vi.fn().mockResolvedValue({ success: true, data: 'test' });
        const { invoke } = await import('@tauri-apps/api/tauri');
        vi.mocked(invoke).mockImplementation(mockInvoke);
        
        // Act
        const result = await client.executeCommand('test-command', ['arg1', 'arg2']);
        
        // Assert
        expect(mockInvoke).toHaveBeenCalledWith('execute_web_command', {
            name: 'test-command',
            args: ['arg1', 'arg2']
        });
        expect(result).toEqual({ success: true, data: 'test' });
    });
    
    it('should create a subscription and return an unsubscribe function', async () => {
        // Arrange
        const mockInvoke = vi.fn().mockResolvedValue('subscription-123');
        const mockListen = vi.fn().mockResolvedValue(() => {});
        const { invoke } = await import('@tauri-apps/api/tauri');
        const { listen } = await import('@tauri-apps/api/event');
        vi.mocked(invoke).mockImplementation(mockInvoke);
        vi.mocked(listen).mockImplementation(mockListen);
        
        // Act
        const { subscriptionId, unsubscribe } = await client.createSubscription('test-channel', 'test-event');
        
        // Assert
        expect(mockInvoke).toHaveBeenCalledWith('web_create_subscription', {
            channel: 'test-channel',
            eventType: 'test-event'
        });
        expect(subscriptionId).toBe('subscription-123');
        expect(unsubscribe).toBeInstanceOf(Function);
    });
});
```

## Migration Path for Users

The bridge pattern provides a seamless migration path for users from the standalone web UI to the unified Tauri + React UI:

1. **Automatic Detection**: The application detects whether Tauri APIs are available
2. **Feature Parity**: All web UI features are available in the unified UI
3. **Familiar Interface**: The UI maintains a familiar layout and workflow
4. **Graceful Fallback**: When Tauri APIs are unavailable, the application falls back to web mode

## Implementation Status

The bridge pattern implementation is complete with all major features integrated:

| Feature | Status | Notes |
|---------|--------|-------|
| Command Execution | ✅ Complete | Full support for executing commands with parameter handling |
| Plugin Management | ✅ Complete | Ability to list, query, and interact with plugins |
| Authentication | ✅ Complete | Login, token refresh, and user info retrieval |
| WebSocket | ✅ Complete | Subscription, event handling, and message sending |
| Error Handling | ✅ Complete | Robust error handling and fallback mechanisms |
| Unit Tests | ✅ Complete | Comprehensive test coverage for bridge components |

## Conclusion

The Web Bridge pattern has successfully integrated the functionality from the standalone web UI into the unified Tauri + React application. This approach provides a seamless experience for users while leveraging the existing web implementation.

The bridge pattern demonstrates the effectiveness of a unified UI approach that can operate in both web and desktop contexts, providing a robust foundation for future development.

---

Last Updated: 2024-05-10 