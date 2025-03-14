use std::collections::HashMap;
use crate::ui::layout::{LayoutError, Position, Rect, Size, Spacing};

/// Configuration for a grid layout.
/// 
/// Defines the structure of a grid with rows, columns, and spacing options.
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Number of rows in the grid.
    pub rows: u16,
    /// Number of columns in the grid.
    pub columns: u16,
    /// Spacing between grid cells.
    pub spacing: Spacing,
    /// Margin around the entire grid.
    pub margin: Spacing,
}

impl GridConfig {
    /// Creates a new grid configuration with the specified number of rows and columns.
    /// 
    /// # Arguments
    /// * `rows` - Number of rows in the grid
    /// * `columns` - Number of columns in the grid
    #[must_use]
    pub fn new(rows: u16, columns: u16) -> Self {
        Self {
            rows,
            columns,
            spacing: Spacing::uniform(0),
            margin: Spacing::uniform(0),
        }
    }

    /// Sets the spacing between grid cells.
    /// 
    /// # Arguments
    /// * `spacing` - The spacing to apply between cells
    #[must_use]
    pub fn with_spacing(mut self, spacing: Spacing) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the margin around the entire grid.
    /// 
    /// # Arguments
    /// * `margin` - The margin to apply around the grid
    #[must_use]
    pub fn with_margin(mut self, margin: Spacing) -> Self {
        self.margin = margin;
        self
    }
}

/// Represents a cell in the grid layout.
/// 
/// A cell defines a rectangular area within the grid and can optionally span
/// multiple rows and columns.
#[derive(Debug, Clone)]
pub struct GridCell {
    /// The rectangular area occupied by the cell.
    pub rect: Rect,
    /// Number of rows this cell spans.
    pub row_span: u16,
    /// Number of columns this cell spans.
    pub col_span: u16,
}

impl GridCell {
    /// Creates a new grid cell with the specified rectangular area.
    /// 
    /// # Arguments
    /// * `rect` - The rectangular area for the cell
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            row_span: 1,
            col_span: 1,
        }
    }

    /// Creates a new grid cell with the specified area and span.
    /// 
    /// # Arguments
    /// * `rect` - The rectangular area for the cell
    /// * `row_span` - Number of rows this cell spans
    /// * `col_span` - Number of columns this cell spans
    #[must_use]
    pub fn with_span(rect: Rect, row_span: u16, col_span: u16) -> Self {
        Self {
            rect,
            row_span,
            col_span,
        }
    }
}

/// Grid layout manager that arranges cells in a grid pattern.
/// 
/// Handles the calculation and arrangement of cells within a grid layout,
/// taking into account spacing, margins, and cell spans.
#[derive(Debug, Clone)]
pub struct Grid {
    /// The configuration settings for the grid layout.
    config: GridConfig,
    /// The map of grid cell positions to their corresponding cell data.
    cells: HashMap<(u16, u16), GridCell>,
    /// A 2D vector tracking which grid positions are occupied.
    occupied: Vec<Vec<bool>>,
}

impl Grid {
    /// Creates a new grid with the specified configuration.
    /// 
    /// # Arguments
    /// * `config` - The configuration for the grid layout
    #[must_use]
    pub fn new(config: GridConfig) -> Self {
        let mut occupied = Vec::with_capacity(config.rows as usize);
        for _ in 0..config.rows {
            occupied.push(vec![false; config.columns as usize]);
        }

        Self {
            config,
            cells: HashMap::new(),
            occupied,
        }
    }

    /// Adds a cell to the grid at the specified position
    pub fn add_cell(&mut self, row: u16, col: u16, cell: GridCell) -> Result<(), LayoutError> {
        // Validate position
        if row >= self.config.rows || col >= self.config.columns {
            return Err(LayoutError::InvalidGrid {
                message: format!(
                    "Cell position ({}, {}) is outside grid bounds ({}, {})",
                    row, col, self.config.rows, self.config.columns
                ),
            });
        }

        // Validate span
        if row + cell.row_span > self.config.rows || col + cell.col_span > self.config.columns {
            return Err(LayoutError::InvalidGrid {
                message: format!(
                    "Cell span ({}, {}) at ({}, {}) exceeds grid bounds",
                    cell.row_span, cell.col_span, row, col
                ),
            });
        }

        // Check if cells are available
        for r in row..row + cell.row_span {
            for c in col..col + cell.col_span {
                if self.occupied[r as usize][c as usize] {
                    return Err(LayoutError::InvalidGrid {
                        message: format!("Cell at ({}, {}) is already occupied", r, c),
                    });
                }
            }
        }

        // Mark cells as occupied
        for r in row..row + cell.row_span {
            for c in col..col + cell.col_span {
                self.occupied[r as usize][c as usize] = true;
            }
        }

        self.cells.insert((row, col), cell);
        Ok(())
    }

