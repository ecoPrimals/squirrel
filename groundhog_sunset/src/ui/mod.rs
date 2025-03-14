use std::io::{self, Write, Stdout};
use std::error::Error as StdError;
use std::fmt;
use crossterm::{
    event::KeyEvent,
};
use crate::ui::components::ComponentId;
use crate::ui::layout::{Rect, Size};
use crate::ui::theme::Theme;

/// UI components like headers, status bars, and input fields.
pub mod components;

/// Error types for UI operations.
pub mod error;

/// Header component for displaying titles and subtitles.
pub mod header;

/// Input handling and text input components.
pub mod input;

/// Layout management and positioning utilities.
pub mod layout;

/// Status message and notification components.
pub mod status;

/// Table display and data grid components.
pub mod table;

/// Terminal initialization and cleanup utilities.
mod terminal;

/// Theme management and styling components.
pub mod theme;

use self::table::{Table, Column, Alignment, TableStyle, TableError};

pub use error::UiError;
pub use theme::{ThemeManager, Themeable, ColorRole, DefaultThemeManager};
pub use layout::{LayoutManager, LayoutError, Position, Spacing, constraint::Constraint, grid::GridConfig};

/// Error types that can occur during UI operations.
#[derive(Debug)]
pub enum UIError {
    /// An I/O error occurred during UI operations.
    IOError(io::Error),
    /// An error occurred while processing table operations.
    TableError(TableError),
    /// An error occurred while processing input operations.
    InputError(String),
    /// An error occurred in the layout system.
    LayoutError(LayoutError),
    /// Attempted to use grid layout features when no grid layout was configured.
    NoGridLayout,
    /// An error occurred while processing theme operations.
    Theme(theme::ThemeError),
    /// An error occurred while processing a component operation.
    Component(String),
    /// The component was in an invalid state for the requested operation.
    InvalidState(String),
}

impl fmt::Display for UIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UIError::IOError(err) => write!(f, "IO error: {}", err),
            UIError::TableError(err) => write!(f, "Table error: {}", err),
            UIError::InputError(msg) => write!(f, "Input error: {}", msg),
            UIError::LayoutError(err) => write!(f, "Layout error: {}", err),
            UIError::NoGridLayout => write!(f, "No grid layout"),
            UIError::Theme(err) => write!(f, "Theme error: {}", err),
            UIError::Component(msg) => write!(f, "Component error: {}", msg),
            UIError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl StdError for UIError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            UIError::IOError(err) => Some(err),
            UIError::TableError(err) => Some(err),
            UIError::InputError(_) => None,
            UIError::LayoutError(err) => Some(err),
            UIError::NoGridLayout => None,
            UIError::Theme(err) => Some(err),
            UIError::Component(_) => None,
            UIError::InvalidState(_) => None,
        }
    }
}

impl From<io::Error> for UIError {
    fn from(err: io::Error) -> Self {
        UIError::IOError(err)
    }
}

impl From<TableError> for UIError {
    fn from(err: TableError) -> Self {
        UIError::TableError(err)
    }
}

/// A trait representing a UI component that can be rendered and managed by the UI system.
/// 
/// Components implementing this trait must provide:
/// - A unique identifier
/// - Rendering capabilities
/// - Size requirements
/// - Layout update handling
/// - State management
pub trait Component: Send + Sync {
    /// Returns the unique identifier for this component.
    /// 
    /// This ID is used to track and manage the component within the UI system.
    fn id(&self) -> &ComponentId;

    /// Renders the component to the given writer within the specified rectangle using the provided theme.
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The output writer to render to
    /// * `rect` - The rectangle defining the component's boundaries
    /// * `theme` - The theme to use for styling
    /// 
    /// # Returns
    /// 
    /// * `io::Result<()>` - The result of the rendering operation
    fn render(&self, writer: &mut dyn Write, rect: Rect, theme: &Theme) -> io::Result<()>;

    /// Returns the minimum size required to render this component.
    /// 
    /// This is used by the layout system to ensure the component has enough space.
    fn minimum_size(&self) -> Size;

    /// Returns the preferred size for this component.
    /// 
    /// This is used by the layout system to optimize component placement.
    fn preferred_size(&self) -> Size;

    /// Called when the component's layout has been updated.
    /// 
    /// # Arguments
    /// 
    /// * `rect` - The new rectangle defining the component's boundaries
    fn on_layout(&mut self, _rect: Rect) {}

