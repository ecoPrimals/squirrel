//! Theme management system for the UI components.
//!
//! This module provides a comprehensive theming system that allows customization of colors,
//! styles, and attributes for UI components. It includes:
//! - Color and attribute definitions
//! - Style management
//! - Theme configuration
//! - Theme management utilities

use crossterm::style::{self, Color, Attribute, Stylize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Error type for theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Failed to load theme: {0}")]
    LoadError(String),
    #[error("Invalid theme format: {0}")]
    FormatError(String),
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Theme validation failed: {0}")]
    ValidationError(String),
}

/// This enum provides a comprehensive set of colors that can be used for both
/// foreground and background styling in the terminal UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Black color
    Black,
    /// Red color
    Red,
    /// Green color
    Green,
    /// Yellow color
    Yellow,
    /// Blue color
    Blue,
    /// Magenta color
    Magenta,
    /// Cyan color
    Cyan,
    /// White color
    White,
    /// Gray color
    Gray,
    /// Bright red color
    BrightRed,
    /// Bright green color
    BrightGreen,
    /// Bright yellow color
    BrightYellow,
    /// Bright blue color
    BrightBlue,
    /// Bright magenta color
    BrightMagenta,
    /// Bright cyan color
    BrightCyan,
    /// Bright white color
    BrightWhite,
    /// RGB color with red, green, and blue components
    Rgb { r: u8, g: u8, b: u8 },
}

impl From<Color> for crossterm::style::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => crossterm::style::Color::Black,
            Color::Red => crossterm::style::Color::Red,
            Color::Green => crossterm::style::Color::Green,
            Color::Yellow => crossterm::style::Color::Yellow,
            Color::Blue => crossterm::style::Color::Blue,
            Color::Magenta => crossterm::style::Color::Magenta,
            Color::Cyan => crossterm::style::Color::Cyan,
            Color::White => crossterm::style::Color::White,
            Color::Gray => crossterm::style::Color::Grey,
            Color::BrightRed => crossterm::style::Color::DarkRed,
            Color::BrightGreen => crossterm::style::Color::DarkGreen,
            Color::BrightYellow => crossterm::style::Color::DarkYellow,
            Color::BrightBlue => crossterm::style::Color::DarkBlue,
            Color::BrightMagenta => crossterm::style::Color::DarkMagenta,
            Color::BrightCyan => crossterm::style::Color::DarkCyan,
            Color::BrightWhite => crossterm::style::Color::Grey,
            Color::Rgb { r, g, b } => crossterm::style::Color::Rgb { r, g, b },
        }
    }
}

impl Color {
    /// Check if the color is an RGB color
    #[must_use]
    pub fn is_rgb(&self) -> bool {
        matches!(self, Color::Rgb { .. })
    }

    /// Convert to crossterm color
    pub fn to_crossterm(&self) -> crossterm::style::Color {
        (*self).into()
    }
}

/// Represents a text attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    /// Bold text
    Bold,
    /// Dimmed text
    Dim,
    /// Underlined text
    Underlined,
    /// Reversed text
    Reverse,
    /// Hidden text
    Hidden,
    /// Italic text
    Italic,
    /// Strikethrough text
    Strikethrough,
    /// Slow blink text
    SlowBlink,
    /// Rapid blink text
    RapidBlink,
    /// Overlined text
    Overlined,
}

impl From<Attribute> for crossterm::style::Attribute {
    fn from(attr: Attribute) -> Self {
        match attr {
            Attribute::Bold => crossterm::style::Attribute::Bold,
            Attribute::Dim => crossterm::style::Attribute::Dim,
            Attribute::Underlined => crossterm::style::Attribute::Underlined,
            Attribute::Reverse => crossterm::style::Attribute::Reverse,
            Attribute::Hidden => crossterm::style::Attribute::Hidden,
            Attribute::Italic => crossterm::style::Attribute::Italic,
            Attribute::Strikethrough => crossterm::style::Attribute::CrossedOut,
            Attribute::SlowBlink => crossterm::style::Attribute::SlowBlink,
            Attribute::RapidBlink => crossterm::style::Attribute::RapidBlink,
            Attribute::Overlined => crossterm::style::Attribute::Overlined,
        }
    }
}

