use std::io::{self, Write};
use crossterm::{
    execute,
    style::{Color as CrosstermColor, ResetColor, SetForegroundColor, Stylize},
    cursor::MoveToNextLine,
};
use crate::ui::theme::Color as UiColor;
use crate::ui::theme::Style;
use std::error::Error as StdError;
use std::fmt;
use crossterm::event::{KeyEvent, KeyCode};
use ratatui::style::Color as RatatuiColor;

/// Error type for table operations
#[derive(Debug, thiserror::Error)]
pub enum TableError {
    /// IO error occurred
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    /// Invalid row data
    #[error("Invalid row: {0}")]
    InvalidRow(String),
    /// Invalid column data
    #[error("Invalid column: {0}")]
    InvalidColumn(String),
    /// Invalid style data
    #[error("Invalid style: {0}")]
    InvalidStyle(String),
    /// Invalid table reference
    #[error("Invalid table reference")]
    InvalidTable,
}

/// Text alignment within a table cell
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Left-aligned text
    Left,
    /// Center-aligned text
    Center,
    /// Right-aligned text
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Left
    }
}

/// Sort order for table rows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order
    Ascending,
    /// Descending order
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

/// Represents a column in the table
#[derive(Debug, Clone)]
pub struct Column {
    /// The title of the column
    title: String,
    /// The width of the column
    width: usize,
    /// The alignment of text in the column
    alignment: Alignment,
}

impl Column {
    /// Create a new column with the specified title, width, and alignment.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the column
    /// * `width` - The width of the column in characters
    /// * `alignment` - The text alignment within the column
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::ui::components::controls::table::{Column, Alignment};
    ///
    /// let column = Column::new("Name", 20, Alignment::Left);
    /// ```
    #[must_use]
    pub fn new(title: &str, width: usize, alignment: Alignment) -> Self {
        Self {
            title: title.to_string(),
            width,
            alignment,
        }
    }

    /// Get the title of the column
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the width of the column
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Get the alignment of the column
    #[must_use]
    pub const fn alignment(&self) -> Alignment {
        self.alignment
    }
}

/// Represents a row in the table
#[derive(Debug, Clone)]
pub struct Row {
    /// The cells in the row
    cells: Vec<String>,
}

impl Row {
    /// Create a new row from a vector of string slices
    ///
    /// # Arguments
    ///
    /// * `cells` - The cell values for the row
    #[must_use]
    pub fn new(cells: Vec<&str>) -> Self {
        Self {
            cells: cells.iter().map(ToString::to_string).collect(),
        }
    }

    /// Convert the row to a vector of strings
    #[must_use]
    pub fn to_vec(&self) -> Vec<String> {
        self.cells.clone()
    }

    /// Get a reference to the cells in the row
    #[must_use]
    pub fn cells(&self) -> &[String] {
        &self.cells
    }
}

/// Represents the style of a table
#[derive(Debug, Clone)]
pub struct TableStyle {
    /// The border color
    border_color: CrosstermColor,
    /// The header background color
    header_bg_color: CrosstermColor,
    /// The header foreground color
    header_fg_color: CrosstermColor,
    /// The selected row background color
    selected_bg_color: CrosstermColor,
    /// The selected row foreground color
    selected_fg_color: CrosstermColor,
    /// The cell background color
    cell_bg_color: CrosstermColor,
    /// The cell foreground color
    cell_fg_color: CrosstermColor,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            border_color: CrosstermColor::White,
            header_bg_color: CrosstermColor::Blue,
            header_fg_color: CrosstermColor::White,
            selected_bg_color: CrosstermColor::DarkBlue,
            selected_fg_color: CrosstermColor::White,
            cell_bg_color: CrosstermColor::Black,
            cell_fg_color: CrosstermColor::White,
        }
    }
}

impl TableStyle {
    /// Create a new table style with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert a UI color to a Crossterm color
    const fn get_crossterm_color(color: UiColor) -> CrosstermColor {
        match color {
            UiColor::Black => CrosstermColor::Black,
            UiColor::Red => CrosstermColor::Red,
            UiColor::Green => CrosstermColor::Green,
            UiColor::Yellow => CrosstermColor::Yellow,
            UiColor::Blue => CrosstermColor::Blue,
            UiColor::Magenta => CrosstermColor::Magenta,
            UiColor::Cyan => CrosstermColor::Cyan,
            UiColor::White => CrosstermColor::White,
            UiColor::LightBlack => CrosstermColor::DarkGrey,
            UiColor::LightRed => CrosstermColor::Red,
            UiColor::LightGreen => CrosstermColor::Green,
            UiColor::LightYellow => CrosstermColor::Yellow,
            UiColor::LightBlue => CrosstermColor::Blue,
            UiColor::LightMagenta => CrosstermColor::Magenta,
            UiColor::LightCyan => CrosstermColor::Cyan,
            UiColor::LightWhite => CrosstermColor::White,
            UiColor::Rgb(r, g, b) => CrosstermColor::Rgb { r, g, b },
            UiColor::Reset => CrosstermColor::Reset,
        }
    }

    /// Set the border color
    pub fn set_border_color(&mut self, color: UiColor) {
        self.border_color = Self::get_crossterm_color(color);
    }