    /// Updates the component's state.
    /// 
    /// This method is called periodically to allow the component to update its internal state.
    /// 
    /// # Returns
    /// 
    /// * `io::Result<()>` - The result of the update operation
    fn update(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A trait for components that can be updated independently.
pub trait Updateable {
    /// Updates the component's state.
    /// 
    /// # Returns
    /// 
    /// * `io::Result<()>` - The result of the update operation
    fn update(&mut self) -> io::Result<()>;
}

impl Updateable for dyn Component {
    fn update(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Manages the UI state and layout for the application.
/// 
/// This struct handles:
/// - Component layout and positioning
/// - Terminal size management
/// - Theme configuration
/// - Grid-based layout system
pub struct UiManager {
    /// The layout manager for positioning UI elements.
    layout: LayoutManager,
    /// The current size of the terminal.
    terminal_size: Size,
    /// The theme configuration for the UI.
    theme: theme::Theme,
}

impl UiManager {
    /// Creates a new UI manager with default settings.
    /// 
    /// # Returns
    /// 
    /// * `Self` - A new UI manager instance
    pub fn new() -> Self {
        Self {
            layout: LayoutManager::new(),
            terminal_size: Size::new(0, 0),
            theme: theme::Theme::default(),
        }
    }

    /// Updates the terminal size.
    /// 
    /// # Arguments
    /// 
    /// * `width` - The new terminal width
    /// * `height` - The new terminal height
    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = Size::new(width, height);
    }

    /// Returns the current terminal size.
    /// 
    /// # Returns
    /// 
    /// * `Size` - The current terminal dimensions
    pub fn get_terminal_size(&self) -> Size {
        self.terminal_size
    }

    /// Sets up a grid layout with the specified dimensions.
    /// 
    /// # Arguments
    /// 
    /// * `rows` - The number of rows in the grid
    /// * `columns` - The number of columns in the grid
    pub fn setup_grid(&mut self, rows: u16, columns: u16) {
        let config = GridConfig::new(rows, columns)
            .with_spacing(Spacing::uniform(1))
            .with_margin(Spacing::uniform(0));
        self.layout.setup_grid(config);
    }

    /// Adds a component to the grid layout.
    /// 
    /// # Arguments
    /// 
    /// * `row` - The starting row position
    /// * `col` - The starting column position
    /// * `_component` - The component to add
    /// * `row_span` - Number of rows the component spans
    /// * `col_span` - Number of columns the component spans
    /// 
    /// # Returns
    /// 
    /// * `Result<(), UIError>` - The result of the operation
    pub fn add_component<T: Component>(
        &mut self,
        row: u16,
        col: u16,
        _component: &T,
        row_span: u16,
        col_span: u16,
    ) -> Result<(), UIError> {
        let grid = self.layout.get_grid_mut().ok_or(UIError::NoGridLayout)?;
        let rect = Rect::new(Position::new(0, 0), Size::new(0, 0));
        let cell = layout::grid::GridCell::with_span(rect, row_span, col_span);
        grid.add_cell(row, col, cell).map_err(UIError::LayoutError)?;
        Ok(())
    }

    /// Calculates the layout for all components.
    /// 
    /// # Returns
    /// 
    /// * `Result<(), UIError>` - The result of the operation
    pub fn calculate_layout(&mut self) -> Result<(), UIError> {
        if let Some(grid) = self.layout.get_grid_mut() {
            grid.calculate_layout(self.terminal_size)
                .map_err(UIError::LayoutError)?;
        }
        Ok(())
    }

    /// Gets the rectangle for a component at the specified grid position.
    /// 
    /// # Arguments
    /// 
    /// * `row` - The row position
    /// * `col` - The column position
    /// 
    /// # Returns
    /// 
    /// * `Option<Rect>` - The component's rectangle if found
    pub fn get_component_rect(&self, row: u16, col: u16) -> Option<Rect> {
        self.layout
            .get_grid()
            .and_then(|grid| grid.get_cell(row, col))
            .map(|cell| cell.rect)
    }

    /// Clears the terminal screen.
    /// 
    /// # Arguments
    /// 
    /// * `writer` - The output writer to clear
    /// 
    /// # Returns
    /// 
    /// * `io::Result<()>` - The result of the operation
    pub fn clear_screen<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "\x1B[2J\x1B[1;1H")
    }

    /// Returns a reference to the current theme.
    /// 
    /// # Returns
    /// 
    /// * `&Theme` - The current theme
    pub fn theme(&self) -> &theme::Theme {
        &self.theme
    }

    /// Returns a mutable reference to the current theme.
    /// 
    /// # Returns
    /// 
    /// * `&mut Theme` - A mutable reference to the current theme
    pub fn theme_mut(&mut self) -> &mut theme::Theme {
        &mut self.theme
    }
}

impl Default for UiManager {
    fn default() -> Self {
        Self::new()
    }
}

/// The main UI struct that manages the terminal interface.
/// 
/// This struct provides:
/// - Terminal output management
/// - Table handling
/// - Progress tracking
/// - Theme management
pub struct UI<W: Write + From<Stdout>> {
    /// The layout manager for positioning UI elements.
    layout: LayoutManager,
    /// The current size of the terminal.
    /// This field is used for layout calculations and window resizing.
    #[allow(dead_code)]
    terminal_size: Size,
    /// The theme configuration for the UI.
    /// This field is used for consistent styling across components.
    #[allow(dead_code)]
    theme: theme::Theme,
    /// The output writer for terminal operations.
    pub stdout: W,
    /// The last progress update that was displayed, stored as (message, progress).
    pub last_progress: Option<(String, f32)>,
    /// The collection of tables currently being managed.
    pub tables: Vec<Table>,
}

impl<W: Write + From<Stdout>> UI<W> {
    /// Creates a new UI instance with default settings.
    pub fn new() -> Self {
        Self {
            layout: LayoutManager::new(),
            terminal_size: Size::new(80, 24), // Default terminal size
            theme: theme::Theme::default(),
            stdout: W::from(io::stdout()),
            last_progress: None,
            tables: Vec::new(),
        }
    }

