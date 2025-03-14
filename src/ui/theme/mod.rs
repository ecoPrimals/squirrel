use std::collections::HashMap;
use ratatui::style::{Color as RatatuiColor, Style as RatatuiStyle, Modifier};
use crossterm::style::{Color as CrosstermColor, Attribute as CrosstermAttribute};
use serde::{Deserialize, Serialize};
use crate::ui::error::UiError;
use std::fmt;
use std::collections::HashSet;
use ratatui::prelude::{Color, Style};

pub use self::color::ColorRole;
pub use self::style::StyleRole;
pub use self::theme::Theme;

mod color;
mod style;
mod theme;

/// Theme configuration for UI components.
pub struct Theme {
    /// Name of the theme
    pub name: String,
    /// Description of the theme
    pub description: String,
    /// Color mappings
    colors: Vec<Color>,
    /// Style mappings
    styles: Vec<Style>,
}

impl Theme {
    /// Creates a new theme with the given name and description.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            colors: Vec::new(),
            styles: Vec::new(),
        }
    }

    pub fn get_color(&self, role: ColorRole) -> Color {
        match role {
            ColorRole::Primary => self.colors.get(0).copied().unwrap_or(Color::White),
            ColorRole::Secondary => self.colors.get(1).copied().unwrap_or(Color::Gray),
            ColorRole::Accent => self.colors.get(2).copied().unwrap_or(Color::Blue),
            ColorRole::Background => self.colors.get(3).copied().unwrap_or(Color::Black),
            ColorRole::Border => self.colors.get(4).copied().unwrap_or(Color::DarkGray),
            ColorRole::Text => self.colors.get(5).copied().unwrap_or(Color::White),
            ColorRole::Error => self.colors.get(6).copied().unwrap_or(Color::Red),
            ColorRole::Warning => self.colors.get(7).copied().unwrap_or(Color::Yellow),
            ColorRole::Success => self.colors.get(8).copied().unwrap_or(Color::Green),
            ColorRole::Info => self.colors.get(9).copied().unwrap_or(Color::Cyan),
            ColorRole::Custom(index) => self.colors.get(index as usize).copied().unwrap_or(Color::White),
        }
    }

    pub fn get_style(&self, role: StyleRole) -> Style {
        match role {
            StyleRole::Default => self.styles.get(0).copied().unwrap_or_default(),
            StyleRole::Primary => self.styles.get(1).copied().unwrap_or_default(),
            StyleRole::Secondary => self.styles.get(2).copied().unwrap_or_default(),
            StyleRole::Accent => self.styles.get(3).copied().unwrap_or_default(),
            StyleRole::Border => self.styles.get(4).copied().unwrap_or_default(),
            StyleRole::Text => self.styles.get(5).copied().unwrap_or_default(),
            StyleRole::Error => self.styles.get(6).copied().unwrap_or_default(),
            StyleRole::Warning => self.styles.get(7).copied().unwrap_or_default(),
            StyleRole::Success => self.styles.get(8).copied().unwrap_or_default(),
            StyleRole::Info => self.styles.get(9).copied().unwrap_or_default(),
            StyleRole::Custom(index) => self.styles.get(index as usize).copied().unwrap_or_default(),
        }
    }

    pub fn set_color(&mut self, role: ColorRole, color: Color) {
        let index = match role {
            ColorRole::Primary => 0,
            ColorRole::Secondary => 1,
            ColorRole::Accent => 2,
            ColorRole::Background => 3,
            ColorRole::Border => 4,
            ColorRole::Text => 5,
            ColorRole::Error => 6,
            ColorRole::Warning => 7,
            ColorRole::Success => 8,
            ColorRole::Info => 9,
            ColorRole::Custom(index) => index as usize,
        };

        if index >= self.colors.len() {
            self.colors.resize(index + 1, Color::White);
        }
        self.colors[index] = color;
    }

    pub fn set_style(&mut self, role: StyleRole, style: Style) {
        let index = match role {
            StyleRole::Default => 0,
            StyleRole::Primary => 1,
            StyleRole::Secondary => 2,
            StyleRole::Accent => 3,
            StyleRole::Border => 4,
            StyleRole::Text => 5,
            StyleRole::Error => 6,
            StyleRole::Warning => 7,
            StyleRole::Success => 8,
            StyleRole::Info => 9,
            StyleRole::Custom(index) => index as usize,
        };

        if index >= self.styles.len() {
            self.styles.resize(index + 1, Style::default());
        }
        self.styles[index] = style;
    }

    /// Creates a new style with the given color role.
    pub fn color_style(&self, role: ColorRole) -> Style {
        Style::default().fg(self.get_color(role))
    }

    /// Creates a new style with the given background color role.
    pub fn background_style(&self, role: ColorRole) -> Style {
        Style::default().bg(self.get_color(role))
    }

    /// Creates a new style combining a style role and color role.
    pub fn combined_style(&self, style_role: StyleRole, color_role: ColorRole) -> Style {
        self.get_style(style_role).fg(self.get_color(color_role))
    }
}

