use crate::ui::layout::{LayoutError, Size, Rect, Position};

/// Module for defining layout constraints that control how UI elements are sized and positioned.

/// Represents a size constraint for a UI element.
/// 
/// Constraints can be specified as a percentage, fixed length, or ratio of the available space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    /// Fixed number of units
    Fixed(u16),
    /// Percentage of parent size (0-100)
    Percentage(u8),
    /// Minimum number of units
    Min(u16),
    /// Maximum number of units
    Max(u16),
    /// Range between min and max units
    Range(u16, u16),
    /// Fill remaining space with optional weight
    Fill(u8),
    /// Minimum width
    MinWidth(u16),
    /// Maximum width
    MaxWidth(u16),
    /// Minimum height
    MinHeight(u16),
    /// Maximum height
    MaxHeight(u16),
}

impl Constraint {
    /// Applies the constraint to calculate the actual size given a parent size.
    ///
    /// # Arguments
    ///
    /// * `parent_size` - The size of the parent container
    ///
    /// # Returns
    ///
    /// Returns `Ok(u16)` with the calculated size, or a `LayoutError` if the constraint
    /// cannot be satisfied.
    pub fn apply(&self, parent_size: u16) -> Result<u16, LayoutError> {
        match *self {
            Constraint::Fixed(size) => Ok(size),
            Constraint::Percentage(percent) => {
                if percent > 100 {
                    return Err(LayoutError::InvalidConstraint {
                        message: format!("Percentage must be between 0 and 100, got {}", percent),
                    });
                }
                Ok((parent_size as f32 * (percent as f32 / 100.0)) as u16)
            }
            Constraint::Min(min) => Ok(min),
            Constraint::Max(max) => Ok(max.min(parent_size)),
            Constraint::Range(min, max) => {
                if min > max {
                    return Err(LayoutError::InvalidConstraint {
                        message: format!("Range min ({}) must be less than max ({})", min, max),
                    });
                }
                Ok(parent_size.clamp(min, max))
            }
            Constraint::Fill(weight) => Ok((parent_size as f32 * (weight as f32 / 100.0)) as u16),
            Constraint::MinWidth(min_width) => Ok(min_width),
            Constraint::MaxWidth(max_width) => Ok(max_width.min(parent_size)),
            Constraint::MinHeight(min_height) => Ok(min_height),
            Constraint::MaxHeight(max_height) => Ok(max_height.min(parent_size)),
        }
    }
}

/// Collection of constraints for both horizontal and vertical layout directions.
/// 
/// Used to define how UI elements should be sized and positioned within their container.
#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    /// The list of layout constraints to be applied.
    constraints: Vec<Constraint>,
}