impl Attribute {
    /// Convert to crossterm attribute
    pub fn to_crossterm(&self) -> crossterm::style::Attribute {
        (*self).into()
    }
}

/// A set of text attributes
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    /// The set of attributes
    attributes: std::collections::HashSet<Attribute>,
}

impl Attributes {
    /// Create a new empty set of attributes
    #[must_use]
    pub fn new() -> Self {
        Self {
            attributes: std::collections::HashSet::new(),
        }
    }

    /// Add an attribute to the set
    pub fn add(&mut self, attr: Attribute) {
        self.attributes.insert(attr);
    }

    /// Remove an attribute from the set
    pub fn remove(&mut self, attr: Attribute) {
        self.attributes.remove(&attr);
    }

    /// Check if an attribute is in the set
    pub fn contains(&self, attr: Attribute) -> bool {
        self.attributes.contains(&attr)
    }

    /// Get an iterator over the attributes
    pub fn iter(&self) -> std::collections::hash_set::Iter<Attribute> {
        self.attributes.iter()
    }

    /// Clear all attributes
    pub fn clear(&mut self) {
        self.attributes.clear();
    }

    /// Get the number of attributes in the set
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

impl IntoIterator for Attributes {
    type Item = Attribute;
    type IntoIter = std::collections::hash_set::IntoIter<Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = &'a Attribute;
    type IntoIter = std::collections::hash_set::Iter<'a, Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter()
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}

/// A style that can be applied to text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// The foreground color
    pub fg: Option<Color>,
    /// The background color
    pub bg: Option<Color>,
    /// The text attributes
    pub attrs: std::collections::HashSet<Attribute>,
}

impl Style {
    /// Create a new style with no colors or attributes
    #[must_use]
    pub fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            attrs: std::collections::HashSet::new(),
        }
    }

    /// Create a new style with a foreground color
    #[must_use]
    pub fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            bg: None,
            attrs: std::collections::HashSet::new(),
        }
    }

    /// Create a new style with a background color
    #[must_use]
    pub fn bg(color: Color) -> Self {
        Self {
            fg: None,
            bg: Some(color),
            attrs: std::collections::HashSet::new(),
        }
    }

    /// Create a new style with a foreground and background color
    #[must_use]
    pub fn fg_bg(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            attrs: std::collections::HashSet::new(),
        }
    }

    /// Add a foreground color to the style
    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Add a background color to the style
    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Add an attribute to the style
    pub fn with_attr(mut self, attr: Attribute) -> Self {
        self.attrs.insert(attr);
        self
    }

    /// Add multiple attributes to the style
    pub fn with_attrs(mut self, attrs: impl IntoIterator<Item = Attribute>) -> Self {
        self.attrs.extend(attrs);
        self
    }

    /// Apply the style to a string
    pub fn apply<T: Into<String>>(&self, text: T) -> StyledText {
        StyledText {
            text: text.into(),
            style: *self,
        }
    }

    /// Convert to crossterm style
    pub fn to_crossterm(&self) -> crossterm::style::Style {
        let mut style = crossterm::style::Style::default();
        
        if let Some(fg) = self.fg {
            style = style.foreground(fg.to_crossterm());
        }
        
        if let Some(bg) = self.bg {
            style = style.background(bg.to_crossterm());
        }
        
        for attr in &self.attrs {
            style = style.attribute(attr.to_crossterm());
        }
        
        style
    }
}

/// A text with a style applied to it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledText {
    /// The text content
    pub text: String,
    /// The style applied to the text
    pub style: Style,
}

impl StyledText {
    /// Create a new styled text
    #[must_use]
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    /// Get the text content
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the style
    #[must_use]
    pub fn style(&self) -> Style {
        self.style
    }

