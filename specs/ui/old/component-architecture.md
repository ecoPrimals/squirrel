---
title: Squirrel UI Component Architecture
version: 1.0.0
date: 2024-03-26
status: planning
---

# Squirrel UI Component Architecture

## Overview

This document outlines the component architecture for the Squirrel AI Coding Assistant's Ratatui-based UI system. It defines the structure, responsibilities, and relationships between UI components.

## Design Principles

The component architecture follows these core principles:

1. **Separation of Concerns**: Each component has a well-defined, focused responsibility
2. **Composition Over Inheritance**: Components are designed to be composed rather than extended
3. **State Isolation**: Component state is isolated and only exposed through controlled interfaces
4. **Testability**: Components are designed for easy testing with minimal dependencies
5. **Reusability**: Core components are reusable across different screens and contexts

## Component Hierarchy

The UI system follows a hierarchical structure:

```
┌─────────────────────────────────┐
│          Application            │
└─────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│             Screens             │
└─────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│           Containers            │
└─────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│            Widgets              │
└─────────────────────────────────┘
```

## Component Types

### 1. Application Components

The top-level application components that manage the overall UI:

- **SquirrelTui**: Main application entry point and coordinator
- **EventHandler**: Manages input and system events
- **ThemeManager**: Provides styling and theming capabilities
- **ScreenManager**: Controls screen navigation and history

### 2. Screen Components

Full-screen user interfaces for specific functions:

- **MainScreen**: Primary user interface with command input and output
- **ContextScreen**: View and edit context information
- **HelpScreen**: Display documentation and keyboard shortcuts
- **ToolScreen**: View and interact with running tools
- **SettingsScreen**: Configure application settings

### 3. Container Components

Layout components that organize and arrange widgets:

- **SplitContainer**: Horizontal or vertical split with adjustable divider
- **TabContainer**: Multiple tabs with selection capabilities
- **ScrollContainer**: Scrollable container for overflow content
- **CardContainer**: Content with header, body, and optional footer
- **ModalContainer**: Floating modal dialog with overlay

### 4. Widget Components

Individual UI elements for specific interactions:

- **CommandInput**: Text input with history and autocomplete
- **CommandOutput**: Formatted command execution results
- **ContextTree**: Hierarchical visualization of context
- **ToolList**: List of available tools with filtering
- **StatusBar**: Application status information
- **ProgressIndicator**: Visual feedback for operations
- **CodeView**: Syntax-highlighted code display
- **MessageList**: Conversation message display

## Component Details

### Core Components

#### SquirrelTui

```rust
pub struct SquirrelTui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    screen_manager: ScreenManager,
    event_handler: EventHandler,
    theme_manager: ThemeManager,
    app_state: AppState,
}
```

Responsibilities:
- Initialize and configure terminal
- Coordinate event handling
- Manage rendering pipeline
- Handle application lifecycle

#### ScreenManager

```rust
pub struct ScreenManager {
    screens: HashMap<ScreenId, Box<dyn Screen>>,
    screen_stack: Vec<ScreenId>,
    current_screen: ScreenId,
}
```

Responsibilities:
- Register available screens
- Navigate between screens
- Maintain screen history
- Manage screen lifecycle events

#### ThemeManager

```rust
pub struct ThemeManager {
    theme: Theme,
    color_palette: ColorPalette,
    styles: HashMap<StyleId, Style>,
}
```

Responsibilities:
- Provide consistent styling across components
- Map logical style identifiers to Ratatui styles
- Support theme switching
- Handle terminal color capabilities

### Screen Components

#### MainScreen

```rust
pub struct MainScreen {
    id: ScreenId,
    layout: Layout,
    command_input: CommandInput,
    command_output: CommandOutput,
    status_bar: StatusBar,
    tool_panel: ToolPanel,
}
```

Responsibilities:
- Provide primary user interface
- Handle command input and execution
- Display command output
- Show execution status

#### ContextScreen

```rust
pub struct ContextScreen {
    id: ScreenId,
    layout: Layout,
    context_tree: ContextTree,
    context_details: ContextDetails,
    context_controls: ContextControls,
}
```

Responsibilities:
- Display hierarchical context visualization
- Allow context navigation and selection
- Show detailed context information
- Provide context editing capabilities

### Widget Components

#### CommandInput

```rust
pub struct CommandInput {
    id: WidgetId,
    input_state: InputState,
    history: CommandHistory,
    completion_engine: CompletionEngine,
    validator: CommandValidator,
}
```

Responsibilities:
- Capture user input
- Provide command history navigation
- Offer command completion
- Validate input syntax

#### ContextTree

```rust
pub struct ContextTree {
    id: WidgetId,
    root_items: Vec<ContextItem>,
    expanded_state: HashMap<String, bool>,
    selected_path: Option<String>,
    render_config: TreeRenderConfig,
}
```

