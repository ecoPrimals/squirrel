# Squirrel UI Architecture Plan

## UI Component Roles

### 1. Legacy Terminal UI (`crates/ui-terminal`)
- **Role**: Legacy component, to be pruned once Tauri UI reaches feature parity
- **Use Case**: Local, direct access monitoring via command line
- **Environment**: Command line on local NAS or server
- **Technology**: Rust + ratatui
- **Status**: To be pruned

### 2. Desktop UI (`crates/ui-tauri-react` - Desktop Mode)
- **Role**: Primary local monitoring interface
- **Use Case**: Local NAS/server monitoring with rich UI
- **Environment**: Native desktop application running on the local system
- **Technology**: Tauri + React
- **Features**:
  - Full system monitoring
  - Plugin management
  - Direct access to local resources
  - Offline operation capability
  - System tray integration
  - Native notifications
- **Status**: Main development focus - mature GUI planned

### 3. Web UI (`crates/ui-tauri-react` - Web Mode)
- **Role**: Remote monitoring interface
- **Use Case**: LAN and Internet remote monitoring
- **Environment**: Web browser
- **Technology**: React (served by Tauri in dev mode, by web server in production)
- **Features**:
  - Authentication and security
  - Remote system monitoring
  - Plugin configuration
  - Cross-device compatibility
  - API-based data access
  - WebSocket for real-time updates
- **Status**: Secondary development focus

### 4. Modern Terminal UI (`crates/ui-tauri-react` - TUI Mode)
- **Role**: Fallback terminal interface for headless systems
- **Use Case**: Command-line monitoring when GUI is not available
- **Environment**: Terminal on local or SSH connection
- **Technology**: Tauri with terminal rendering (either via Rust TUI lib or JS-based TUI)
- **Features**:
  - Simplified system monitoring
  - Basic command execution
  - Lightweight resource usage
  - SSH-friendly interface
- **Status**: Planned as fallback option within Tauri UI codebase

## Unified UI Framework

With our updated architecture, we will have four UI modes but consolidated into two codebases:

1. **Legacy Terminal UI**: Will be pruned
2. **Tauri-based UI**: A unified framework with three modes:
   - **Desktop GUI Mode**: Rich, mature graphical interface (primary)
   - **Web Mode**: Browser-based remote interface
   - **Terminal Mode**: Modern TUI fallback for headless systems

### Codebase Organization

```
crates/ui-tauri-react/
├── src/                 # Shared React components and logic
│   ├── components/      # UI components (shared or mode-specific)
│   │   ├── desktop.tsx  # Desktop GUI mode
│   │   ├── web.tsx      # Web browser mode
│   │   └── terminal.tsx # Terminal UI mode
│   └── utils/           # Shared utilities
├── src-tauri/           # Rust backend for Tauri
│   ├── src/             # Rust code
│   │   ├── gui.rs       # GUI-specific functionality
│   │   ├── tui.rs       # Terminal UI functionality
│   │   └── main.rs      # Main entry point with mode detection
└── tui/                 # Terminal UI rendering components
    └── src/             # TUI-specific components
```

## Pruning Timeline for Legacy `ui-terminal`

### Criteria for Pruning
We can hard prune the legacy `ui-terminal` crate when:
1. The Tauri UI implements all core monitoring features present in the terminal UI
2. The new Tauri-based TUI mode provides adequate fallback functionality
3. Tests verify equivalent functionality in all UI modes

### Specific Milestones for Pruning
1. **Feature Parity Checklist**:
   - [ ] Dashboard overview
   - [ ] System metrics
   - [ ] Network monitoring
   - [ ] Plugin management
   - [ ] Alert handling
   - [ ] Health status display
   - [ ] MCP integration

2. **TUI Fallback Mode Implementation**:
   - [ ] Basic terminal rendering implementation
   - [ ] Navigation in terminal mode
   - [ ] System metrics display
   - [ ] Command execution in TUI
   - [ ] Error handling in terminal context

3. **UI Quality Checklist**:
   - [ ] Desktop UI runs stably
   - [ ] TUI fallback functions properly
   - [ ] All tests passing
   - [ ] Performance verified on target hardware
   - [ ] Proper error handling implemented

## Implementation Plan

### Phase 1: Current (1-2 weeks)
- Complete web UI demo mode
- Fix Tauri dependencies for desktop mode
- Design TUI fallback architecture
- Identify and implement missing core features from terminal UI

### Phase 2: Feature Completion (2-3 weeks)
- Implement remaining terminal UI features in Tauri UI (GUI mode)
- Enhance desktop-specific UX (system tray, notifications)
- Implement basic TUI fallback mode
- Improve web mode security and APIs

### Phase 3: Testing & Finalization (1-2 weeks)
- Comprehensive testing on target environments (all modes)
- Performance optimization
- Complete documentation update
- Ensure proper fallback to TUI mode when needed

### Phase 4: Pruning (1 day)
- Hard prune legacy `ui-terminal` crate
- Update build scripts and documentation
- Ensure TUI fallback mode works properly
- Redirect any existing terminal UI users to new Tauri UI with TUI mode

## Architectural Decisions

### 1. Unified Codebase with Mode Detection
The UI will detect its operating mode (desktop GUI, web, or terminal) and adapt accordingly:

```typescript
// Mode detection
const isDesktopMode = Boolean(window.__TAURI__) && !process.env.TAURI_TUI_MODE;
const isWebMode = !Boolean(window.__TAURI__) || import.meta.env.VITE_WEB_MODE === 'true';
const isTuiMode = Boolean(window.__TAURI__) && process.env.TAURI_TUI_MODE === 'true';

// Conditional UI rendering
if (isTuiMode) {
  return <TerminalInterface />;
} else {
  return <GraphicalInterface />;
}
```

### 2. Feature Flags for Environment-Specific Features
Use feature flags to enable/disable features based on environment:

```typescript
// Feature availability
export const features = {
  // GUI features
  systemTray: isDesktopMode && !isTuiMode,
  fileSystemAccess: isDesktopMode,
  nativeNotifications: isDesktopMode && !isTuiMode,
  offlineMode: isDesktopMode,
  
  // Web features
  remoteAccess: isWebMode,
  multiDeviceSync: isWebMode,
  
  // TUI features
  simplifiedNavigation: isTuiMode,
  asciiCharts: isTuiMode,
  lowBandwidthMode: isTuiMode
};
```

### 3. Responsive Design with Terminal Fallback
- **Desktop GUI**: Rich, mature graphical interface
- **Web**: Responsive from mobile to desktop
- **Terminal**: ASCII-based UI that works in standard terminals

### 4. TUI Implementation Options
1. **Rust-based TUI**: Using a Rust TUI library through Tauri commands
   ```rust
   #[tauri::command]
   fn render_tui() -> Result<String, String> {
       // Implement terminal rendering in Rust and return
       // output for the frontend to display
   }
   ```

2. **JavaScript-based TUI**: Using a JS terminal rendering library
   ```typescript
   import { Terminal } from 'terminal-kit';
   
   function TerminalUI() {
     // Implement terminal rendering in JS
   }
   ```

## Conclusion

This plan provides a clear path forward for our UI architecture, with specific roles for each component and a concrete plan for pruning the legacy terminal UI. By focusing on a unified Tauri-based codebase with multiple mode options (desktop GUI, web, and TUI fallback), we'll create a flexible and robust UI system that works in all environments while maintaining a modern, mature GUI as the primary interface. 