    /// Convert to crossterm styled text
    pub fn to_crossterm(&self) -> crossterm::style::StyledContent<String> {
        crossterm::style::StyledContent::new(self.style.to_crossterm(), self.text.clone())
    }
}

/// Represents a complete theme for the terminal UI
#[derive(Debug, Clone)]
pub struct Theme {
    /// The name of the theme
    pub name: String,
    /// The color scheme
    pub colors: ColorScheme,
    /// The set of styles
    pub styles: StyleSet,
    /// Theme metadata
    pub metadata: ThemeMetadata,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            colors: ColorScheme {
                primary: Color::Blue,
                secondary: Color::Cyan,
                background: Color::Black,
                foreground: Color::White,
                accent: Color::Magenta,
                error: Color::Red,
                warning: Color::Yellow,
                success: Color::Green,
                info: Color::Blue,
            },
            styles: StyleSet {
                default: Style::new(),
                header: Style::new().with_attr(Attribute::Bold),
                heading: Style::new().with_attr(Attribute::Bold),
                subheading: Style::new().with_attr(Attribute::Bold),
                text: Style::new(),
                link: Style::new().with_fg(Color::Blue),
                button: Style::new().with_bg(Color::Blue),
                input: Style::new(),
                error: Style::new().with_fg(Color::Red),
                warning: Style::new().with_fg(Color::Yellow),
                success: Style::new().with_fg(Color::Green),
                info: Style::new().with_fg(Color::Blue),
            },
            metadata: ThemeMetadata {
                author: "System".to_string(),
                version: "1.0.0".to_string(),
                description: "Default theme".to_string(),
                license: "MIT".to_string(),
            },
        }
    }
}

impl Theme {
    /// Create a new theme with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the theme configuration
    pub fn validate(&self) -> Result<(), ThemeError> {
        // Validate color scheme
        self.colors.validate()?;
        
        // Validate styles
        self.styles.validate()?;
        
        // Validate metadata
        self.metadata.validate()?;
        
        Ok(())
    }
}

/// Represents a color scheme for the theme
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// The primary color
    pub primary: Color,
    /// The secondary color
    pub secondary: Color,
    /// The background color
    pub background: Color,
    /// The foreground color
    pub foreground: Color,
    /// The accent color
    pub accent: Color,
    /// The error color
    pub error: Color,
    /// The warning color
    pub warning: Color,
    /// The success color
    pub success: Color,
    /// The info color
    pub info: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            background: Color::Black,
            foreground: Color::White,
            accent: Color::Magenta,
            error: Color::Red,
            warning: Color::Yellow,
            success: Color::Green,
            info: Color::Blue,
        }
    }
}

impl ColorScheme {
    /// Create a new color scheme with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the color scheme
    pub fn validate(&self) -> Result<(), ThemeError> {
        // Check for invalid color combinations
        if self.background == self.foreground {
            return Err(ThemeError::ValidationError(
                "Background and foreground colors cannot be the same".to_string(),
            ));
        }

        // Check for contrast issues
        if self.primary == self.background {
            return Err(ThemeError::ValidationError(
                "Primary color should contrast with background".to_string(),
            ));
        }

        Ok(())
    }
}

/// Represents a set of styles for the theme
#[derive(Debug, Clone)]
pub struct StyleSet {
    /// The default style
    pub default: Style,
    /// The header style
    pub header: Style,
    /// The heading style
    pub heading: Style,
    /// The subheading style
    pub subheading: Style,
    /// The text style
    pub text: Style,
    /// The link style
    pub link: Style,
    /// The button style
    pub button: Style,
    /// The input style
    pub input: Style,
    /// The error style
    pub error: Style,
    /// The warning style
    pub warning: Style,
    /// The success style
    pub success: Style,
    /// The info style
    pub info: Style,
}

