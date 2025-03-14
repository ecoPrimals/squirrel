//! Theme management system for the UI components.
//!
//! This module provides a comprehensive theming system that allows customization of colors,
//! styles, and attributes for UI components. It includes:
//! - Color and attribute definitions
//! - Style management
//! - Theme configuration
//! - Theme management utilities

use crossterm::style::{self, Color as CrosstermColor, Attribute as CrosstermAttribute};
use ratatui::style::{Color as RatatuiColor, Style as RatatuiStyle, Modifier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use thiserror::Error;

use super::{ColorRole, StyleRole, ThemeError};

/// Error type for theme-related operations
#[derive(Debug, Error)]
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

/// A wrapper around RatatuiColor that provides conversion to/from CrosstermColor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(RatatuiColor);

impl Color {
    /// Create a new color
    pub fn new(color: RatatuiColor) -> Self {
        Self(color)
    }

    /// Check if the color is an RGB color
    #[must_use]
    pub fn is_rgb(&self) -> bool {
        matches!(self.0, RatatuiColor::Rgb(..))
    }
}

impl From<RatatuiColor> for Color {
    fn from(color: RatatuiColor) -> Self {
        Self(color)
    }
}

impl From<Color> for RatatuiColor {
    fn from(color: Color) -> Self {
        color.0
    }
}

impl From<Color> for CrosstermColor {
    fn from(color: Color) -> Self {
        match color.0 {
            RatatuiColor::Black => CrosstermColor::Black,
            RatatuiColor::Red => CrosstermColor::Red,
            RatatuiColor::Green => CrosstermColor::Green,
            RatatuiColor::Yellow => CrosstermColor::Yellow,
            RatatuiColor::Blue => CrosstermColor::Blue,
            RatatuiColor::Magenta => CrosstermColor::Magenta,
            RatatuiColor::Cyan => CrosstermColor::Cyan,
            RatatuiColor::Gray => CrosstermColor::Grey,
            RatatuiColor::DarkGray => CrosstermColor::DarkGrey,
            RatatuiColor::LightRed => CrosstermColor::DarkRed,
            RatatuiColor::LightGreen => CrosstermColor::DarkGreen,
            RatatuiColor::LightYellow => CrosstermColor::DarkYellow,
            RatatuiColor::LightBlue => CrosstermColor::DarkBlue,
            RatatuiColor::LightMagenta => CrosstermColor::DarkMagenta,
            RatatuiColor::LightCyan => CrosstermColor::DarkCyan,
            RatatuiColor::White => CrosstermColor::White,
            RatatuiColor::Rgb(r, g, b) => CrosstermColor::Rgb { r, g, b },
            RatatuiColor::Indexed(i) => CrosstermColor::AnsiValue(i),
            RatatuiColor::Reset => CrosstermColor::Reset,
        }
    }
}

/// A wrapper around CrosstermAttribute that provides conversion to RatatuiModifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attribute(CrosstermAttribute);

impl Attribute {
    /// Create a new attribute
    pub fn new(attr: CrosstermAttribute) -> Self {
        Self(attr)
    }
}

impl From<CrosstermAttribute> for Attribute {
    fn from(attr: CrosstermAttribute) -> Self {
        Self(attr)
    }
}

impl From<Attribute> for CrosstermAttribute {
    fn from(attr: Attribute) -> Self {
        attr.0
    }
}

impl From<Attribute> for Modifier {
    fn from(attr: Attribute) -> Self {
        match attr.0 {
            CrosstermAttribute::Bold => Modifier::BOLD,
            CrosstermAttribute::Dim => Modifier::DIM,
            CrosstermAttribute::Italic => Modifier::ITALIC,
            CrosstermAttribute::Underlined => Modifier::UNDERLINED,
            CrosstermAttribute::SlowBlink => Modifier::SLOW_BLINK,
            CrosstermAttribute::RapidBlink => Modifier::RAPID_BLINK,
            CrosstermAttribute::Reverse => Modifier::REVERSED,
            CrosstermAttribute::Hidden => Modifier::HIDDEN,
            CrosstermAttribute::CrossedOut => Modifier::CROSSED_OUT,
            _ => Modifier::empty(),
        }
    }
}

/// A set of text attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attributes {
    bits: u16,
}

impl Attributes {
    /// Create a new empty set of attributes
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    /// Insert an attribute into the set
    pub fn insert(&mut self, modifier: Modifier) {
        self.bits |= modifier as u16;
    }

    /// Remove an attribute from the set
    pub fn remove(&mut self, modifier: Modifier) {
        self.bits &= !(modifier as u16);
    }

    /// Check if an attribute is in the set
    pub fn contains(&self, modifier: Modifier) -> bool {
        self.bits & (modifier as u16) != 0
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self::new()
    }
}

/// A style that can be applied to text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    attrs: Attributes,
    fg: Option<Color>,
    bg: Option<Color>,
}

impl Style {
    /// Create a new style with no colors or attributes
    pub fn new() -> Self {
        Self {
            attrs: Attributes::new(),
            fg: None,
            bg: None,
        }
    }