Responsibilities:
- Display hierarchical context structure
- Allow navigation and selection
- Handle expand/collapse interactions
- Support filtering and search

## Component Interactions

### Event Flow

Events flow through the component hierarchy:

1. **EventHandler** captures system and user events
2. Events are passed to the **SquirrelTui** application
3. **ScreenManager** routes events to the active screen
4. Active screen delegates to its child components
5. Components handle events or propagate to children
6. Results flow back up the hierarchy

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ EventHandler│────▶│ SquirrelTui │────▶│ScreenManager│
└─────────────┘     └─────────────┘     └─────────────┘
                                                │
                                                ▼
                                        ┌─────────────┐
                                        │Active Screen│
                                        └─────────────┘
                                                │
                                                ▼
                                        ┌─────────────┐
                                        │  Containers │
                                        └─────────────┘
```

### Render Flow

Rendering flows from the top down:

1. **SquirrelTui** initiates rendering with terminal frame
2. **ScreenManager** delegates to active screen
3. Screen computes layout and delegates to containers
4. Containers delegate to child widgets
5. Widgets render directly to the frame

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ SquirrelTui │────▶│ScreenManager│────▶│Active Screen│
└─────────────┘     └─────────────┘     └─────────────┘
                                                │
                                                ▼
                                        ┌─────────────┐
                                        │  Containers │
                                        └─────────────┘
                                                │
                                                ▼
                                        ┌─────────────┐
                                        │   Widgets   │
                                        └─────────────┘
                                                │
                                                ▼
                                        ┌─────────────┐
                                        │ Frame Buffer│
                                        └─────────────┘
```

## State Management

### Component State Types

The UI system uses several state types:

1. **AppState**: Global application state
2. **ScreenState**: Screen-specific state
3. **WidgetState**: Widget-specific state
4. **InputState**: State for input components
5. **SelectionState**: State for selectable components

### State Flow

State changes follow a unidirectional flow:

1. User interaction or system event triggers state change
2. State is updated through controlled interfaces
3. State changes trigger re-renders
4. Components read state during render
5. Rendered output reflects current state

## Layout System

The UI system uses Ratatui's constraint-based layout system:

```rust
pub struct LayoutConfig {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: Margin,
}

pub struct Layout {
    config: LayoutConfig,
    areas: Vec<Rect>,
}
```

Layout is computed hierarchically:
1. Screen creates root layout areas
2. Containers create child layout areas
3. Widgets render within assigned areas

## Styling System

The styling system maps logical style identifiers to Ratatui styles:

```rust
pub enum StyleId {
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Header,
    Body,
    Footer,
    // ...
}

pub struct Theme {
    name: String,
    styles: HashMap<StyleId, Style>,
    color_palette: ColorPalette,
}
```

Components request styles by logical identifier rather than hard-coding specific styles.

## Component Integration with Core Features

### Command System Integration

```rust
impl CommandSystem {
    pub fn register_ui(&mut self, ui: &SquirrelTui) {
        let event_sender = ui.event_handler.sender();
        
        // Register command execution callback
        self.on_command_execution(move |result| {
            let _ = event_sender.send(Event::CommandResult(result));
        });
    }
}
```

### Context Management Integration

```rust
impl ContextManager {
    pub fn register_ui(&mut self, ui: &SquirrelTui) {
        let event_sender = ui.event_handler.sender();
        
        // Register context change callback
        self.on_context_change(move |context| {
            let _ = event_sender.send(Event::ContextUpdate(context));
        });
    }
}
```

## Accessibility Considerations

The component architecture supports accessibility through:

1. **Keyboard Navigation**: All interactions available via keyboard
2. **Focus Management**: Clear focus indicators and navigation
3. **Color Contrast**: Theme system enforces minimum contrast ratios
4. **Screen Reader Support**: Text alternatives for visual elements
5. **Customizable UI**: Settings for font size, colors, and layout

## Component Implementation Priorities

Implementation will follow this priority order:

1. **Core Infrastructure**: SquirrelTui, EventHandler, ScreenManager, ThemeManager
2. **Essential Widgets**: CommandInput, CommandOutput, StatusBar
3. **Main Screen**: Primary user interface integration
4. **Context Components**: ContextTree, ContextDetails
5. **Tool Visualization**: ToolList, ToolDetails
6. **Secondary Screens**: Help, Settings
7. **Advanced Features**: Keyboard shortcuts, theming, accessibility

## References

- [Ratatui Widget Gallery](https://github.com/ratatui-org/ratatui/blob/main/examples/)
- [Ratatui Layout Documentation](https://docs.rs/ratatui/latest/ratatui/layout/index.html)
- [Squirrel Command System](../commands/README.md)
- [Squirrel Context Management](../context/README.md) 