    /// Set the header colors
    pub fn set_header_colors(&mut self, fg: UiColor, bg: UiColor) {
        self.header_fg_color = Self::get_crossterm_color(fg);
        self.header_bg_color = Self::get_crossterm_color(bg);
    }

    /// Set the selected row colors
    pub fn set_selected_colors(&mut self, fg: UiColor, bg: UiColor) {
        self.selected_fg_color = Self::get_crossterm_color(fg);
        self.selected_bg_color = Self::get_crossterm_color(bg);
    }

    /// Set the cell colors
    pub fn set_cell_colors(&mut self, fg: UiColor, bg: UiColor) {
        self.cell_fg_color = Self::get_crossterm_color(fg);
        self.cell_bg_color = Self::get_crossterm_color(bg);
    }
}

/// Represents a table with columns, rows, and styling
#[derive(Debug)]
pub struct Table {
    /// The columns in the table
    columns: Vec<Column>,
    /// The rows in the table
    rows: Vec<Row>,
    /// The style of the table
    style: TableStyle,
    /// The currently selected row index
    selected_row: Option<usize>,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            style: TableStyle::default(),
            selected_row: None,
        }
    }
}

impl Table {
    /// Create a new empty table
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column to the table
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Add a row to the table
    ///
    /// # Errors
    ///
    /// Returns `TableError::InvalidRow` if the number of cells doesn't match
    /// the number of columns.
    pub fn add_row(&mut self, cells: Vec<&str>) -> Result<(), TableError> {
        if cells.len() != self.columns.len() {
            return Err(TableError::InvalidRow(format!(
                "Expected {} cells, got {}",
                self.columns.len(),
                cells.len()
            )));
        }
        self.rows.push(Row::new(cells));
        Ok(())
    }

    /// Set the style of the table
    pub fn set_style(&mut self, style: TableStyle) {
        self.style = style;
    }

    /// Render the table to a writer
    ///
    /// # Errors
    ///
    /// Returns `TableError::IoError` if writing to the output fails.
    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), TableError> {
        self.render_header(writer)?;
        for (i, row) in self.rows.iter().enumerate() {
            self.render_row(writer, row, Some(i) == self.selected_row)?;
        }
        Ok(())
    }

    /// Render the table header
    ///
    /// # Errors
    ///
    /// Returns `TableError::IoError` if writing to the output fails.
    fn render_header<W: Write>(&self, writer: &mut W) -> Result<(), TableError> {
        execute!(
            writer,
            SetForegroundColor(self.style.header_fg_color),
        )?;

        for column in &self.columns {
            let title = format!("{:width$}", column.title(), width = column.width());
            write!(writer, "│ {} ", title)?;
        }
        writeln!(writer, "│")?;

        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Render a table row
    ///
    /// # Errors
    ///
    /// Returns `TableError::IoError` if writing to the output fails.
    fn render_row<W: Write>(&self, writer: &mut W, row: &Row, is_selected: bool) -> Result<(), TableError> {
        let color = if is_selected {
            self.style.selected_fg_color
        } else {
            self.style.cell_fg_color
        };

        execute!(writer, SetForegroundColor(color))?;

        for (cell, column) in row.cells().iter().zip(self.columns.iter()) {
            let formatted = match column.alignment() {
                Alignment::Left => format!("{:<width$}", cell, width = column.width()),
                Alignment::Center => format!("{:^width$}", cell, width = column.width()),
                Alignment::Right => format!("{:>width$}", cell, width = column.width()),
            };
            write!(writer, "│ {} ", formatted)?;
        }
        writeln!(writer, "│")?;

        execute!(writer, ResetColor)?;
        Ok(())
    }

    /// Handle a key event for table navigation
    ///
    /// Returns true if the event was handled, false otherwise.
    pub fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Up => {
                if let Some(selected) = self.selected_row {
                    if selected > 0 {
                        self.selected_row = Some(selected - 1);
                        return true;
                    }
                } else if !self.rows.is_empty() {
                    self.selected_row = Some(0);
                    return true;
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.selected_row {
                    if selected < self.rows.len() - 1 {
                        self.selected_row = Some(selected + 1);
                        return true;
                    }
                } else if !self.rows.is_empty() {
                    self.selected_row = Some(0);
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Get the currently selected row
    #[must_use]
    pub fn selected_row(&self) -> Option<&Row> {
        self.selected_row.map(|i| &self.rows[i])
    }

    /// Set the selected row by index
    pub fn set_selected_row(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.rows.len() {
                self.selected_row = Some(idx);
                return;
            }
        }
        self.selected_row = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_column(Column::new("Age", 5, Alignment::Right));

        assert!(table.add_row(vec!["John Doe", "30"]).is_ok());
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.columns.len(), 2);
    }

    #[test]
    fn test_invalid_row() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));

        let result = table.add_row(vec!["John Doe", "30"]);
        assert!(result.is_err());
        if let Err(TableError::InvalidRow(msg)) = result {
            assert!(msg.contains("Expected 1 cells, got 2"));
        }
    }

    #[test]
    fn test_row_selection() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_row(vec!["John Doe"]).unwrap();
        table.add_row(vec!["Jane Doe"]).unwrap();

        assert_eq!(table.selected_row, None);
        table.set_selected_row(Some(0));
        assert_eq!(table.selected_row, Some(0));
        table.set_selected_row(Some(100)); // Invalid index
        assert_eq!(table.selected_row, None);
    }
} 