    /// Create a new style with a foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Create a new style with a background color
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Add a modifier to the style
    pub fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.attrs.insert(modifier);
        self
    }

    /// Remove a modifier from the style
    pub fn remove_modifier(mut self, modifier: Modifier) -> Self {
        self.attrs.remove(modifier);
        self
    }

    /// Get the modifiers of the style
    pub fn modifiers(&self) -> &Attributes {
        &self.attrs
    }

    /// Get the foreground color of the style
    pub fn fg_color(&self) -> Option<Color> {
        self.fg
    }

    /// Get the background color of the style
    pub fn bg_color(&self) -> Option<Color> {
        self.bg
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Style> for RatatuiStyle {
    fn from(style: Style) -> Self {
        let mut ratatui_style = RatatuiStyle::default();
        if let Some(fg) = style.fg {
            ratatui_style = ratatui_style.fg(fg.into());
        }
        if let Some(bg) = style.bg {
            ratatui_style = ratatui_style.bg(bg.into());
        }
        ratatui_style = ratatui_style.add_modifier(style.attrs.to_modifier());
        ratatui_style
    }
}

impl From<RatatuiStyle> for Style {
    fn from(style: RatatuiStyle) -> Self {
        Self {
            fg: style.fg.map(Color::from),
            bg: style.bg.map(Color::from),
            attrs: Attributes::new(), // Note: RatatuiStyle doesn't expose its modifiers
        }
    }
}

/// A styled text with content and style
#[derive(Debug, Clone)]
pub struct StyledText {
    /// The text content
    pub text: String,
    /// The style applied to the text
    pub style: Style,
}

impl StyledText {
    /// Create a new styled text
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the style
    pub fn style(&self) -> Style {
        self.style
    }
}

/// A theme containing colors and styles for UI components
#[derive(Debug, Clone)]
pub struct Theme {
    colors: HashMap<ColorRole, Color>,
    styles: HashMap<StyleRole, Style>,
}

impl Theme {
    /// Create a new empty theme
    pub fn new() -> Self {
        let mut theme = Self {
            colors: HashMap::new(),
            styles: HashMap::new(),
        };
        theme.set_defaults();
        theme
    }

    /// Set default colors and styles
    fn set_defaults(&mut self) {
        // Set default colors
        self.colors.insert(ColorRole::Primary, Color::new(RatatuiColor::Blue));
        self.colors.insert(ColorRole::Secondary, Color::new(RatatuiColor::Cyan));
        self.colors.insert(ColorRole::Background, Color::new(RatatuiColor::Black));
        self.colors.insert(ColorRole::Foreground, Color::new(RatatuiColor::White));
        self.colors.insert(ColorRole::Accent, Color::new(RatatuiColor::Yellow));
        self.colors.insert(ColorRole::Error, Color::new(RatatuiColor::Red));
        self.colors.insert(ColorRole::Warning, Color::new(RatatuiColor::Yellow));
        self.colors.insert(ColorRole::Success, Color::new(RatatuiColor::Green));
        self.colors.insert(ColorRole::Info, Color::new(RatatuiColor::Blue));

        // Set default styles
        self.styles.insert(StyleRole::Default, Style::new());
        self.styles.insert(StyleRole::Header, Style::new().with_fg(self.colors[&ColorRole::Primary]));
        self.styles.insert(StyleRole::Text, Style::new().with_fg(self.colors[&ColorRole::Foreground]));
        self.styles.insert(StyleRole::Error, Style::new().with_fg(self.colors[&ColorRole::Error]));
        self.styles.insert(StyleRole::Warning, Style::new().with_fg(self.colors[&ColorRole::Warning]));
        self.styles.insert(StyleRole::Success, Style::new().with_fg(self.colors[&ColorRole::Success]));
        self.styles.insert(StyleRole::Info, Style::new().with_fg(self.colors[&ColorRole::Info]));
    }

    /// Get a color for a specific role
    pub fn get_color(&self, role: ColorRole) -> Color {
        self.colors[&role]
    }

    /// Set a color for a specific role
    pub fn set_color(&mut self, role: ColorRole, color: Color) {
        self.colors.insert(role, color);
    }

    /// Get a style for a specific role
    pub fn get_style(&self, role: StyleRole) -> Style {
        self.styles[&role]
    }

    /// Set a style for a specific role
    pub fn set_style(&mut self, role: StyleRole, style: Style) {
        self.styles.insert(role, style);
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

/// A color scheme for a theme
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
            primary: Color::new(RatatuiColor::Blue),
            secondary: Color::new(RatatuiColor::Cyan),
            background: Color::new(RatatuiColor::Black),
            foreground: Color::new(RatatuiColor::White),
            accent: Color::new(RatatuiColor::Yellow),
            error: Color::new(RatatuiColor::Red),
            warning: Color::new(RatatuiColor::Yellow),
            success: Color::new(RatatuiColor::Green),
            info: Color::new(RatatuiColor::Blue),
        }
    }
}

