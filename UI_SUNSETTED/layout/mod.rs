/// Module providing layout management functionality for the terminal user interface.
/// This includes grid layouts, constraints, and spacing management.
/// 
/// The layout system is responsible for organizing and positioning UI elements
/// in the terminal window.
pub mod constraint;

/// Grid layout system for arranging UI elements in a grid pattern.
pub mod grid;

/// Layout manager for controlling component positioning and spacing.
pub mod manager;

/// Module for caching layout calculations to improve performance.
mod cache;

use std::io;
use thiserror::Error;
use std::hash::{Hash, Hasher};

/// Errors that can occur during layout operations.
#[derive(Debug, Error)]
pub enum LayoutError {
    /// Invalid indentation size specified.
    #[error("Invalid indent size: {size}")]
    InvalidIndentSize { 
        /// The invalid indent size that was specified
        size: usize 
    },

    /// Attempted to decrease indentation below zero.
    #[error("Negative indentation")]
    NegativeIndentation,

    /// Invalid constraint configuration.
    #[error("Invalid constraint: {message}")]
    InvalidConstraint { 
        /// Description of the invalid constraint
        message: String 
    },

    /// Invalid grid configuration.
    #[error("Invalid grid: {message}")]
    InvalidGrid { 
        /// Description of the invalid grid configuration
        message: String 
    },

    /// Error occurred while caching layout data.
    #[error("Cache error: {message}")]
    CacheError { 
        /// Description of the cache error
        message: String 
    },

    /// Maximum recursion depth exceeded.
    #[error("Maximum recursion depth exceeded")]
    MaxRecursionDepthExceeded,

    /// IO error occurred during layout operations.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub use constraint::{Constraint, ConstraintSystem};
pub use grid::{Grid, GridCell, GridConfig};
pub use manager::LayoutManager;

// Re-export internal cache types that are part of the public API
pub use cache::{LayoutCache, LayoutCacheKey};

/// Represents spacing values for UI elements.
/// 
/// Used to define margins, padding, and other spacing-related properties
/// in the layout system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spacing {
    /// Top spacing value.
    pub top: u16,
    /// Right spacing value.
    pub right: u16,
    /// Bottom spacing value.
    pub bottom: u16,
    /// Left spacing value.
    pub left: u16,
}

impl Spacing {
    /// Creates a new Spacing instance with the specified values.
    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Creates a new Spacing instance with uniform values on all sides.
    #[must_use]
    pub fn uniform(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Returns the total horizontal spacing (left + right).
    pub fn horizontal(&self) -> u16 {
        self.left + self.right
    }

    /// Returns the total vertical spacing (top + bottom).
    pub fn vertical(&self) -> u16 {
        self.top + self.bottom
    }
}

impl Default for Spacing {
    fn default() -> Self {
        Self::uniform(0)
    }
}

/// Represents the size of a UI element.
/// 
/// Used to define the dimensions of UI components in the layout system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    /// Width of the element.
    pub width: u16,
    /// Height of the element.
    pub height: u16,
}

impl Hash for Size {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl Size {
    /// Creates a new Size instance with the specified dimensions.
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Represents a position in the terminal.
/// 
/// Used to define the location of UI elements in the layout system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// X coordinate (horizontal position).
    pub x: u16,
    /// Y coordinate (vertical position).
    pub y: u16,
}

impl Position {
    /// Creates a new Position instance with the specified coordinates.
    #[must_use]
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// Represents a rectangular area in the terminal.
/// 
/// Used to define the boundaries and position of UI elements in the layout system.
/// A Rect combines a position and size to define a complete area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// Position of the rectangle.
    pub position: Position,
    /// Size of the rectangle.
    pub size: Size,
}

impl Rect {
    /// Creates a new Rect instance with the specified position and size.
    #[must_use]
    pub fn new(position: Position, size: Size) -> Self {
        Self { position, size }
    }

    /// Creates a new Rect with the specified margin applied.
    pub fn with_margin(&self, margin: Spacing) -> Self {
        Self {
            position: Position::new(
                self.position.x + margin.left,
                self.position.y + margin.top,
            ),
            size: Size::new(
                self.size.width.saturating_sub(margin.horizontal()),
                self.size.height.saturating_sub(margin.vertical()),
            ),
        }
    }

    /// Checks if the given position is within the rectangle's bounds.
    pub fn contains(&self, position: Position) -> bool {
        position.x >= self.position.x
            && position.x < self.position.x + self.size.width
            && position.y >= self.position.y
            && position.y < self.position.y + self.size.height
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(Position::default(), Size::default())
    }
} 