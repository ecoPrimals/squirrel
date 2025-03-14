use crossterm::{
    style,
    QueueableCommand,
};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

use crate::ui::layout::LayoutManager;
use crate::ui::theme::{self, Theme, Themeable, ColorRole, ThemeError, Style, Color};

/// Style options for box borders in headers
#[derive(Debug, Clone, Default)]
pub enum BoxStyle {
    /// Rounded corners for the box border.
    #[default]
    Rounded,
    /// Sharp corners for the box border.
    Sharp,
    /// Double lines for the box border.
    Double,
    /// No visible border.
    Hidden,
}

impl BoxStyle {
    /// Returns the characters used for drawing the box borders
    /// Returns (top-left, top-right, bottom-left, bottom-right, horizontal, vertical)
    pub fn get_chars(&self) -> (char, char, char, char, char, char) {
        match self {
            BoxStyle::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
            BoxStyle::Sharp => ('┌', '┐', '└', '┘', '─', '│'),
            BoxStyle::Double => ('╔', '╗', '╚', '╝', '═', '║'),
            BoxStyle::Hidden => (' ', ' ', ' ', ' ', ' ', ' '),
        }
    }
}

/// Text alignment options for header content
#[derive(Debug, Clone, Default)]
pub enum Alignment {
    /// Align text to the left.
    #[default]
    Left,
    /// Center the text.
    Center,
    /// Align text to the right.
    Right,
}

/// A header component that can display styled text with borders and gradients
#[derive(Debug)]
pub struct Header {
    /// The layout manager for positioning the header.
    pub layout: LayoutManager,
    /// The main title text.
    pub title: String,
    /// Optional subtitle text.
    pub subtitle: Option<String>,
    /// Style configuration for the header.
    pub style: HeaderStyle,
    /// Maximum width of the header.
    pub max_width: Option<usize>,
}

/// Style configuration for headers
#[derive(Debug, Clone)]
pub struct HeaderStyle {
    /// Base style for the header.
    pub style: Style,
    /// Border style for the header box.
    pub box_style: BoxStyle,
    /// Text alignment within the header.
    pub alignment: Alignment,
    /// Padding around the header content.
    pub padding: usize,
    /// Optional gradient colors for the header text.
    pub gradient: Option<(Color, Color)>,
}

impl Default for HeaderStyle {
    fn default() -> Self {
        Self {
            style: Style::new(),
            box_style: BoxStyle::default(),
            alignment: Alignment::default(),
            padding: 1,
            gradient: None,
        }
    }
}

/// Interpolates between two colors based on a progress value.
///
/// # Arguments
/// * `start` - The starting color
/// * `end` - The ending color
/// * `progress` - A value between 0.0 and 1.0 indicating the interpolation progress
///
/// # Returns
/// The interpolated color
fn interpolate_color(start: &theme::Color, end: &theme::Color, progress: f32) -> theme::Color {
    match (start, end) {
        (theme::Color::Rgb(r1, g1, b1), theme::Color::Rgb(r2, g2, b2)) => {
            let r = interpolate_component(*r1, *r2, progress);
            let g = interpolate_component(*g1, *g2, progress);
            let b = interpolate_component(*b1, *b2, progress);
            theme::Color::Rgb(r, g, b)
        }
        (start_color, end_color) => {
            // Convert non-RGB colors to RGB for smooth interpolation
            let start_rgb = color_to_rgb(start_color);
            let end_rgb = color_to_rgb(end_color);
            interpolate_color(&start_rgb, &end_rgb, progress)
        }
    }
}