impl StyleSet {
    /// Create a new style set with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            default: Style::new(),
            header: Style::new()
                .with_fg(Color::White)
                .with_attr(Attribute::Bold),
            heading: Style::new()
                .with_fg(Color::White)
                .with_attr(Attribute::Bold),
            subheading: Style::new()
                .with_fg(Color::White)
                .with_attr(Attribute::Bold),
            text: Style::new()
                .with_fg(Color::White),
            link: Style::new()
                .with_fg(Color::Blue)
                .with_attr(Attribute::Underlined),
            button: Style::new()
                .with_bg(Color::Blue)
                .with_fg(Color::White),
            input: Style::new()
                .with_fg(Color::White),
            error: Style::new()
                .with_fg(Color::Red),
            warning: Style::new()
                .with_fg(Color::Yellow),
            success: Style::new()
                .with_fg(Color::Green),
            info: Style::new()
                .with_fg(Color::Blue),
        }
    }

    /// Validate the style set
    pub fn validate(&self) -> Result<(), ThemeError> {
        // Validate each style in the set
        let mut result = Ok(());
        result = result.and_then(|| self.default.validate())?;
        result = result.and_then(|| self.header.validate())?;
        result = result.and_then(|| self.heading.validate())?;
        result = result.and_then(|| self.subheading.validate())?;
        result = result.and_then(|| self.text.validate())?;
        result = result.and_then(|| self.link.validate())?;
        result = result.and_then(|| self.button.validate())?;
        result = result.and_then(|| self.input.validate())?;
        result = result.and_then(|| self.error.validate())?;
        result = result.and_then(|| self.warning.validate())?;
        result = result.and_then(|| self.success.validate())?;
        result = result.and_then(|| self.info.validate())?;
        result
    }
}

impl Default for StyleSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents metadata for the theme
#[derive(Debug, Clone)]
pub struct ThemeMetadata {
    /// The author of the theme
    pub author: String,
    /// The version of the theme
    pub version: String,
    /// The description of the theme
    pub description: String,
    /// The license of the theme
    pub license: String,
}

impl Default for ThemeMetadata {
    fn default() -> Self {
        Self {
            author: "System".to_string(),
            version: "1.0.0".to_string(),
            description: "Default theme".to_string(),
            license: "MIT".to_string(),
        }
    }
}

impl ThemeMetadata {
    /// Create a new theme metadata with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the theme metadata
    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.name.is_empty() {
            return Err(ThemeError::ValidationError("Theme name cannot be empty".to_string()));
        }

        if self.version.is_empty() {
            return Err(ThemeError::ValidationError("Theme version cannot be empty".to_string()));
        }

        if self.description.is_empty() {
            return Err(ThemeError::ValidationError("Theme description cannot be empty".to_string()));
        }

        Ok(())
    }
}

/// Represents information about a theme
#[derive(Debug, Clone)]
pub struct ThemeInfo {
    /// The name of the theme
    pub name: String,
    /// The description of the theme
    pub description: String,
    /// Whether the theme is built-in
    pub is_builtin: bool,
}

impl ThemeInfo {
    /// Create a new theme info with default values
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default theme".to_string(),
            is_builtin: true,
        }
    }
}

/// This trait defines the operations that can be performed on themes,
/// including loading, applying, and managing themes.
pub trait ThemeManager {
    /// Load a theme
    fn load_theme(&mut self, theme: Theme) -> Result<(), ThemeError>;
    /// Get the current theme
    fn get_current_theme(&self) -> &Theme;
    /// Get all available themes
    fn get_available_themes(&self) -> Vec<ThemeInfo>;
    /// Create a custom theme
    fn create_custom_theme(&mut self, theme: Theme) -> Result<(), ThemeError>;
}

/// Default implementation of the theme manager
#[derive(Debug)]
pub struct DefaultThemeManager {
    /// The current theme
    current_theme: Theme,
    /// The list of available themes
    available_themes: Vec<Theme>,
}

impl DefaultThemeManager {
    /// Create a new theme manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_theme: Theme::default(),
            available_themes: Vec::new(),
        }
    }
}

