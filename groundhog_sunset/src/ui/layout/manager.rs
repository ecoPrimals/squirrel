/// Module for managing layout indentation and spacing in the terminal UI.

use std::io::Write;
use std::collections::HashMap;
use crate::ui::{
    Size,
    Rect,
    layout::{
        LayoutError,
        constraint::ConstraintSystem,
        grid::Grid,
        cache::LayoutCache,
    },
};
use crate::ui::layout::{
    Position,
    cache::{LayoutCacheKey},
    constraint::{Constraint},
    grid::{GridConfig},
};
use std::fmt;

/// A function that calculates layout sizes based on constraints.
type LayoutFn = Box<dyn Fn(Size, &[u8]) -> Result<Size, LayoutError> + Send + Sync>;

/// Manages indentation and layout spacing for terminal output.
/// 
/// Provides functionality to control indentation levels and write indented content
/// to the terminal output.
pub struct LayoutManager {
    /// Current indentation level.
    current_indentation: usize,
    /// Size of each indentation level in spaces.
    indent_size: usize,
    /// Maximum recursion depth for layout calculations
    max_recursion_depth: usize,
    /// Current recursion depth
    current_depth: usize,
    /// The cache for storing and retrieving layout calculations.
    cache: LayoutCache,
    /// The system for managing layout constraints.
    constraint_system: ConstraintSystem,
    /// The optional grid layout system.
    grid: Option<Grid>,
    /// A map of layout functions that can be used to calculate sizes.
    layouts: HashMap<String, LayoutFn>,
}

impl fmt::Debug for LayoutManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayoutManager")
            .field("current_indentation", &self.current_indentation)
            .field("indent_size", &self.indent_size)
            .field("max_recursion_depth", &self.max_recursion_depth)
            .field("current_depth", &self.current_depth)
            .field("cache", &self.cache)
            .field("layouts", &format!("<{} layout functions>", self.layouts.len()))
            .finish()
    }
}

impl LayoutManager {
    /// Creates a new layout manager with default settings.
    /// 
    /// Default indentation size is 4 spaces.
    pub fn new() -> Self {
        Self {
            current_indentation: 0,
            indent_size: 4,
            max_recursion_depth: 100,
            current_depth: 0,
            cache: LayoutCache::new(),
            constraint_system: ConstraintSystem::new(),
            grid: None,
            layouts: HashMap::new(),
        }
    }

    /// Creates a new layout manager with a custom indentation size.
    /// 
    /// # Arguments
    /// * `indent_size` - Number of spaces for each indentation level
    /// 
    /// # Returns
    /// * `Ok(Self)` - A new layout manager with the specified indentation size
    /// * `Err(LayoutError)` - If the indentation size is invalid (zero)
    pub fn with_indentation_size(indent_size: usize) -> Result<Self, LayoutError> {
        if indent_size == 0 {
            return Err(LayoutError::InvalidIndentSize { size: indent_size });
        }

        Ok(Self {
            current_indentation: 0,
            indent_size,
            max_recursion_depth: 100,
            current_depth: 0,
            cache: LayoutCache::new(),
            constraint_system: ConstraintSystem::new(),
            grid: None,
            layouts: HashMap::new(),
        })
    }

    /// Sets a new indentation size.
    /// 
    /// # Arguments
    /// * `size` - Number of spaces for each indentation level
    /// 
    /// # Returns
    /// * `Ok(())` - If the indentation size was successfully set
    /// * `Err(LayoutError)` - If the indentation size is invalid (zero)
    pub fn set_indentation_size(&mut self, size: usize) -> Result<(), LayoutError> {
        if size == 0 {
            return Err(LayoutError::InvalidIndentSize { size });
        }
        self.indent_size = size;
        Ok(())
    }

    /// Increases the current indentation level by one.
    pub fn increase_indentation(&mut self) -> Result<(), LayoutError> {
        let new_indentation = self.current_indentation.checked_add(self.indent_size)
            .ok_or_else(|| LayoutError::NegativeIndentation)?;
        self.current_indentation = new_indentation;
        Ok(())
    }

    /// Decreases the current indentation level by one.
    /// 
    /// # Returns
    /// * `Ok(())` - If the indentation was successfully decreased
    /// * `Err(LayoutError)` - If the current indentation is already zero
    pub fn decrease_indentation(&mut self) -> Result<(), LayoutError> {
        if self.current_indentation < self.indent_size {
            return Err(LayoutError::NegativeIndentation);
        }
        self.current_indentation -= self.indent_size;
        Ok(())
    }

