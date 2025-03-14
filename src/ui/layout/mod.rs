pub use manager::LayoutManager;

// Re-export internal cache types that are part of the public API
pub use cache::{LayoutCache, LayoutCacheKey};

use std::fmt;
use ratatui::layout::{Constraint, Direction, Layout as TuiLayout, Rect};
use std::hash::{Hash, Hasher};

pub use ratatui::layout::Rect;

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

impl Size {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

impl Hash for Size {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

impl From<LayoutDirection> for Direction {
    fn from(direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Horizontal => Direction::Horizontal,
            LayoutDirection::Vertical => Direction::Vertical,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    direction: LayoutDirection,
    constraints: Vec<Constraint>,
    margin: u16,
    spacing: u16,
}

impl LayoutConfig {
    pub fn new(direction: LayoutDirection) -> Self {
        Self {
            direction,
            constraints: Vec::new(),
            margin: 0,
            spacing: 1,
        }
    }

    pub fn horizontal() -> Self {
        Self::new(LayoutDirection::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(LayoutDirection::Vertical)
    }

    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_margin(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }
}

pub struct Layout {
    config: LayoutConfig,
    chunks: Vec<Rect>,
}

impl Layout {
    pub fn new(config: LayoutConfig) -> Self {
        Self {
            config,
            chunks: Vec::new(),
        }
    }

    pub fn split(&mut self, area: Rect) {
        self.chunks = TuiLayout::default()
            .direction(self.config.direction.into())
            .constraints(self.config.constraints.as_slice())
            .margin(self.config.margin)
            .spacing(self.config.spacing)
            .split(area);
    }

    pub fn get_chunk(&self, index: usize) -> Option<Rect> {
        self.chunks.get(index).copied()
    }
}

pub struct LayoutManager {
    constraints: Vec<Constraint>,
    direction: Direction,
}

impl LayoutManager {
    pub fn new(constraints: Vec<Constraint>, direction: Direction) -> Self {
        Self {
            constraints,
            direction,
        }
    }

    pub fn split(&self, area: Rect) -> Vec<Rect> {
        TuiLayout::default()
            .direction(self.direction)
            .constraints(self.constraints.clone())
            .split(area)
    }
} 