/// Converts a theme color to its RGB representation.
///
/// # Arguments
/// * `color` - The theme color to convert
///
/// # Returns
/// The RGB color representation or an error if conversion is not possible
fn color_to_rgb(color: &theme::Color) -> theme::Color {
    match color {
        theme::Color::Black => theme::Color::Rgb(0, 0, 0),
        theme::Color::Red => theme::Color::Rgb(255, 0, 0),
        theme::Color::Green => theme::Color::Rgb(0, 255, 0),
        theme::Color::Yellow => theme::Color::Rgb(255, 255, 0),
        theme::Color::Blue => theme::Color::Rgb(0, 0, 255),
        theme::Color::Magenta => theme::Color::Rgb(255, 0, 255),
        theme::Color::Cyan => theme::Color::Rgb(0, 255, 255),
        theme::Color::White => theme::Color::Rgb(255, 255, 255),
        theme::Color::LightBlack => theme::Color::Rgb(128, 128, 128),
        theme::Color::LightRed => theme::Color::Rgb(255, 128, 128),
        theme::Color::LightGreen => theme::Color::Rgb(128, 255, 128),
        theme::Color::LightYellow => theme::Color::Rgb(255, 255, 128),
        theme::Color::LightBlue => theme::Color::Rgb(128, 128, 255),
        theme::Color::LightMagenta => theme::Color::Rgb(255, 128, 255),
        theme::Color::LightCyan => theme::Color::Rgb(128, 255, 255),
        theme::Color::LightWhite => theme::Color::Rgb(255, 255, 255),
        theme::Color::Rgb(r, g, b) => theme::Color::Rgb(*r, *g, *b),
        theme::Color::Reset => theme::Color::White,
    }
}

/// Interpolates between two color components using gamma correction.
///
/// # Arguments
/// * `start` - The starting component value
/// * `end` - The ending component value
/// * `progress` - A value between 0.0 and 1.0 indicating the interpolation progress
///
/// # Returns
/// The interpolated component value
fn interpolate_component(start: u8, end: u8, progress: f32) -> u8 {
    let start_f = start as f32;
    let end_f = end as f32;
    let gamma = 2.2; // Standard gamma correction value
    
    // Apply gamma correction for perceptually uniform interpolation
    let start_linear = (start_f / 255.0).powf(gamma);
    let end_linear = (end_f / 255.0).powf(gamma);
    
    // Interpolate in linear space
    let result_linear = start_linear + (end_linear - start_linear) * progress;
    
    // Convert back to sRGB space
    let result = (result_linear.powf(1.0 / gamma) * 255.0) as u8;
    result.clamp(0, 255)
}

impl Header {
    /// Creates a new header with the given title.
    ///
    /// # Arguments
    /// * `title` - The title text for the header
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            layout: LayoutManager::new(),
            title: title.into(),
            subtitle: None,
            style: HeaderStyle::default(),
            max_width: None,
        }
    }

    /// Writes text with a gradient effect between two colors.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The text to write
    /// * `start_color` - The starting color of the gradient
    /// * `end_color` - The ending color of the gradient
    /// * `_width` - The total width available (currently unused)
    ///
    /// # Returns
    /// * `io::Result<()>` - Success or error during writing
    pub fn write_gradient_text<W: Write>(
        writer: &mut W,
        text: &str,
        start_color: &Color,
        end_color: &Color,
        _width: usize,  // Prefixed with underscore as it's unused
    ) -> io::Result<()> {
        let text_width = text.width();
        let chars: Vec<_> = text.chars().collect();
        
        for (i, ch) in chars.iter().enumerate() {
            let progress = i as f32 / (text_width.saturating_sub(1) as f32);
            let color = interpolate_color(start_color, end_color, progress);
            
            writer.queue(style::SetForegroundColor(color.into()))?;
            write!(writer, "{}", ch)?;
        }
        
        writer.queue(style::SetAttribute(style::Attribute::Reset))?;
        Ok(())
    }
}

impl Themeable for Header {
    fn get_color(&self, role: ColorRole) -> Color {
        match role {
            ColorRole::Primary => self.style.style.foreground.clone().unwrap_or(Color::White),
            ColorRole::Secondary => self.style.style.background.clone().unwrap_or(Color::Black),
            _ => Color::White,
        }
    }

    fn get_style(&self) -> &Style {
        &self.style.style
    }

    fn apply_theme(&mut self, theme: &Theme) -> Result<(), ThemeError> {
        self.style.style = theme.styles.header.clone();
        Ok(())
    }
} 