    /// Resets the UI state, clearing all tables and progress information.
    pub fn reset(&mut self) {
        self.layout.reset();
    }

    /// Creates a new table and returns its index.
    pub fn create_table(&mut self) -> usize {
        let table = Table::new();
        self.tables.push(table);
        self.tables.len() - 1
    }

    /// Adds a column to the specified table.
    ///
    /// # Arguments
    /// * `table_index` - The index of the table to add the column to
    /// * `title` - The title of the column
    /// * `width` - The width of the column in characters
    pub fn add_column(&mut self, table_index: usize, title: &str, width: usize) -> std::result::Result<(), UIError> {
        if let Some(table) = self.tables.get_mut(table_index) {
            table.add_column(Column::new(title, width, Alignment::Left));
            Ok(())
        } else {
            Err(UIError::TableError(TableError::InvalidTable))
        }
    }

    /// Adds a row to the specified table.
    ///
    /// # Arguments
    /// * `table_index` - The index of the table to add the row to
    /// * `cells` - The cell values for the row
    pub fn add_row(&mut self, table_index: usize, cells: Vec<&str>) -> std::result::Result<(), UIError> {
        if let Some(table) = self.tables.get_mut(table_index) {
            table.add_row(cells)?;
            Ok(())
        } else {
            Err(UIError::TableError(TableError::InvalidTable))
        }
    }

    /// Sets the style for the specified table.
    ///
    /// # Arguments
    /// * `table_index` - The index of the table to style
    /// * `style` - The style to apply to the table
    pub fn set_table_style(&mut self, table_index: usize, style: TableStyle) -> std::result::Result<(), UIError> {
        if let Some(table) = self.tables.get_mut(table_index) {
            table.set_style(style);
            Ok(())
        } else {
            Err(UIError::TableError(TableError::InvalidTable))
        }
    }

    /// Renders the specified table to the output.
    ///
    /// # Arguments
    /// * `table_index` - The index of the table to render
    pub fn render_table(&mut self, table_index: usize) -> std::result::Result<(), UIError> {
        if let Some(table) = self.tables.get(table_index) {
            table.render(&mut self.stdout)?;
            Ok(())
        } else {
            Err(UIError::TableError(TableError::InvalidTable))
        }
    }

    /// Handles keyboard input for the specified table.
    ///
    /// # Arguments
    /// * `table_index` - The index of the table to handle input for
    /// * `key` - The key event to handle
    pub fn handle_key(&mut self, table_index: usize, key: KeyEvent) -> std::result::Result<(), UIError> {
        if let Some(table) = self.tables.get_mut(table_index) {
            table.handle_key(key)?;
            Ok(())
        } else {
            Err(UIError::TableError(TableError::InvalidTable))
        }
    }
}

impl Default for UI<Stdout> {
    fn default() -> Self {
        Self::new()
    }
}