impl ColorScheme {
    /// Create a new color scheme with default colors
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the color scheme
    pub fn validate(&self) -> Result<(), ThemeError> {
        // Ensure all colors are valid
        let colors = [
            ("primary", self.primary),
            ("secondary", self.secondary),
            ("background", self.background),
            ("foreground", self.foreground),
            ("accent", self.accent),
            ("error", self.error),
            ("warning", self.warning),
            ("success", self.success),
            ("info", self.info),
        ];

        for (name, color) in colors {
            if color.is_rgb() && !cfg!(feature = "truecolor") {
                return Err(ThemeError::ValidationError(
                    format!("RGB color used for {} but truecolor is not enabled", name)
                ));
            }
        }

        Ok(())
    }
}

/// A set of styles for different UI elements
#[derive(Debug, Clone)]
pub struct StyleSet {
    /// The default style
    pub default: Style,
    /// The header style
    pub header: Style,
    /// The text style
    pub text: Style,
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
    /// Create a new style set with default styles
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the style set
    pub fn validate(&self) -> Result<(), ThemeError> {
        // Ensure all styles have valid colors
        let styles = [
            ("default", &self.default),
            ("header", &self.header),
            ("text", &self.text),
            ("error", &self.error),
            ("warning", &self.warning),
            ("success", &self.success),
            ("info", &self.info),
        ];

        for (name, style) in styles {
            if let Some(color) = style.fg {
                if color.is_rgb() && !cfg!(feature = "truecolor") {
                    return Err(ThemeError::ValidationError(
                        format!("RGB color used in {} style but truecolor is not enabled", name)
                    ));
                }
            }
            if let Some(color) = style.bg {
                if color.is_rgb() && !cfg!(feature = "truecolor") {
                    return Err(ThemeError::ValidationError(
                        format!("RGB color used in {} style background but truecolor is not enabled", name)
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Default for StyleSet {
    fn default() -> Self {
        let color_scheme = ColorScheme::default();
        Self {
            default: Style::new(),
            header: Style::new().with_fg(color_scheme.primary),
            text: Style::new().with_fg(color_scheme.foreground),
            error: Style::new().with_fg(color_scheme.error),
            warning: Style::new().with_fg(color_scheme.warning),
            success: Style::new().with_fg(color_scheme.success),
            info: Style::new().with_fg(color_scheme.info),
        }
    }
}

/// Theme metadata
#[derive(Debug, Clone, Default)]
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

impl ThemeMetadata {
    /// Create new theme metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the metadata
    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.author.is_empty() {
            return Err(ThemeError::ValidationError("Author is required".to_string()));
        }
        if self.version.is_empty() {
            return Err(ThemeError::ValidationError("Version is required".to_string()));
        }
        if self.description.is_empty() {
            return Err(ThemeError::ValidationError("Description is required".to_string()));
        }
        if self.license.is_empty() {
            return Err(ThemeError::ValidationError("License is required".to_string()));
        }
        Ok(())
    }
}

/// Information about a theme
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
    /// Create new theme info
    pub fn new(name: impl Into<String>, description: impl Into<String>, is_builtin: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            is_builtin,
        }
    }
}

/// Trait for theme management
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

/// Default implementation of ThemeManager
#[derive(Debug)]
pub struct DefaultThemeManager {
    /// The current theme
    current_theme: Theme,
    /// The list of available themes
    available_themes: Vec<Theme>,
}

impl DefaultThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        Self {
            current_theme: Theme::default(),
            available_themes: vec![Theme::default()],
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
            .enumerate()
            .map(|(i, _)| {
                ThemeInfo::new(
                    format!("Theme {}", i),
                    "A default theme".to_string(),
                    true,
                )
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

/// Trait for components that can be themed
pub trait Themeable {
    /// Apply a theme to the component
    fn apply_theme(&mut self, theme: &Theme) -> Result<(), ThemeError>;
    /// Get the current style of the component
    fn get_style(&self) -> Style;
    /// Get a color for a specific role
    fn get_color(&self, role: ColorRole) -> Color;
}

/// Color roles for theme components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Style roles for theme components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleRole {
    /// Default style role
    Default,
    /// Header style role
    Header,
    /// Text style role
    Text,
    /// Error style role
    Error,
    /// Warning style role
    Warning,
    /// Success style role
    Success,
    /// Info style role
    Info,
}

impl Default for StyleRole {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager() {
        let manager = DefaultThemeManager::new();
        assert!(manager.get_available_themes().len() > 0);
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::new(RatatuiColor::Red);
        let crossterm_color: CrosstermColor = color.into();
        assert!(matches!(crossterm_color, CrosstermColor::Red));
    }

    #[test]
    fn test_style_creation() {
        let style = Style::new()
            .with_fg(Color::new(RatatuiColor::Red))
            .with_bg(Color::new(RatatuiColor::Blue));
        let ratatui_style: RatatuiStyle = style.into();
        assert_eq!(ratatui_style.fg, Some(RatatuiColor::Red));
        assert_eq!(ratatui_style.bg, Some(RatatuiColor::Blue));
    }

    #[test]
    fn test_theme_validation() {
        let theme = Theme::new();
        assert!(theme.get_color(ColorRole::Primary).is_rgb() == false);
    }
} 