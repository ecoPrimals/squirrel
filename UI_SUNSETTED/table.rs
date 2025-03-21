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

        // Render bottom border
        self.render_horizontal_border(writer)?;

        Ok(())
    }

    /// Render the header of the table
    fn render_header<W: Write>(&self, writer: &mut W) -> Result<(), TableError> {
        // Render top border
        self.render_horizontal_border(writer)?;

        // Render header cells
        for (i, column) in self.columns.iter().enumerate() {
            write!(writer, "│")?;
            let padded_title = self.pad_cell(&column.title, column.width, column.alignment);
            write!(
                writer,
                " {} ",
                crossterm::style::style(padded_title)
                    .with(self.style.header_fg_color)
                    .on(self.style.header_bg_color)
            )?;
        }
        writeln!(writer, "│")?;

        // Render separator
        self.render_horizontal_border(writer)?;

        Ok(())
    }

    /// Render a row of the table
    fn render_row<W: Write>(&self, writer: &mut W, row: &Row, is_selected: bool) -> Result<(), TableError> {
        for (i, cell) in row.cells.iter().enumerate() {
            write!(writer, "│")?;
            let padded_cell = self.pad_cell(cell, self.columns[i].width, self.columns[i].alignment);
            let style = if is_selected {
                crossterm::style::style(padded_cell)
                    .with(self.style.selected_fg_color)
                    .on(self.style.selected_bg_color)
            } else {
                crossterm::style::style(padded_cell)
                    .with(self.style.cell_fg_color)
                    .on(self.style.cell_bg_color)
            };
            write!(writer, " {} ", style)?;
        }
        writeln!(writer, "│")?;
        Ok(())
    }

    /// Render a horizontal border of the table
    fn render_horizontal_border<W: Write>(&self, writer: &mut W) -> Result<(), TableError> {
        write!(writer, "├")?;
        for (i, column) in self.columns.iter().enumerate() {
            for _ in 0..column.width + 2 {
                write!(
                    writer,
                    "{}",
                    crossterm::style::style("─")
                        .with(self.style.border_color)
                )?;
            }
            if i < self.columns.len() - 1 {
                write!(writer, "┼")?;
            }
        }
        writeln!(writer, "┤")?;
        Ok(())
    }

    /// Pad a cell's content according to its alignment
    fn pad_cell(&self, text: &str, width: usize, alignment: Alignment) -> String {
        let text_width = text.chars().count();
        if text_width >= width {
            return text.to_string();
        }

        let padding = width - text_width;
        match alignment {
            Alignment::Left => format!("{}{}", text, " ".repeat(padding)),
            Alignment::Center => {
                let left_padding = padding / 2;
                let right_padding = padding - left_padding;
                format!(
                    "{}{}{}",
                    " ".repeat(left_padding),
                    text,
                    " ".repeat(right_padding)
                )
            }
            Alignment::Right => format!("{}{}", " ".repeat(padding), text),
        }
    }

    /// Handle a key event
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool, TableError> {
        match key.code {
            crossterm::event::KeyCode::Up => {
                if let Some(current) = self.selected_row {
                    if current > 0 {
                        self.selected_row = Some(current - 1);
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            crossterm::event::KeyCode::Down => {
                if let Some(current) = self.selected_row {
                    if current < self.rows.len() - 1 {
                        self.selected_row = Some(current + 1);
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            crossterm::event::KeyCode::Home => {
                if !self.rows.is_empty() {
                    self.selected_row = Some(0);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            crossterm::event::KeyCode::End => {
                if !self.rows.is_empty() {
                    self.selected_row = Some(self.rows.len() - 1);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    /// Get the currently selected row index
    pub fn get_selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    /// Get a row by index
    pub fn get_row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// Get the Crossterm color for a UI color
    fn get_crossterm_color(&self, color: RatatuiColor) -> CrosstermColor {
        match color {
            RatatuiColor::Black => CrosstermColor::Black,
            RatatuiColor::Red => CrosstermColor::Red,
            RatatuiColor::Green => CrosstermColor::Green,
            RatatuiColor::Yellow => CrosstermColor::Yellow,
            RatatuiColor::Blue => CrosstermColor::Blue,
            RatatuiColor::Magenta => CrosstermColor::Magenta,
            RatatuiColor::Cyan => CrosstermColor::Cyan,
            RatatuiColor::White => CrosstermColor::White,
            RatatuiColor::Gray => CrosstermColor::DarkGrey,
            RatatuiColor::LightRed => CrosstermColor::Red,
            RatatuiColor::LightGreen => CrosstermColor::Green,
            RatatuiColor::LightYellow => CrosstermColor::Yellow,
            RatatuiColor::LightBlue => CrosstermColor::Blue,
            RatatuiColor::LightMagenta => CrosstermColor::Magenta,
            RatatuiColor::LightCyan => CrosstermColor::Cyan,
            RatatuiColor::LightGray => CrosstermColor::White,
            RatatuiColor::Rgb(r, g, b) => CrosstermColor::Rgb { r, g, b },
            RatatuiColor::Reset => CrosstermColor::Reset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_column(Column::new("Age", 10, Alignment::Right));
        table.add_row(&["John Doe", "30"]).unwrap();
        table.add_row(&["Jane Smith", "25"]).unwrap();

        let mut output = Vec::new();
        table.render(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("John Doe"));
        assert!(output.contains("Jane Smith"));
    }

    #[test]
    fn test_row_selection() {
        let mut table = Table::new();
        table.add_column(Column::new("Name", 20, Alignment::Left));
        table.add_row(&["John Doe"]).unwrap();
        table.add_row(&["Jane Smith"]).unwrap();

        assert_eq!(table.get_selected_row(), None);
        assert!(table.handle_key(KeyEvent::new(crossterm::event::KeyCode::Down, crossterm::event::KeyModifiers::NONE)).unwrap());
        assert_eq!(table.get_selected_row(), Some(0));
        assert!(table.handle_key(KeyEvent::new(crossterm::event::KeyCode::Down, crossterm::event::KeyModifiers::NONE)).unwrap());
        assert_eq!(table.get_selected_row(), Some(1));
        assert!(!table.handle_key(KeyEvent::new(crossterm::event::KeyCode::Down, crossterm::event::KeyModifiers::NONE)).unwrap());
        assert_eq!(table.get_selected_row(), Some(1));
    }

    #[test]
    fn test_cell_padding() {
        let table = Table::new();
        assert_eq!(table.pad_cell("test", 10, Alignment::Left), "test      ");
        assert_eq!(table.pad_cell("test", 10, Alignment::Center), "   test   ");
        assert_eq!(table.pad_cell("test", 10, Alignment::Right), "      test");
    }
} 