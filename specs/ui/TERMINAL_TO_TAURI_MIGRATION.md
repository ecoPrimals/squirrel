# Terminal UI to Tauri+React UI Migration Plan

## Overview

This document outlines the plan for migrating from the legacy terminal-based UI (`crates/ui-terminal`) to the modern Tauri+React UI (`crates/ui-tauri-react`). The migration will include implementing a new TUI fallback mode within the Tauri framework, which will replace the standalone terminal UI while maintaining terminal-based access capabilities. Since this is a home project, we can perform a hard prune of the legacy terminal UI as soon as feature parity is reached and the new TUI fallback is implemented.

## UI Role Separation

### Desktop GUI Mode Role
- **Primary local monitoring interface**
- Running as a native application on the NAS/server system
- Featuring a mature, sophisticated graphical interface
- Providing direct access to local resources
- Supporting offline operation
- Integrating with the system tray and native notifications

### Web UI Mode Role
- **Remote monitoring interface**
- Accessible via LAN and Internet connections
- Focusing on secure remote access
- Providing cross-device compatibility
- Supporting WebSocket for real-time updates

### TUI Fallback Mode Role
- **Terminal-based monitoring interface**
- Integrated within the Tauri framework (not standalone)
- Providing command-line access for headless systems
- Supporting SSH sessions for remote terminal access
- Offering lightweight resource consumption

## Feature Mapping

### Core Features to Migrate

| Legacy Terminal UI Feature | Target Tauri UI Component | Implementation Modes |
|---------------------|------------------------------|--------|
| Dashboard Overview  | `Dashboard.tsx` / `DashboardTUI.ts` | GUI, Web, TUI |
| System Metrics      | `SystemWidget.tsx` / `SystemWidgetTUI.ts` | GUI, Web, TUI |
| Network Monitoring  | `NetworkWidget.tsx` / `NetworkWidgetTUI.ts` | GUI, Web, TUI |
| Plugin Management   | `PluginManager.tsx` / `PluginManagerTUI.ts` | GUI, Web, TUI |
| Alert Handling      | `AlertsWidget.tsx` / `AlertsWidgetTUI.ts` | GUI, Web, TUI |
| AI Chat             | `AIChat.tsx` / `AIChatTUI.ts` | GUI, Web, TUI |
| Protocol Viewer     | `ProtocolView.tsx` / `ProtocolViewTUI.ts` | GUI, Web, TUI |
| Health Status       | `HealthStatus.tsx` / `HealthStatusTUI.ts` | GUI, Web, TUI |

### Back-end Service Integration

| Service | Legacy Terminal Integration | Tauri Integration | UI Modes |
|---------|---------------------|-------------------|--------|
| Dashboard Core | Direct Rust integration | Tauri commands | All modes |
| Monitoring     | Direct metrics reading | Tauri API calls | All modes |
| Plugin System  | Direct plugin loading  | Tauri plugin API | All modes |
| MCP            | Terminal MCP client    | Web MCP bridge  | All modes |
| AI Tools       | Terminal AI client     | Web AI interface | All modes |

## Migration Phases

### Phase 1: Parallel Development (Current)

- Continue maintaining legacy terminal UI
- Complete test infrastructure for Tauri+React UI
- Ensure all backend service integrations work via Tauri commands
- Fix Tauri desktop app dependency issues
- Complete web demo mode functionality
- Design TUI fallback mode architecture

**Timeline:** 1-2 weeks

### Phase 2: Feature Parity & TUI Implementation

- Implement all remaining features in Tauri UI (GUI mode)
- Develop TUI fallback mode within Tauri framework
- Create TUI-specific components and interfaces
- Implement mode switching between GUI and TUI
- Address any performance or usability issues
- Verify cross-platform compatibility

**Timeline:** 2-3 weeks

### Phase 3: Pruning

- Verify feature parity across all modes (GUI, Web, TUI)
- Hard prune the legacy terminal UI
- Follow the steps in `PRUNING_STRATEGY.md`
- Complete documentation updates
- Ensure clean removal without dependency issues

**Timeline:** 1 day (since this is a home project, we can move quickly)

## Implementation Details

### Component Migration Patterns

For each terminal UI component, follow this migration pattern:

1. **Identify Component Responsibilities**
   - Document all functionality of the legacy terminal component
   - Note all data sources and backend interactions
   - Capture all user interaction patterns

2. **Design Multi-mode Components**
   - Create GUI version using React components
   - Develop TUI version for terminal fallback
   - Ensure shared data models between versions
   - **Implement mode detection** for GUI/Web/TUI differences

3. **Implement Backend Integration**
   - Use Tauri commands for all modes
   - Implement real-time updates via events or WebSockets
   - Ensure error handling and loading states
   - Share backend integration code across modes

