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