impl Default for Theme {
    fn default() -> Self {
        let mut theme = Self::new();
        theme.set_color(ColorRole::Primary, Color::White);
        theme.set_color(ColorRole::Secondary, Color::Gray);
        theme.set_color(ColorRole::Accent, Color::Blue);
        theme.set_color(ColorRole::Background, Color::Black);
        theme.set_color(ColorRole::Border, Color::DarkGray);
        theme.set_color(ColorRole::Text, Color::White);
        theme.set_color(ColorRole::Error, Color::Red);
        theme.set_color(ColorRole::Warning, Color::Yellow);
        theme.set_color(ColorRole::Success, Color::Green);
        theme.set_color(ColorRole::Info, Color::Cyan);

        theme.set_style(StyleRole::Default, Style::default());
        theme.set_style(StyleRole::Primary, Style::default().fg(Color::White));
        theme.set_style(StyleRole::Secondary, Style::default().fg(Color::Gray));
        theme.set_style(StyleRole::Accent, Style::default().fg(Color::Blue));
        theme.set_style(StyleRole::Border, Style::default().fg(Color::DarkGray));
        theme.set_style(StyleRole::Text, Style::default().fg(Color::White));
        theme.set_style(StyleRole::Error, Style::default().fg(Color::Red));
        theme.set_style(StyleRole::Warning, Style::default().fg(Color::Yellow));
        theme.set_style(StyleRole::Success, Style::default().fg(Color::Green));
        theme.set_style(StyleRole::Info, Style::default().fg(Color::Cyan));

        theme
    }
}

/// Theme manager for handling multiple themes.
#[derive(Debug, Default)]
pub struct ThemeManager {
    /// Available themes
    themes: HashMap<String, Theme>,
    /// Currently active theme
    active_theme: Option<String>,
}

impl ThemeManager {
    /// Creates a new theme manager.
    pub fn new() -> Self {
        let mut manager = Self {
            themes: HashMap::new(),
            active_theme: None,
        };
        manager.add_theme(Theme::default());
        manager.set_active_theme("Default");
        manager
    }

    /// Adds a theme to the manager.
    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Gets a reference to a theme by name.
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Gets a mutable reference to a theme by name.
    pub fn get_theme_mut(&mut self, name: &str) -> Option<&mut Theme> {
        self.themes.get_mut(name)
    }

    /// Sets the active theme.
    pub fn set_active_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.active_theme = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Gets a reference to the active theme.
    pub fn active_theme(&self) -> Option<&Theme> {
        self.active_theme.as_ref().and_then(|name| self.get_theme(name))
    }
}

pub trait Themeable {
    fn theme(&self) -> Option<&Theme>;
    fn theme_mut(&mut self) -> &mut Theme;
    fn set_theme(&mut self, theme: Theme);
    fn apply_theme(&mut self, theme: &Theme) -> Result<(), UiError>;
}

#[derive(Debug)]
pub enum ThemeError {
    InvalidColor(String),
    InvalidStyle(String),
}

impl std::fmt::Display for ThemeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeError::InvalidColor(msg) => write!(f, "Invalid color: {}", msg),
            ThemeError::InvalidStyle(msg) => write!(f, "Invalid style: {}", msg),
        }
    }
}

impl std::error::Error for ThemeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_defaults() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Default");
        assert_eq!(theme.get_color(ColorRole::Primary), Color::White);
        assert_eq!(theme.get_color(ColorRole::Error), Color::Red);
    }

    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        assert!(manager.active_theme().is_some());
        assert_eq!(manager.active_theme().unwrap().name, "Default");

        let custom_theme = Theme::new();
        manager.add_theme(custom_theme);
        assert!(manager.set_active_theme("Default"));
        assert_eq!(manager.active_theme().unwrap().name, "Default");
    }

    #[test]
    fn test_theme_styles() {
        let theme = Theme::default();
        let normal_style = theme.get_style(StyleRole::Default);
        let header_style = theme.get_style(StyleRole::Primary);
        
        assert_ne!(normal_style, header_style);
        assert!(header_style.add_modifier.contains(Modifier::BOLD));
    }
} 