impl ConstraintSystem {
    /// Creates a new empty set of constraints.
    ///
    /// # Returns
    ///
    /// Returns a new `ConstraintSystem` with no constraints.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Adds a horizontal constraint.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The constraint to add
    pub fn add_horizontal(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Adds a vertical constraint.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The constraint to add
    pub fn add_vertical(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Adds a constraint.
    ///
    /// # Arguments
    ///
    /// * `constraint` - The constraint to add
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Clears all constraints.
    ///
    /// This method removes all constraints from the system, allowing it to be reused
    /// with a new set of constraints.
    pub fn clear(&mut self) {
        self.constraints.clear();
    }

    /// Calculates sizes for all cells in the layout given a parent size.
    ///
    /// This method performs a two-pass calculation:
    /// 1. First pass calculates fixed sizes and tracks remaining space
    /// 2. Second pass distributes remaining space to Fill constraints
    ///
    /// # Arguments
    ///
    /// * `parent_size` - The size of the parent container
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<Size>)` with the calculated sizes for each cell, or a `LayoutError`
    /// if the constraints cannot be satisfied.
    pub fn calculate_sizes(&self, parent_size: Size) -> Result<Vec<Size>, LayoutError> {
        let mut remaining_width = parent_size.width;
        let mut remaining_height = parent_size.height;

        // First pass: Calculate fixed sizes and track remaining space
        let mut horizontal_sizes = Vec::with_capacity(self.constraints.len());
        let mut vertical_sizes = Vec::with_capacity(self.constraints.len());

        // Calculate horizontal sizes
        for constraint in &self.constraints {
            match constraint {
                Constraint::Fixed(size) => {
                    let size = *size;
                    remaining_width = remaining_width.saturating_sub(size);
                    horizontal_sizes.push(size);
                }
                Constraint::Percentage(percent) => {
                    let size = (parent_size.width as f32 * (*percent as f32 / 100.0)) as u16;
                    remaining_width = remaining_width.saturating_sub(size);
                    horizontal_sizes.push(size);
                }
                Constraint::Min(min) => {
                    horizontal_sizes.push(*min);
                }
                Constraint::Max(max) => {
                    let size = max.min(&parent_size.width);
                    remaining_width = remaining_width.saturating_sub(*size);
                    horizontal_sizes.push(*size);
                }
                Constraint::Range(min, max) => {
                    let size = parent_size.width.clamp(*min, *max);
                    remaining_width = remaining_width.saturating_sub(size);
                    horizontal_sizes.push(size);
                }
                Constraint::Fill(_weight) => {
                    horizontal_sizes.push(0); // Placeholder for Fill constraints
                }
                Constraint::MinWidth(min_width) => {
                    horizontal_sizes.push(*min_width);
                }
                Constraint::MaxWidth(max_width) => {
                    let size = max_width.min(&parent_size.width);
                    remaining_width = remaining_width.saturating_sub(*size);
                    horizontal_sizes.push(*size);
                }
                Constraint::MinHeight(_) | Constraint::MaxHeight(_) => {
                    horizontal_sizes.push(0); // These are handled in vertical calculation
                }
            }
        }

        // Calculate vertical sizes
        for constraint in &self.constraints {
            match constraint {
                Constraint::Fixed(size) => {
                    let size = *size;
                    remaining_height = remaining_height.saturating_sub(size);
                    vertical_sizes.push(size);
                }
                Constraint::Percentage(percent) => {
                    let size = (parent_size.height as f32 * (*percent as f32 / 100.0)) as u16;
                    remaining_height = remaining_height.saturating_sub(size);
                    vertical_sizes.push(size);
                }
                Constraint::Min(min) => {
                    vertical_sizes.push(*min);
                }
                Constraint::Max(max) => {
                    let size = max.min(&parent_size.height);
                    remaining_height = remaining_height.saturating_sub(*size);
                    vertical_sizes.push(*size);
                }
                Constraint::Range(min, max) => {
                    let size = parent_size.height.clamp(*min, *max);
                    remaining_height = remaining_height.saturating_sub(size);
                    vertical_sizes.push(size);
                }
                Constraint::Fill(_weight) => {
                    vertical_sizes.push(0); // Placeholder for Fill constraints
                }
                Constraint::MinWidth(_) | Constraint::MaxWidth(_) => {
                    vertical_sizes.push(0); // These are handled in horizontal calculation
                }
                Constraint::MinHeight(min_height) => {
                    vertical_sizes.push(*min_height);
                }
                Constraint::MaxHeight(max_height) => {
                    let size = max_height.min(&parent_size.height);
                    remaining_height = remaining_height.saturating_sub(*size);
                    vertical_sizes.push(*size);
                }
            }
        }

        // Second pass: Distribute remaining space to Fill constraints
        let total_horizontal_fill: u8 = self.constraints.iter()
            .filter_map(|c| if let Constraint::Fill(w) = c { Some(w) } else { None })
            .sum();
        let total_vertical_fill: u8 = self.constraints.iter()
            .filter_map(|c| if let Constraint::Fill(w) = c { Some(w) } else { None })
            .sum();

        if total_horizontal_fill > 0 {
            let unit = remaining_width as f32 / total_horizontal_fill as f32;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if let Constraint::Fill(weight) = constraint {
                    horizontal_sizes[i] = (unit * *weight as f32) as u16;
                }
            }
        }

        if total_vertical_fill > 0 {
            let unit = remaining_height as f32 / total_vertical_fill as f32;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if let Constraint::Fill(weight) = constraint {
                    vertical_sizes[i] = (unit * *weight as f32) as u16;
                }
            }
        }

        // Create final sizes
        let mut sizes = Vec::with_capacity(self.constraints.len());
        for i in 0..self.constraints.len() {
            sizes.push(Size::new(horizontal_sizes[i], vertical_sizes[i]));
        }

        Ok(sizes)
    }

    /// Calculates a rectangle based on the given constraints and available size.
    /// 
    /// This method applies width and height constraints to determine the final
    /// dimensions of a rectangle. It handles minimum and maximum size constraints
    /// and returns an error if the constraints cannot be satisfied.
    /// 
    /// # Arguments
    /// 
    /// * `available_size` - The total available space for the rectangle
    /// * `constraints` - A slice of constraints to apply to the rectangle
    /// 
    /// # Returns
    /// 
    /// * `Ok(Rect)` - The calculated rectangle with applied constraints
    /// * `Err(LayoutError)` - If the constraints cannot be satisfied
    /// 
    /// # Examples
    /// 
    /// ```
    /// use groundhog_mcp::ui::layout::{Constraint, ConstraintSystem, Size, Rect};
    /// 
    /// let mut system = ConstraintSystem::new();
    /// let rect = system.calculate_rect(
    ///     Size::new(100, 100),
    ///     &[Constraint::MinWidth(50), Constraint::MaxWidth(200)]
    /// ).unwrap();
    /// assert_eq!(rect.size.width, 100);
    /// ```
    pub fn calculate_rect(&self, available_size: Size, constraints: &[Constraint]) -> Result<Rect, LayoutError> {
        let mut rect = Rect::new(
            Position::new(0, 0),
            Size::new(available_size.width, available_size.height)
        );
        
        for constraint in constraints {
            match constraint {
                Constraint::MinWidth(min_width) => {
                    if rect.size.width < *min_width {
                        return Err(LayoutError::InvalidConstraint {
                            message: format!("Width {} is less than minimum width {}", rect.size.width, min_width),
                        });
                    }
                }
                Constraint::MaxWidth(max_width) => {
                    rect.size.width = rect.size.width.min(*max_width);
                }
                Constraint::MinHeight(min_height) => {
                    if rect.size.height < *min_height {
                        return Err(LayoutError::InvalidConstraint {
                            message: format!("Height {} is less than minimum height {}", rect.size.height, min_height),
                        });
                    }
                }
                Constraint::MaxHeight(max_height) => {
                    rect.size.height = rect.size.height.min(*max_height);
                }
                Constraint::Fixed(_) | Constraint::Percentage(_) | Constraint::Min(_) | 
                Constraint::Max(_) | Constraint::Range(_, _) | Constraint::Fill(_) => {
                    // These constraints are handled in calculate_sizes
                }
            }
        }
        
        Ok(rect)
    }
}

impl Default for ConstraintSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_constraint() {
        let constraint = Constraint::Fixed(10);
        assert_eq!(constraint.apply(100).unwrap(), 10);
    }

