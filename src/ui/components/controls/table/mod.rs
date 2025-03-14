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
use crossterm::event::KeyEvent;
use ratatui::style::Color as RatatuiColor;

/// Error type for table operations
#[derive(Debug)]
pub enum TableError {
    /// IO error occurred
    IoError(io::Error),
    /// Invalid row data
    InvalidRow(String),
    /// Invalid column data
    InvalidColumn(String),
    /// Invalid style data
    InvalidStyle(String),
    /// Invalid table reference
    InvalidTable,
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::InvalidRow(e) => write!(f, "Invalid row: {}", e),
            Self::InvalidColumn(e) => write!(f, "Invalid column: {}", e),
            Self::InvalidStyle(e) => write!(f, "Invalid style: {}", e),
            Self::InvalidTable => write!(f, "Invalid table reference"),
        }
    }
}

impl StdError for TableError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for TableError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
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

/// Sort order for table rows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order
    Ascending,
    /// Descending order
    Descending,
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
    /// Create a new column
    pub fn new(title: &str, width: usize, alignment: Alignment) -> Self {
        Self {
            title: title.to_string(),
            width,
            alignment,
        }
    }
}

/// Represents a row in the table
#[derive(Debug, Clone)]
pub struct Row {
    /// The cells in the row
    cells: Vec<String>,
}

impl Row {
    /// Create a new row
    pub fn new(cells: Vec<&str>) -> Self {
        Self {
            cells: cells.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Convert the row to a vector of strings
    pub fn to_vec(&self) -> Vec<String> {
        self.cells.clone()
    }

    /// Get the cells in the row
    pub fn get_cells(&self) -> &Vec<String> {
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the Crossterm color for a UI color
    fn get_crossterm_color(&self, color: UiColor) -> CrosstermColor {
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
    /// Create a new table
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column to the table
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Add a row to the table
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
    pub fn render<W: Write>(&self, writer: &mut W) -> Result<(), TableError> {
        // Render header
        self.render_header(writer)?;

        // Render rows
        for (i, row) in self.rows.iter().enumerate() {
            self.render_row(writer, row, Some(i) == self.selected_row)?;
        }

        Ok(())
    }

    /// Render the header row
    fn render_header<W: Write>(&self, writer: &mut W) -> Result<(), TableError> {
        // Set header colors
        writer.queue(SetForegroundColor(self.style.header_fg_color))?;
        writer.queue(SetBackgroundColor(self.style.header_bg_color))?;

        // Render column titles
        for (i, column) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(writer, "│")?;
            }
            match column.alignment {
                Alignment::Left => write!(writer, " {:<width$}", column.title, width = column.width)?,
                Alignment::Center => write!(writer, " {:^width$}", column.title, width = column.width)?,
                Alignment::Right => write!(writer, " {:>width$}", column.title, width = column.width)?,
            }
        }

        // Reset colors
        writer.queue(ResetColor)?;
        writer.write_all(b"\n")?;

        // Render header separator
        writer.queue(SetForegroundColor(self.style.border_color))?;
        for (i, column) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(writer, "┼")?;
            }
            write!(writer, "{}", "─".repeat(column.width + 2))?;
        }
        writer.queue(ResetColor)?;
        writer.write_all(b"\n")?;

        Ok(())
    }

    /// Render a row
    fn render_row<W: Write>(&self, writer: &mut W, row: &Row, is_selected: bool) -> Result<(), TableError> {
        let (fg_color, bg_color) = if is_selected {
            (self.style.selected_fg_color, self.style.selected_bg_color)
        } else {
            (self.style.cell_fg_color, self.style.cell_bg_color)
        };

        writer.queue(SetForegroundColor(fg_color))?;
        writer.queue(SetBackgroundColor(bg_color))?;

        for (i, (cell, column)) in row.cells.iter().zip(self.columns.iter()).enumerate() {
            if i > 0 {
                writer.queue(SetForegroundColor(self.style.border_color))?;
                write!(writer, "│")?;
                writer.queue(SetForegroundColor(fg_color))?;
            }
            match column.alignment {
                Alignment::Left => write!(writer, " {:<width$}", cell, width = column.width)?,
                Alignment::Center => write!(writer, " {:^width$}", cell, width = column.width)?,
                Alignment::Right => write!(writer, " {:>width$}", cell, width = column.width)?,
            }
        }

        writer.queue(ResetColor)?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    /// Handle keyboard events
    pub fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        match event.code {
            KeyCode::Up => {
                if let Some(current) = self.selected_row {
                    if current > 0 {
                        self.selected_row = Some(current - 1);
                        return true;
                    }
                }
                false
            }
            KeyCode::Down => {
                if let Some(current) = self.selected_row {
                    if current < self.rows.len() - 1 {
                        self.selected_row = Some(current + 1);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Get the currently selected row
    pub fn get_selected_row(&self) -> Option<&Row> {
        self.selected_row.and_then(|i| self.rows.get(i))
    }

    /// Set the selected row
    pub fn set_selected_row(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.rows.len() {
                self.selected_row = Some(i);
            }
        } else {
            self.selected_row = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_table_creation() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_column(Column::new("Age", 10, Alignment::Right));
        
        assert!(table.add_row(vec!["John Doe", "30"]).is_ok());
        assert!(table.add_row(vec!["Jane Smith", "25"]).is_ok());
        
        let mut buffer = Cursor::new(Vec::new());
        assert!(table.render(&mut buffer).is_ok());
    }

    #[test]
    fn test_invalid_row() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_column(Column::new("Age", 10, Alignment::Right));
        
        assert!(matches!(
            table.add_row(vec!["John Doe"]),
            Err(TableError::InvalidRow(_))
        ));
    }

    #[test]
    fn test_row_selection() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_row(vec!["John Doe"]).unwrap();
        
        table.set_selected_row(Some(0));
        assert_eq!(table.get_selected_row().unwrap().get_cells()[0], "John Doe");
        
        table.set_selected_row(None);
        assert!(table.get_selected_row().is_none());
    }
} 