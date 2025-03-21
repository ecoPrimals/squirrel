use std::io::{self, Write};
use crate::ui::{Component, error::UiError, layout::Rect, theme::Theme, Size};
use crate::ui::components::registry::ComponentId;

/// Defines the horizontal alignment of text within a table cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    /// Aligns text to the left side of the cell.
    Left,
    /// Centers text within the cell.
    Center,
    /// Aligns text to the right side of the cell.
    Right,
}

/// Represents a column in a table.
#[derive(Debug, Clone)]
pub struct Column {
    /// The header text for the column.
    pub header: String,
    /// The alignment of content in this column.
    pub alignment: Alignment,
    /// The minimum width of the column.
    pub min_width: usize,
}

/// Represents a row in a table.
#[derive(Debug, Clone)]
pub struct Row {
    /// The cells in this row.
    pub cells: Vec<String>,
}

/// Represents the style of a table.
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// Whether to show borders between cells.
    pub show_borders: bool,
    /// Whether to show the header row.
    pub show_header: bool,
}

/// A table component for displaying tabular data.
#[derive(Debug)]
pub struct Table {
    /// The component ID.
    id: ComponentId,
    /// The columns in the table.
    columns: Vec<Column>,
    /// The rows in the table.
    rows: Vec<Row>,
    /// The style configuration for the table.
    #[allow(dead_code)]
    style: TableStyle,
}

impl Table {
    /// Creates a new table with the given columns and style.
    #[must_use]
    pub fn new(columns: Vec<Column>, style: TableStyle) -> Self {
        Self {
            id: ComponentId::new("table"),
            columns,
            rows: Vec::new(),
            style,
        }
    }

    /// Adds a row to the table.
    pub fn add_row(&mut self, cells: Vec<String>) -> Result<(), UiError> {
        if cells.len() != self.columns.len() {
            return Err(UiError::InvalidInput(format!(
                "Row has {} cells but table has {} columns",
                cells.len(),
                self.columns.len()
            )));
        }
        self.rows.push(Row { cells });
        Ok(())
    }
}

impl Component for Table {
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn render(&self, _writer: &mut dyn Write, _rect: Rect, _theme: &Theme) -> io::Result<()> {
        // TODO: Implement table rendering
        Ok(())
    }

    fn minimum_size(&self) -> Size {
        // TODO: Calculate minimum size based on content
        Size::new(0, 0)
    }

    fn preferred_size(&self) -> Size {
        // TODO: Calculate preferred size based on content
        Size::new(0, 0)
    }

    fn update(&mut self) -> io::Result<()> {
        Ok(())
    }
} 