    #[test]
    fn test_percentage_constraint() {
        let constraint = Constraint::Percentage(50);
        assert_eq!(constraint.apply(100).unwrap(), 50);
    }

    #[test]
    fn test_invalid_percentage() {
        let constraint = Constraint::Percentage(101);
        assert!(constraint.apply(100).is_err());
    }

    #[test]
    fn test_range_constraint() {
        let constraint = Constraint::Range(10, 20);
        assert_eq!(constraint.apply(15).unwrap(), 15);
        assert_eq!(constraint.apply(5).unwrap(), 10);
        assert_eq!(constraint.apply(25).unwrap(), 20);
    }

    #[test]
    fn test_invalid_range() {
        let constraint = Constraint::Range(20, 10);
        assert!(constraint.apply(15).is_err());
    }

    #[test]
    fn test_constraint_system() {
        let mut system = ConstraintSystem::new();
        system.add_constraint(Constraint::Fixed(3));      // Takes 3 units
        system.add_constraint(Constraint::Fill(1));       // Takes remaining space
        system.add_constraint(Constraint::Percentage(50)); // Takes 50% of total space

        let sizes = system.calculate_sizes(Size::new(12, 12)).unwrap();
        assert_eq!(sizes.len(), 3);
        assert_eq!(sizes[0].width, 3);  // Fixed size
        assert_eq!(sizes[1].width, 3);  // Fill takes remaining space after others
        assert_eq!(sizes[2].width, 6);  // 50% of total width
    }
} 