impl ThemeManager for DefaultThemeManager {
    fn load_theme(&mut self, theme: Theme) -> Result<(), ThemeError> {
        self.current_theme = theme;
        Ok(())
    }

    fn get_current_theme(&self) -> &Theme {
        &self.current_theme
    }

    fn get_available_themes(&self) -> Vec<ThemeInfo> {
        self.available_themes
            .iter()
            .map(|theme| ThemeInfo {
                name: theme.name.clone(),
                description: theme.metadata.description.clone(),
                is_builtin: true,
            })
            .collect()
    }

    fn create_custom_theme(&mut self, theme: Theme) -> Result<(), ThemeError> {
        self.available_themes.push(theme);
        Ok(())
    }
}

impl Default for DefaultThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// This trait should be implemented by any UI component that wants to
/// support theming. It provides methods for applying themes and getting
/// the current style.
pub trait Themeable {
    /// Apply a theme to the component
    fn apply_theme(&mut self, theme: &Theme) -> Result<(), ThemeError>;
    /// Get the current style of the component
    fn get_style(&self) -> &Style;
    /// Get a color for a specific role
    fn get_color(&self, role: ColorRole) -> Color;
}

/// Represents a role for a color in the theme system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    /// Primary color role
    Primary,
    /// Secondary color role
    Secondary,
    /// Background color role
    Background,
    /// Foreground color role
    Foreground,
    /// Accent color role
    Accent,
    /// Error color role
    Error,
    /// Warning color role
    Warning,
    /// Success color role
    Success,
    /// Info color role
    Info,
}