4. **Test Thoroughly**
   - Create unit tests for component behavior
   - Test integration with backend services
   - Verify performance with large datasets
   - **Test in all modes** (GUI, Web, TUI)

Example implementation with mode-specific components:

```typescript
// Base shared component logic
class MetricsMonitor {
  // Shared data fetching logic
  async fetchMetrics() {
    return await invoke('get_system_metrics');
  }
  
  // Shared data processing logic
  processMetrics(data) {
    // Common processing
    return processedData;
  }
}

// GUI Mode Component
const SystemWidgetGUI: React.FC = () => {
  const metrics = useMetrics(); // Shared hook using MetricsMonitor
  
  return (
    <Card title="System Metrics">
      <CPUUsageChart data={metrics?.cpu} />
      <MemoryUsageChart data={metrics?.memory} />
      <DiskUsageChart data={metrics?.disk} />
      <StatusIndicator status={metrics?.status} />
    </Card>
  );
};

// TUI Mode Component
const SystemWidgetTUI = () => {
  const metrics = useMetrics(); // Same shared hook
  
  return `
    ┌─── System Metrics ────────────┐
    │ CPU: ${'#'.repeat(metrics.cpu / 10)} ${metrics.cpu}% │
    │ MEM: ${'#'.repeat(metrics.memory / 10)} ${metrics.memory}% │
    │ DISK: ${'#'.repeat(metrics.disk / 10)} ${metrics.disk}% │
    │ Status: ${metrics.status === 'ok' ? '✓' : '✗'}  │
    └────────────────────────────────┘
  `;
};

// Mode selector component
const SystemWidget = () => {
  // Detect operating mode
  const { isTuiMode } = useUIMode();
  
  return isTuiMode 
    ? <SystemWidgetTUI /> 
    : <SystemWidgetGUI />;
};
```

### TUI Implementation Approaches

1. **React-based Terminal Renderer**

Use a React-based terminal renderer that can be integrated with the existing React components:

```typescript
import { Terminal } from 'react-terminal-ui';

const TUIMode: React.FC = () => {
  const [output, setOutput] = useState('');
  
  useEffect(() => {
    // Generate TUI output and update
    const generateTUI = async () => {
      const metrics = await invoke('get_system_metrics');
      setOutput(renderTUI(metrics));
    };
    
    generateTUI();
    const interval = setInterval(generateTUI, 1000);
    return () => clearInterval(interval);
  }, []);
  
  return <Terminal content={output} />;
};
```

2. **Rust-based TUI Renderer**

Use a Rust-based TUI renderer that runs via Tauri commands:

```rust
#[tauri::command]
fn render_tui_dashboard(state: State<AppState>) -> Result<String, String> {
  let metrics = state.get_metrics()?;
  let tui_output = format!(
    "┌─── System Metrics ────────────┐\n\
     │ CPU: {:50} {:3}% │\n\
     │ MEM: {:50} {:3}% │\n\
     │ DISK: {:50} {:3}% │\n\
     │ Status: {:8}             │\n\
     └────────────────────────────────┘",
    "#".repeat((metrics.cpu / 2) as usize), metrics.cpu,
    "#".repeat((metrics.memory / 2) as usize), metrics.memory,
    "#".repeat((metrics.disk / 2) as usize), metrics.disk,
    if metrics.status == "ok" { "✓" } else { "✗" }
  );
  
  Ok(tui_output)
}
```

## Testing Strategy

1. **Component Testing**
   - Unit tests for all React components
   - Unit tests for TUI rendering functions
   - Mock Tauri invoke calls for all modes
   - Test state management and UI updates

2. **Integration Testing**
   - Test GUI mode with actual Tauri backend
   - Test TUI mode with terminal rendering
   - Test web mode with actual API server
   - Verify data flow between UI and services
   - Test all user interaction patterns

3. **Migration Testing**
   - Side-by-side feature comparison between legacy TUI and new TUI mode
   - Verify all terminal UI features are available in new TUI mode
   - Test with real-world usage patterns

## Fallback Strategy

Since this is a home project, we can be more aggressive with pruning, but will still:

1. Archive the legacy terminal UI codebase before removal
2. Ensure the new TUI mode works properly before pruning
3. Keep the pruning steps reversible as outlined in PRUNING_STRATEGY.md
4. Ensure proper testing before final removal

## Conclusion

This migration plan provides a structured approach to transitioning from the legacy terminal-based UI to the modern Tauri+React UI with a new integrated TUI fallback mode. By following a phased approach and ensuring thorough testing across all modes, we can maintain functionality while providing users with a more powerful and user-friendly interface. Since this is a home project, we can move quickly and prune the legacy terminal UI as soon as feature parity is reached and the new TUI mode is implemented. 