    /// Resets the indentation level to zero.
    pub fn reset(&mut self) {
        self.current_indentation = 0;
        self.current_depth = 0;
        self.cache.clear();
        self.grid = None;
        self.layouts.clear();
    }

    /// Returns the current indentation level.
    pub fn get_current_indentation(&self) -> usize {
        self.current_indentation
    }

    /// Returns the size of each indentation level in spaces.
    pub fn get_indent_size(&self) -> usize {
        self.indent_size
    }

    /// Writes the current indentation to the specified writer.
    /// 
    /// # Arguments
    /// * `writer` - The writer to output the indentation to
    /// 
    /// # Returns
    /// * `Ok(())` - If the indentation was successfully written
    /// * `Err(std::io::Error)` - If an error occurred while writing
    pub fn write_indentation<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write!(writer, "{:width$}", "", width = self.current_indentation)
    }

    // New methods for enhanced layout management

    /// Sets up a grid layout with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the grid layout
    pub fn setup_grid(&mut self, config: GridConfig) {
        self.grid = Some(Grid::new(config));
    }

    /// Clears the current grid layout.
    ///
    /// This method removes the current grid layout, allowing the layout manager
    /// to be used without a grid.
    pub fn clear_grid(&mut self) {
        self.grid = None;
    }

    /// Gets a reference to the current grid layout.
    ///
    /// # Returns
    ///
    /// Returns `Some(&Grid)` if a grid layout is active, or `None` if no grid
    /// is currently set up.
    pub fn get_grid(&self) -> Option<&Grid> {
        self.grid.as_ref()
    }

    /// Gets a mutable reference to the current grid layout.
    ///
    /// # Returns
    ///
    /// Returns `Some(&mut Grid)` if a grid layout is active, or `None` if no grid
    /// is currently set up.
    pub fn get_grid_mut(&mut self) -> Option<&mut Grid> {
        self.grid.as_mut()
    }

    /// Adds a horizontal constraint to the constraint system.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The constraint to add
    pub fn add_horizontal_constraint(&mut self, constraint: Constraint) {
        self.constraint_system.add_horizontal(constraint);
    }

    /// Adds a vertical constraint to the constraint system.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The constraint to add
    pub fn add_vertical_constraint(&mut self, constraint: Constraint) {
        self.constraint_system.add_vertical(constraint);
    }

    /// Calculates layout with recursion depth check.
    ///
    /// This method calculates the layout for a component while ensuring that the
    /// maximum recursion depth is not exceeded.
    ///
    /// # Arguments
    ///
    /// * `component_id` - The unique identifier for the component
    /// * `size` - The available size for the layout
    /// * `constraints` - Additional constraints for the layout
    ///
    /// # Returns
    ///
    /// Returns `Ok(Size)` with the calculated size, or a `LayoutError` if the
    /// layout cannot be calculated or the maximum recursion depth is exceeded.
    pub fn calculate_layout(&mut self, component_id: &str, size: Size, constraints: &[u8]) -> Result<Size, LayoutError> {
        let key = LayoutCacheKey::new(component_id, size, constraints);
        
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.size);
        }

        self.current_depth += 1;
        if self.current_depth > self.max_recursion_depth {
            self.current_depth -= 1;
            return Err(LayoutError::MaxRecursionDepthExceeded);
        }

        let layout_constraints = vec![
            Constraint::MaxWidth(size.width),
            Constraint::MaxHeight(size.height),
        ];

        let rect = self.constraint_system.calculate_rect(size, &layout_constraints)?;
        self.current_depth -= 1;

        let rect = Rect::new(Position::new(0, 0), rect.size);
        self.cache.insert(key, rect)?;

        Ok(rect.size)
    }

    /// Gets or calculates layout with recursion depth check.
    ///
    /// This method first checks the cache for a previously calculated layout,
    /// and if not found, calculates a new layout while ensuring that the maximum
    /// recursion depth is not exceeded.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key for the layout
    ///
    /// # Returns
    ///
    /// Returns `Ok(Size)` with the calculated size, or a `LayoutError` if the
    /// layout cannot be calculated or the maximum recursion depth is exceeded.
    pub fn get_or_calculate_layout(&mut self, key: LayoutCacheKey) -> Result<Size, LayoutError> {
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.size);
        }

        self.current_depth += 1;
        if self.current_depth > self.max_recursion_depth {
            self.current_depth -= 1;
            return Err(LayoutError::MaxRecursionDepthExceeded);
        }

        let result = self.calculate_layout(&key.component_id, key.size, &key.constraints)?;
        self.current_depth -= 1;

        let rect = Rect::new(Position::new(0, 0), result);
        self.cache.insert(key, rect)?;

        Ok(result)
    }

    /// Clears the layout cache.
    ///
    /// This method removes all cached layouts, forcing new calculations for
    /// subsequent layout requests.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Registers a custom layout function for a specific component.
    /// 
    /// This method allows registering a custom layout calculation function for a component.
    /// The layout function takes the available size and constraints as input and returns
    /// the calculated size for the component.
    /// 
    /// # Arguments
    /// 
    /// * `component_id` - The unique identifier for the component
    /// * `layout_fn` - The function that calculates the layout for the component
    /// 
    /// # Examples
    /// 
    /// ```
    /// use groundhog_mcp::ui::layout::{LayoutManager, Size, LayoutError};
    /// 
    /// let mut manager = LayoutManager::new();
    /// manager.register_layout("custom_component", |size, constraints| {
    ///     Ok(Size::new(size.width / 2, size.height / 2))
    /// });
    /// ```
    pub fn register_layout<F>(&mut self, component_id: impl Into<String>, layout_fn: F)
    where
        F: Fn(Size, &[u8]) -> Result<Size, LayoutError> + Send + Sync + 'static,
    {
        self.layouts.insert(component_id.into(), Box::new(layout_fn));
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layout::Spacing;

    #[test]
    fn test_indentation() {
        let mut layout = LayoutManager::new();
        layout.set_indentation_size(2).unwrap();
        layout.increase_indentation().unwrap();
        assert_eq!(layout.get_current_indentation(), 2);
        layout.reset();
        assert_eq!(layout.get_current_indentation(), 0);
    }

    #[test]
    fn test_multiple_indents() {
        let mut layout = LayoutManager::new();
        layout.set_indentation_size(2).unwrap();
        layout.increase_indentation().unwrap();
        layout.increase_indentation().unwrap();
        assert_eq!(layout.get_current_indentation(), 4);
        layout.decrease_indentation().unwrap();
        assert_eq!(layout.get_current_indentation(), 2);
    }

    #[test]
    fn test_negative_indent() {
        let mut layout = LayoutManager::new();
        layout.set_indentation_size(2).unwrap();
        assert!(matches!(
            layout.decrease_indentation(),
            Err(LayoutError::NegativeIndentation)
        ));
        assert_eq!(layout.get_current_indentation(), 0);
    }

    #[test]
    fn test_invalid_indent_size() {
        assert!(matches!(
            LayoutManager::with_indentation_size(0),
            Err(LayoutError::InvalidIndentSize { size: 0 })
        ));
        let mut layout = LayoutManager::new();
        assert!(matches!(
            layout.set_indentation_size(0),
            Err(LayoutError::InvalidIndentSize { size: 0 })
        ));
    }

    #[test]
    fn test_write_indentation() {
        let mut layout = LayoutManager::new();
        layout.set_indentation_size(2).unwrap();
        layout.increase_indentation().unwrap();
        let mut buffer = std::io::Cursor::new(Vec::new());
        layout.write_indentation(&mut buffer).unwrap();
        assert_eq!(String::from_utf8(buffer.into_inner()).unwrap(), "  ");
    }

    #[test]
    fn test_layout_cache() {
        let mut layout = LayoutManager::new();
        let size = Size::new(100, 100);
        let constraints = vec![1, 2, 3];

        // Add some constraints
        layout.add_horizontal_constraint(Constraint::Fixed(50));
        layout.add_vertical_constraint(Constraint::Fixed(50));

        // First calculation should compute and cache
        let rect1 = layout.get_or_calculate_layout(LayoutCacheKey::new("test", size, &constraints)).unwrap();

        // Second calculation should use cache
        let rect2 = layout.get_or_calculate_layout(LayoutCacheKey::new("test", size, &constraints)).unwrap();

        assert_eq!(rect1, rect2);
    }

    #[test]
    fn test_grid_layout() {
        let mut layout = LayoutManager::new();
        let config = GridConfig::new(2, 2)
            .with_spacing(Spacing::uniform(5))
            .with_margin(Spacing::uniform(10));

        layout.setup_grid(config);
        assert!(layout.get_grid().is_some());

        layout.clear_grid();
        assert!(layout.get_grid().is_none());
    }
} 