impl Default for ColorRole {
    fn default() -> Self {
        Self::Primary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager() {
        let mut manager = DefaultThemeManager::new();
        assert_eq!(manager.get_current_theme().name, "default");
    }

    #[test]
    fn test_custom_theme() {
        let mut manager = DefaultThemeManager::new();
        let custom_theme = Theme {
            name: "custom".to_string(),
            colors: ColorScheme {
                primary: Color::Red,
                secondary: Color::Green,
                background: Color::Black,
                foreground: Color::White,
                accent: Color::Blue,
                error: Color::Red,
                warning: Color::Yellow,
                success: Color::Green,
                info: Color::Blue,
            },
            styles: StyleSet::new(),
            metadata: ThemeMetadata {
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: "Custom theme".to_string(),
                license: "MIT".to_string(),
            },
        };

        assert!(manager.create_custom_theme(custom_theme).is_ok());
        assert_eq!(manager.get_available_themes().len(), 1);
    }

    #[test]
    fn test_style_builder() {
        let style = Style::new()
            .with_attr(Attribute::Bold)
            .with_attr(Attribute::Underlined)
            .with_fg(Color::Red)
            .with_bg(Color::Black);

        assert!(style.attrs.contains(&Attribute::Bold));
        assert!(style.attrs.contains(&Attribute::Underlined));
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Black));
    }

    #[test]
    fn test_style_attributes() {
        let mut attributes = Attributes::new();
        attributes.add(Attribute::Bold);
        attributes.add(Attribute::Underlined);

        assert!(attributes.contains(&Attribute::Bold));
        assert!(attributes.contains(&Attribute::Underlined));
        assert_eq!(attributes.len(), 2);

        attributes.remove(&Attribute::Bold);
        assert!(!attributes.contains(&Attribute::Bold));
        assert_eq!(attributes.len(), 1);
    }

    #[test]
    fn test_style_application() {
        let style = Style::new()
            .with_attr(Attribute::Bold)
            .with_fg(Color::Red);
        let content = "test";
        let styled = style.apply(content);
        assert_eq!(styled.text, "test");
    }

    #[test]
    fn test_style_creation() {
        let style = Style::new();
        assert!(style.attrs.is_empty());
        assert!(style.fg.is_none());
        assert!(style.bg.is_none());
    }

    #[test]
    fn test_color_conversion() {
        let rgb_color = Color::Rgb { r: 255, g: 0, b: 0 };
        let crossterm_color: crossterm::style::Color = rgb_color.into();
        assert!(matches!(crossterm_color, crossterm::style::Color::Rgb { r: 255, g: 0, b: 0 }));
    }

    #[test]
    fn test_attribute_conversion() {
        let bold = Attribute::Bold;
        let crossterm_attr: crossterm::style::Attribute = bold.into();
        assert!(matches!(crossterm_attr, crossterm::style::Attribute::Bold));
    }

    #[test]
    fn test_style_set_default() {
        let style_set = StyleSet::default();
        assert!(style_set.default.attrs.is_empty());
        assert!(style_set.header.attrs.contains(&Attribute::Bold));
        assert!(style_set.link.attrs.contains(&Attribute::Underlined));
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
        assert_eq!(theme.colors.primary, Color::Blue);
        assert_eq!(theme.colors.background, Color::Black);
        assert_eq!(theme.colors.foreground, Color::White);
    }

    #[test]
    fn test_theme_manager_operations() {
        let mut manager = DefaultThemeManager::new();
        
        // Test loading a theme
        let new_theme = Theme::default();
        assert!(manager.load_theme(new_theme).is_ok());
        
        // Test getting current theme
        let current_theme = manager.get_current_theme();
        assert_eq!(current_theme.name, "default");
        
        // Test creating a custom theme
        let custom_theme = Theme {
            name: "custom".to_string(),
            colors: ColorScheme::new(),
            styles: StyleSet::new(),
            metadata: ThemeMetadata::new(),
        };
        assert!(manager.create_custom_theme(custom_theme).is_ok());
        
        // Test getting available themes
        let available_themes = manager.get_available_themes();
        assert_eq!(available_themes.len(), 1);
        assert_eq!(available_themes[0].name, "custom");
    }

    #[test]
    fn test_theme_validation() {
        let theme = Theme::default();
        assert!(theme.validate().is_ok());

        // Test invalid color scheme
        let mut invalid_theme = Theme::default();
        invalid_theme.colors.background = Color::White;
        invalid_theme.colors.foreground = Color::White;
        assert!(invalid_theme.validate().is_err());

        // Test invalid metadata
        let mut invalid_theme = Theme::default();
        invalid_theme.metadata.name = String::new();
        assert!(invalid_theme.validate().is_err());
    }

    #[test]
    fn test_color_scheme_validation() {
        let scheme = ColorScheme::default();
        assert!(scheme.validate().is_ok());

        // Test invalid background/foreground combination
        let mut invalid_scheme = ColorScheme::default();
        invalid_scheme.background = Color::White;
        invalid_scheme.foreground = Color::White;
        assert!(invalid_scheme.validate().is_err());

        // Test invalid primary/background combination
        let mut invalid_scheme = ColorScheme::default();
        invalid_scheme.primary = Color::Black;
        invalid_scheme.background = Color::Black;
        assert!(invalid_scheme.validate().is_err());
    }

    #[test]
    fn test_theme_metadata_validation() {
        let metadata = ThemeMetadata::default();
        assert!(metadata.validate().is_ok());

        // Test empty name
        let mut invalid_metadata = ThemeMetadata::default();
        invalid_metadata.name = String::new();
        assert!(invalid_metadata.validate().is_err());

        // Test empty version
        let mut invalid_metadata = ThemeMetadata::default();
        invalid_metadata.version = String::new();
        assert!(invalid_metadata.validate().is_err());

        // Test empty description
        let mut invalid_metadata = ThemeMetadata::default();
        invalid_metadata.description = String::new();
        assert!(invalid_metadata.validate().is_err());
    }

    #[test]
    fn test_style_set_validation() {
        let style_set = StyleSet::default();
        assert!(style_set.validate().is_ok());

        // Test invalid style
        let mut invalid_style_set = StyleSet::default();
        invalid_style_set.default = Style::new().with_fg(Color::White).with_bg(Color::White);
        assert!(invalid_style_set.validate().is_err());
    }
} 