    /// Calculates the actual size and position of each cell based on the available space
    pub fn calculate_layout(&mut self, available_space: Size) -> Result<(), LayoutError> {
        let inner_space = Size::new(
            available_space.width.saturating_sub(self.config.margin.horizontal()),
            available_space.height.saturating_sub(self.config.margin.vertical()),
        );

        let total_spacing_width = self.config.spacing.horizontal() * (self.config.columns - 1);
        let total_spacing_height = self.config.spacing.vertical() * (self.config.rows - 1);

        let cell_width = (inner_space.width.saturating_sub(total_spacing_width)) / self.config.columns;
        let cell_height = (inner_space.height.saturating_sub(total_spacing_height)) / self.config.rows;

        // Update cell positions and sizes
        for ((row, col), cell) in self.cells.iter_mut() {
            let x = self.config.margin.left + col * (cell_width + self.config.spacing.horizontal());
            let y = self.config.margin.top + row * (cell_height + self.config.spacing.vertical());

            let width = cell_width * cell.col_span + self.config.spacing.horizontal() * (cell.col_span - 1);
            let height = cell_height * cell.row_span + self.config.spacing.vertical() * (cell.row_span - 1);

            cell.rect = Rect::new(Position::new(x, y), Size::new(width, height));
        }

        Ok(())
    }

    /// Gets a reference to a cell at the specified position
    pub fn get_cell(&self, row: u16, col: u16) -> Option<&GridCell> {
        self.cells.get(&(row, col))
    }

    /// Gets a mutable reference to a cell at the specified position
    pub fn get_cell_mut(&mut self, row: u16, col: u16) -> Option<&mut GridCell> {
        self.cells.get_mut(&(row, col))
    }

    /// Removes a cell from the grid
    pub fn remove_cell(&mut self, row: u16, col: u16) -> Option<GridCell> {
        if let Some(cell) = self.cells.remove(&(row, col)) {
            // Clear occupied cells
            for r in row..row + cell.row_span {
                for c in col..col + cell.col_span {
                    if r < self.config.rows && c < self.config.columns {
                        self.occupied[r as usize][c as usize] = false;
                    }
                }
            }
            Some(cell)
        } else {
            None
        }
    }

    /// Clears all cells from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
        for row in self.occupied.iter_mut() {
            row.fill(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let config = GridConfig::new(3, 3);
        let grid = Grid::new(config);
        assert_eq!(grid.cells.len(), 0);
        assert_eq!(grid.occupied.len(), 3);
        assert_eq!(grid.occupied[0].len(), 3);
    }

    #[test]
    fn test_add_cell() {
        let config = GridConfig::new(3, 3);
        let mut grid = Grid::new(config);
        let cell = GridCell::new(Rect::new(Position::new(0, 0), Size::new(10, 10)));
        assert!(grid.add_cell(0, 0, cell).is_ok());
    }

    #[test]
    fn test_add_cell_with_span() {
        let config = GridConfig::new(3, 3);
        let mut grid = Grid::new(config);
        let cell = GridCell::with_span(
            Rect::new(Position::new(0, 0), Size::new(20, 20)),
            2,
            2,
        );
        assert!(grid.add_cell(0, 0, cell).is_ok());
        assert!(grid.get_cell(0, 0).is_some());
        assert!(grid.add_cell(0, 1, GridCell::new(Rect::new(Position::new(0, 0), Size::new(10, 10)))).is_err());
    }

    #[test]
    fn test_calculate_layout() {
        let config = GridConfig::new(2, 2)
            .with_spacing(Spacing::uniform(5))
            .with_margin(Spacing::uniform(10));
        let mut grid = Grid::new(config);

        // Add cells
        let cell1 = GridCell::new(Rect::new(Position::new(0, 0), Size::new(0, 0)));
        let cell2 = GridCell::new(Rect::new(Position::new(0, 0), Size::new(0, 0)));
        let cell3 = GridCell::new(Rect::new(Position::new(0, 0), Size::new(0, 0)));
        let cell4 = GridCell::new(Rect::new(Position::new(0, 0), Size::new(0, 0)));

        assert!(grid.add_cell(0, 0, cell1).is_ok());
        assert!(grid.add_cell(0, 1, cell2).is_ok());
        assert!(grid.add_cell(1, 0, cell3).is_ok());
        assert!(grid.add_cell(1, 1, cell4).is_ok());

        // Calculate layout
        assert!(grid.calculate_layout(Size::new(100, 100)).is_ok());

        // Verify cell positions and sizes
        let cell = grid.get_cell(0, 0).unwrap();
        assert_eq!(cell.rect.position.x, 10); // margin.left
        assert_eq!(cell.rect.position.y, 10); // margin.top
        assert_eq!(cell.rect.size.width, 35); // (100 - 2*10 - 5) / 2 = 37.5
        assert_eq!(cell.rect.size.height, 35);
    }
} 