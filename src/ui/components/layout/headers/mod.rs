use crossterm::{
    style::{self},
    QueueableCommand,
};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;
use std::fmt;

use crate::ui::layout::{LayoutManager, Size, LayoutError};
use crate::ui::theme::{self, Theme, Themeable, ColorRole, ThemeError, Style};

/// Style options for header borders.
/// 
/// Defines different border styles that can be used to draw the header's border.
/// Each style uses different Unicode box-drawing characters to create distinct
/// visual appearances.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::components::header::BoxStyle;
/// 
/// let rounded = BoxStyle::Rounded;  // Uses rounded corners
/// let sharp = BoxStyle::Sharp;      // Uses sharp corners
/// let double = BoxStyle::Double;    // Uses double lines
/// let hidden = BoxStyle::Hidden;    // No visible border
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxStyle {
    /// Rounded corners for the border.
    /// Uses Unicode box-drawing characters with rounded corners.
    Rounded,
    /// Sharp corners for the border.
    /// Uses Unicode box-drawing characters with sharp corners.
    Sharp,
    /// Double lines for the border.
    /// Uses Unicode box-drawing characters with double lines.
    Double,
    /// No visible border.
    /// Uses spaces instead of border characters.
    Hidden,
}

/// Error types that can occur during header operations.
/// 
/// This enum defines all possible errors that can occur when working with the Header component.
/// Each variant includes relevant information about the error condition.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::components::header::HeaderError;
/// 
/// let empty_error = HeaderError::EmptyText;
/// let width_error = HeaderError::InvalidWidth;
/// let text_error = HeaderError::TextTooLong {
///     text_length: 20,
///     max_width: 10,
/// };
/// ```
#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    /// Error when the header text is empty
    #[error("Empty text")]
    EmptyText,
    /// Error when the specified width is invalid (zero or negative)
    #[error("Invalid width")]
    InvalidWidth,
    /// Error when the text length exceeds the maximum allowed width
    #[error("Text too long: length={text_length}, max={max_width}")]
    TextTooLong {
        /// The actual length of the text
        text_length: usize,
        /// The maximum allowed width
        max_width: usize,
    },
    /// Error when gradient colors are invalid
    #[error("Invalid gradient: {message}")]
    InvalidGradient {
        /// Description of the gradient error
        message: String,
    },
    /// Error from the layout system
    #[error("Layout error: {0}")]
    LayoutError(#[from] LayoutError),
    /// Error from IO operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Text alignment options for header content.
/// 
/// Defines how the text content should be positioned within the header's border.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::components::header::Alignment;
/// 
/// let left = Alignment::Left;    // Align text to the left
/// let center = Alignment::Center; // Center the text
/// let right = Alignment::Right;  // Align text to the right
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Align text to the left.
    /// Text starts at the left border with padding.
    Left,
    /// Center the text.
    /// Text is centered between the left and right borders.
    Center,
    /// Align text to the right.
    /// Text ends at the right border with padding.
    Right,
}

/// A header component for displaying titles and subtitles.
/// 
/// The Header component provides a flexible way to display text content with various styling options
/// including borders, alignment, padding, and gradient effects.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::components::header::{Header, BoxStyle, Alignment};
/// 
/// let header = Header::new("my_header", "Hello World")
///     .unwrap()
///     .with_box_style(BoxStyle::Rounded)
///     .with_alignment(Alignment::Center)
///     .with_padding(2);
/// ```
#[derive(Debug)]
pub struct Header {
    /// Unique identifier for the header component
    id: String,
    /// Layout manager for positioning and sizing
    layout: LayoutManager,
    /// The text content to display in the header
    content: String,
    /// Style configuration for the header
    style: HeaderStyle,
}

/// Style configuration for the header component.
/// 
/// HeaderStyle provides a comprehensive set of styling options for the header component,
/// including text style, border style, alignment, padding, and gradient effects.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::components::header::{HeaderStyle, BoxStyle, Alignment};
/// use groundhog_mcp::ui::theme::Style;
/// 
/// let style = HeaderStyle {
///     style: Style::default(),
///     box_style: BoxStyle::Rounded,
///     alignment: Alignment::Center,
///     padding: 2,
///     gradient: None,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct HeaderStyle {
    /// Base style for the header text
    pub style: Style,
    /// Border style for the header
    pub box_style: BoxStyle,
    /// Text alignment within the header
    pub alignment: Alignment,
    /// Padding around the header content
    pub padding: usize,
    /// Optional gradient colors for the header text
    pub gradient: Option<(theme::Color, theme::Color)>,
}

impl BoxStyle {
    /// Returns the characters used for drawing the box borders.
    /// 
    /// Returns a tuple of (top-left, top-right, bottom-left, bottom-right, horizontal, vertical) characters
    /// based on the current box style.
    /// 
    /// # Returns
    /// * `(&'static str, &'static str, &'static str, &'static str, &'static str, &'static str)` - 
    ///   Tuple containing the border characters in the order: (top-left, top-right, bottom-left, bottom-right, horizontal, vertical)
    fn get_chars(&self) -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
        match self {
            BoxStyle::Rounded => ("╭", "╮", "╰", "╯", "─", "│"),
            BoxStyle::Sharp => ("┌", "┐", "└", "┘", "─", "│"),
            BoxStyle::Double => ("╔", "╗", "╚", "╝", "═", "║"),
            BoxStyle::Hidden => (" ", " ", " ", " ", " ", " "),
        }
    }
}

impl Header {
    /// Creates a new header with the given title.
    ///
    /// # Arguments
    /// * `id` - A unique identifier for the header component
    /// * `content` - The text content to display in the header
    ///
    /// # Returns
    /// * `Result<Self, LayoutError>` - A new Header instance or a layout error
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Result<Self, LayoutError> {
        let mut layout = LayoutManager::new();
        let id = id.into();
        let content = content.into();
        
        layout.register_layout(id.clone(), |size: Size, constraints: &[u8]| {
            if constraints.is_empty() {
                return Ok(size);
            }
            let content_len = constraints[0] as u16;
            Ok(Size::new(content_len.min(size.width), 1))
        });

        Ok(Self {
            id,
            layout,
            content,
            style: HeaderStyle::default(),
        })
    }

    /// Sets the layout manager for the header.
    ///
    /// # Arguments
    /// * `layout` - The layout manager to use for positioning and sizing
    ///
    /// # Returns
    /// * `&mut Self` - The header instance for method chaining
    pub fn set_layout(&mut self, layout: LayoutManager) -> &mut Self {
        self.layout = layout;
        self
    }

    /// Sets the maximum width for the header.
    ///
    /// # Arguments
    /// * `width` - The maximum width to set
    ///
    /// # Returns
    /// * `Result<&mut Self, HeaderError>` - The header instance for method chaining or an error
    pub fn set_max_width(&mut self, width: usize) -> Result<&mut Self, HeaderError> {
        if width == 0 {
            return Err(HeaderError::InvalidWidth);
        }

        let content_len = self.content.len();
        if content_len > width {
            return Err(HeaderError::TextTooLong {
                text_length: content_len,
                max_width: width,
            });
        }

        self.layout.calculate_layout(&self.id, Size::new(width as u16, 1), &[content_len as u8])
            .map_err(HeaderError::LayoutError)?;
        Ok(self)
    }

    /// Adds a subtitle to the header.
    ///
    /// # Arguments
    /// * `subtitle` - The subtitle text to add
    ///
    /// # Returns
    /// * `Self` - The header instance for method chaining
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.content = format!("{} - {}", self.content, subtitle.into());
        self
    }

    /// Sets the box style for the header.
    ///
    /// # Arguments
    /// * `box_style` - The box style to use for the header border
    ///
    /// # Returns
    /// * `Self` - The header instance for method chaining
    pub fn with_box_style(mut self, box_style: BoxStyle) -> Self {
        self.style.box_style = box_style;
        self
    }

    /// Sets the text alignment for the header.
    ///
    /// # Arguments
    /// * `alignment` - The alignment to use for the header text
    ///
    /// # Returns
    /// * `Self` - The header instance for method chaining
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.style.alignment = alignment;
        self
    }

    /// Sets the padding for the header.
    ///
    /// # Arguments
    /// * `padding` - The padding value to set
    ///
    /// # Returns
    /// * `Self` - The header instance for method chaining
    pub fn with_padding(mut self, padding: usize) -> Self {
        self.style.padding = padding;
        self
    }

    /// Sets a gradient color effect for the header text.
    ///
    /// # Arguments
    /// * `start_color` - The starting color of the gradient
    /// * `end_color` - The ending color of the gradient
    ///
    /// # Returns
    /// * `Result<Self, HeaderError>` - The header instance for method chaining or an error
    pub fn with_gradient(mut self, start_color: theme::Color, end_color: theme::Color) -> Result<Self, HeaderError> {
        if !start_color.is_rgb() || !end_color.is_rgb() {
            return Err(HeaderError::InvalidGradient {
                message: "Both colors must be RGB colors".to_string(),
            });
        }

        self.style.gradient = Some((start_color, end_color));
        Ok(self)
    }

    /// Calculates the total width of the header content including padding.
    /// 
    /// Returns the sum of the text width and padding on both sides.
    fn calculate_total_width(&self) -> usize {
        UnicodeWidthStr::width(&self.content[..]) + (self.style.padding * 2)
    }

    /// Calculates the padding needed for text alignment.
    /// 
    /// # Arguments
    /// * `text_width` - The width of the text content
    /// * `total_width` - The total available width
    /// 
    /// Returns the number of padding spaces needed based on the alignment setting.
    fn calculate_alignment_padding(&self, text_width: usize, total_width: usize) -> usize {
        let available_width = total_width.saturating_sub(text_width);
        match self.style.alignment {
            Alignment::Left => self.style.padding,
            Alignment::Center => {
                let remaining_space = available_width.saturating_sub(self.style.padding * 2);
                self.style.padding + (remaining_space / 2)
            }
            Alignment::Right => {
                available_width.saturating_sub(self.style.padding)
            }
        }
    }

    /// Draws a horizontal border line for the header.
    /// 
    /// # Arguments
    /// * `writer` - The output writer
    /// * `indent` - The indentation level
    /// * `left` - The left border character
    /// * `middle` - The middle border character
    /// * `right` - The right border character
    /// * `width` - The width of the border
    fn draw_border<W: Write>(
        &self,
        writer: &mut W,
        indent: usize,
        left: char,
        middle: char,
        right: char,
        _width: usize,
    ) -> io::Result<()> {
        writer.write_all(" ".repeat(indent).as_bytes())?;
        writer.write_all(left.to_string().as_bytes())?;
        writer.write_all(middle.to_string().repeat(self.calculate_total_width()).as_bytes())?;
        writer.write_all(right.to_string().as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    /// Draws the content line of the header.
    #[allow(dead_code)]
    fn draw_content<W: Write>(
        &self,
        writer: &mut W,
        indent: usize,
        vertical: char,
        text: &str,
        align_padding: usize,
        total_width: usize,
    ) -> io::Result<()> {
        writer.write_all(" ".repeat(indent).as_bytes())?;
        writer.write_all(vertical.to_string().as_bytes())?;
        writer.write_all(" ".repeat(align_padding).as_bytes())?;
        writer.write_all(text.as_bytes())?;
        writer.write_all(" ".repeat(total_width.saturating_sub(align_padding + text.len())).as_bytes())?;
        writer.write_all(vertical.to_string().as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    /// Writes gradient text to the output.
    ///
    /// # Arguments
    /// * `writer` - The output writer
    /// * `text` - The text to write
    /// * `start_color` - The starting color of the gradient
    /// * `end_color` - The ending color of the gradient
    pub fn write_gradient_text<W: Write>(
        &self,
        writer: &mut W,
        text: &str,
        start_color: &theme::Color,
        end_color: &theme::Color,
    ) -> io::Result<()> {
        // Pre-calculate all colors for the gradient
        let char_count = text.chars().count();
        let colors: Vec<theme::Color> = (0..char_count)
            .map(|i| {
                // Calculate the interpolated color based on progress
                let progress = (i as f64) / (char_count as f64);
                interpolate_color(start_color, end_color, progress)
            })
            .collect();

        // Create a buffer for the colored text
        let mut buffer = Vec::with_capacity(text.len() * 20); // Estimate size including ANSI codes

        // Write all characters with their colors to the buffer
        for (c, color) in text.chars().zip(colors.iter()) {
            let color_owned = color.clone();
            write!(&mut buffer, "{}{}", style::SetForegroundColor(color_owned.into()), c)?;
        }

        // Write the buffer to the output and reset the style
        writer.write_all(&buffer)?;
        writer.queue(style::SetAttribute(style::Attribute::Reset))?;
        Ok(())
    }

    /// Prints the header to the given writer with the specified content.
    /// 
    /// This method handles the actual rendering of the header, including borders,
    /// content alignment, and gradient effects if enabled.
    /// 
    /// # Arguments
    /// * `writer` - The output writer to write the header to
    /// * `content` - The text content to display in the header
    /// 
    /// # Returns
    /// * `Result<(), HeaderError>` - Ok if successful, error otherwise
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::io::Cursor;
    /// use groundhog_mcp::ui::components::header::Header;
    /// 
    /// let mut buffer = Cursor::new(Vec::new());
    /// let header = Header::new("test", "Hello World").unwrap();
    /// header.print_header(&mut buffer, "Hello World").unwrap();
    /// ```
    pub fn print_header<W: Write>(&self, writer: &mut W, content: &str) -> Result<(), HeaderError> {
        if content.is_empty() {
            return Err(HeaderError::EmptyText);
        }

        let total_width = self.calculate_total_width();
        let text_width = UnicodeWidthStr::width(content);
        let align_padding = self.calculate_alignment_padding(text_width, total_width);

        let (_h, _v, _tl, _tr, _bl, _br) = self.style.box_style.get_chars();

        // Draw top border
        self.draw_border(writer, 0, _tl.chars().next().unwrap(), _h.chars().next().unwrap(), _tr.chars().next().unwrap(), total_width)?;

        // Draw content line
        writer.write_all(" ".repeat(0).as_bytes())?;
        writer.write_all(_v.to_string().as_bytes())?;

        if let Some((start_color, end_color)) = &self.style.gradient {
            writer.write_all(" ".repeat(align_padding).as_bytes())?;
            self.write_gradient_text(writer, content, start_color, end_color)?;
        } else {
            writer.write_all(" ".repeat(align_padding).as_bytes())?;
            writer.write_all(content.as_bytes())?;
        }

        writer.write_all(" ".repeat(total_width.saturating_sub(align_padding + text_width)).as_bytes())?;
        writer.write_all(_v.to_string().as_bytes())?;
        writer.write_all(b"\n")?;

        // Draw bottom border
        self.draw_border(writer, 0, _bl.chars().next().unwrap(), _h.chars().next().unwrap(), _br.chars().next().unwrap(), total_width)?;

        Ok(())
    }

    /// Writes the complete header to the given writer.
    ///
    /// # Arguments
    /// * `writer` - The output writer to write the header to
    ///
    /// # Returns
    /// * `Result<(), HeaderError>` - Ok if successful, error otherwise
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), HeaderError> {
        self.print_header(&mut writer, &self.content)
    }

    /// Calculates the layout for the header component.
    ///
    /// # Arguments
    /// * `size` - The available size for the header
    ///
    /// # Returns
    /// * `Result<Size, LayoutError>` - The calculated size or a layout error
    pub fn calculate_layout(&mut self, size: Size) -> Result<Size, LayoutError> {
        let content_len = self.content.len();
        self.layout.calculate_layout(&self.id, size, &[content_len as u8])
    }
}

impl Themeable for Header {
    fn get_color(&self, role: ColorRole) -> theme::Color {
        match role {
            ColorRole::Primary => self.style.style.foreground.clone().unwrap_or(theme::Color::White),
            ColorRole::Secondary => self.style.style.background.clone().unwrap_or(theme::Color::Black),
            _ => theme::Color::White,
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

/// Converts a theme color to RGB components.
#[allow(dead_code)]
fn color_to_rgb(color: &theme::Color) -> Result<(u8, u8, u8), String> {
    match color {
        theme::Color::Rgb(r, g, b) => Ok((*r, *g, *b)),
        _ => Err("Color must be RGB format".to_string()),
    }
}

/// Interpolates between two colors to create a gradient effect.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::theme::Color;
/// use groundhog_mcp::ui::components::header::interpolate_color;
/// 
/// let start = Color::Rgb(255, 0, 0);  // Red
/// let end = Color::Rgb(0, 0, 255);    // Blue
/// let mid = interpolate_color(&start, &end, 0.5);  // Purple
/// ```
pub fn interpolate_color(start: &theme::Color, end: &theme::Color, t: f64) -> theme::Color {
    match (start, end) {
        (theme::Color::Rgb(r1, g1, b1), theme::Color::Rgb(r2, g2, b2)) => {
            let r = interpolate_component(*r1, *r2, t as f32);
            let g = interpolate_component(*g1, *g2, t as f32);
            let b = interpolate_component(*b1, *b2, t as f32);
            theme::Color::Rgb(r, g, b)
        }
        _ => start.clone(),
    }
}

/// Interpolates between two color components.
/// 
/// # Examples
/// 
/// ```
/// use groundhog_mcp::ui::components::header::interpolate_component;
/// 
/// let start = 0;
/// let end = 255;
/// let mid = interpolate_component(start, end, 0.5);  // Gray component
/// assert!(mid >= start && mid <= end);
/// ```
pub fn interpolate_component(start: u8, end: u8, t: f32) -> u8 {
    // Apply gamma correction for perceptually uniform interpolation
    let start_f = (start as f32 / 255.0).powf(2.2);
    let end_f = (end as f32 / 255.0).powf(2.2);
    let interpolated = start_f + (end_f - start_f) * t;
    ((interpolated.powf(1.0 / 2.2) * 255.0) as u8).clamp(0, 255)
}

impl HeaderStyle {
    /// Creates a new header style with default settings.
    ///
    /// The default style uses:
    /// - Rounded box style
    /// - Left alignment
    /// - No padding
    /// - No gradient
    ///
    /// # Returns
    /// * `Self` - A new HeaderStyle instance with default settings
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            box_style: BoxStyle::Rounded,
            alignment: Alignment::Left,
            padding: 0,
            gradient: None,
        }
    }

    /// Sets the padding for the header.
    /// 
    /// # Arguments
    /// * `padding` - The padding value to set
    /// 
    /// # Returns
    /// * `Result<(), HeaderError>` - Ok if padding is valid, error otherwise
    pub fn set_padding(&mut self, padding: usize) -> Result<(), HeaderError> {
        if padding > Self::MAX_PADDING {
            return Err(HeaderError::InvalidWidth);
        }
        self.padding = padding;
        Ok(())
    }

    /// Maximum allowed padding value for the header.
    /// 
    /// This constant defines the maximum allowed padding value to prevent
    /// excessive padding that could cause display issues.
    const MAX_PADDING: usize = 100; // Reasonable maximum padding value
}

impl Default for HeaderStyle {
    /// Creates a default header style.
    ///
    /// This implementation calls `HeaderStyle::new()` to create a new
    /// instance with default settings.
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_width = self.calculate_total_width();
        let text_width = UnicodeWidthStr::width(&self.content[..]);
        let align_padding = self.calculate_alignment_padding(text_width, total_width);

        let (_h, _v, _tl, _tr, _bl, _br) = self.style.box_style.get_chars();

        // Draw top border
        write!(f, "{}", _tl)?;
        for _ in 0..total_width {
            write!(f, "{}", _h)?;
        }
        writeln!(f, "{}", _tr)?;

        // Draw content line
        write!(f, "{}", _v)?;
        write!(f, "{:width$}", "", width = align_padding)?;
        write!(f, "{}", self.content)?;
        write!(f, "{:width$}", "", width = total_width.saturating_sub(align_padding + text_width))?;
        writeln!(f, "{}", _v)?;

        // Draw bottom border
        write!(f, "{}", _bl)?;
        for _ in 0..total_width {
            write!(f, "{}", _h)?;
        }
        write!(f, "{}", _br)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Tests basic header formatting and border drawing
    #[test]
    fn test_header_formatting() {
        let mut buffer = Cursor::new(Vec::new());
        let header = Header::new("Test Header", "Test Header").unwrap();
        header.print_header(&mut buffer, "Test Header").unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        
        assert!(output.contains('╭'));
        assert!(output.contains('╮'));
        assert!(output.contains('╰'));
        assert!(output.contains('╯'));
        assert!(output.contains('│'));
        assert!(output.contains("Test Header"));
    }

    /// Tests all available box styles and their border characters
    #[test]
    fn test_box_styles() {
        let test_styles = vec![
            (BoxStyle::Double, ('╔', '╗', '╚', '╝', '═', '║')),
            (BoxStyle::Rounded, ('╭', '╮', '╰', '╯', '─', '│')),
            (BoxStyle::Sharp, ('┌', '┐', '└', '┘', '─', '│')),
            (BoxStyle::Hidden, (' ', ' ', ' ', ' ', ' ', ' ')),
        ];

        for (style, expected_chars) in test_styles {
            let mut buffer = Cursor::new(Vec::new());
            let header = Header::new("Test Border", "Test Border")
                .unwrap()
                .with_box_style(style)
                .with_padding(0);

            header.print_header(&mut buffer, "Test Border").unwrap();
            let output = String::from_utf8(buffer.into_inner()).unwrap();

            assert!(output.contains(expected_chars.0), "Missing top-left character");
            assert!(output.contains(expected_chars.1), "Missing top-right character");
            assert!(output.contains(expected_chars.2), "Missing bottom-left character");
            assert!(output.contains(expected_chars.3), "Missing bottom-right character");
            assert!(output.contains(expected_chars.4), "Missing horizontal character");
            assert!(output.contains(expected_chars.5), "Missing vertical character");
        }
    }

    /// Tests text alignment within the header
    #[test]
    fn test_alignment() {
        let mut buffer = Cursor::new(Vec::new());
        let text = "Centered";
        let padding = 4;
        let header = Header::new("Centered", "Centered")
            .unwrap()
            .with_padding(padding)
            .with_alignment(Alignment::Center);

        header.print_header(&mut buffer, text).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        
        let content_line = lines[1];
        let stripped_line = String::from_utf8(strip_ansi_escapes::strip(content_line)).unwrap();
        
        let content = stripped_line.trim_matches('│');
        let total_width = UnicodeWidthStr::width(content);
        let text_start = content.find(text).unwrap();
        let text_width = UnicodeWidthStr::width(text);
        
        let expected_center = (total_width - text_width) / 2;
        
        assert!((text_start as i32 - expected_center as i32).abs() <= 1,
            "Text not centered. Found at {}, expected around {}", text_start, expected_center);
    }

    /// Tests right alignment of text within the header
    #[test]
    fn test_right_alignment() {
        let mut buffer = Cursor::new(Vec::new());
        let text = "Right";
        let padding = 2;
        let header = Header::new("Right", "Right")
            .unwrap()
            .with_padding(padding)
            .with_alignment(Alignment::Right);

        header.print_header(&mut buffer, text).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        let content_line = lines[1];
        let stripped_line = String::from_utf8(strip_ansi_escapes::strip(content_line)).unwrap();
        
        let content = stripped_line.trim_matches('│');
        let text_width = UnicodeWidthStr::width(text);
        let total_width = content.len();
        
        let expected_left_padding = total_width - text_width - padding;
        let actual_left_padding = content.chars().take_while(|c| *c == ' ').count();
        
        assert_eq!(actual_left_padding, expected_left_padding,
            "Text not right-aligned. Expected {} spaces before text, found {}",
            expected_left_padding, actual_left_padding);
        
        let right_padding = content.chars().rev().take_while(|c| *c == ' ').count();
        assert_eq!(right_padding, padding,
            "Expected {} spaces after text, found {}",
            padding, right_padding);
    }

    /// Tests gradient color effects in the header
    #[test]
    fn test_gradient() {
        let mut buffer = Cursor::new(Vec::new());
        let header = Header::new("Gradient", "Gradient")
            .unwrap()
            .with_gradient(theme::Color::Rgb(255, 0, 0), theme::Color::Rgb(0, 0, 255))
            .unwrap()
            .with_padding(0);

        header.print_header(&mut buffer, "Gradient").unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        
        assert!(output.contains("\x1b[38;2;255;0;0m")); // Start color (red)
        assert!(output.contains("\x1b[38;2;0;0;255m")); // End color (blue)
        
        let color_codes: Vec<_> = output.match_indices("\x1b[38;2").collect();
        assert!(color_codes.len() >= 3, "Should have at least start, middle, and end colors");
    }

    /// Tests maximum width constraints
    #[test]
    fn test_max_width() {
        let mut header = Header::new("Test", "Test").unwrap();
        
        assert!(header.set_max_width(10).is_ok());
        
        let mut long_header = Header::new("Very Long Header Text", "Very Long Header Text").unwrap();
        assert!(matches!(
            long_header.set_max_width(10),
            Err(HeaderError::TextTooLong {
                text_length: _,
                max_width: 10
            })
        ));
    }

    /// Tests subtitle functionality
    #[test]
    fn test_subtitle() {
        let mut buffer = Cursor::new(Vec::new());
        let header = Header::new("Title", "Title")
            .unwrap()
            .with_subtitle("Subtitle");

        header.write(&mut buffer).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("Title - Subtitle"));
    }

    /// Tests theme application to headers
    #[test]
    fn test_theme_application() {
        let mut header = Header::new("Test Header", "Test Header").unwrap();

        let theme = Theme {
            name: "test".to_string(),
            colors: crate::ui::theme::ColorScheme {
                primary: theme::Color::Red,
                secondary: theme::Color::Cyan,
                background: theme::Color::Black,
                foreground: theme::Color::White,
                accent: theme::Color::Yellow,
                error: theme::Color::Red,
                warning: theme::Color::DarkYellow,
                success: theme::Color::Green,
            },
            styles: crate::ui::theme::StyleSet {
                header: Style::new().bold().underlined(),
                text: Style::new(),
                input: Style::new(),
                button: Style::new(),
                dialog: Style::new(),
            },
            metadata: crate::ui::theme::ThemeMetadata {
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: "Test theme".to_string(),
            },
        };

        assert!(header.apply_theme(&theme).is_ok());
        assert!(header.get_style().attributes.contains(&crate::ui::theme::Attribute::Bold));
        assert!(header.get_style().attributes.contains(&crate::ui::theme::Attribute::Underlined));
    }

    /// Tests color interpolation functionality
    #[test]
    fn test_color_interpolation() {
        let start = theme::Color::Rgb(255, 0, 0);
        let end = theme::Color::Rgb(0, 0, 255);
        let mid = interpolate_color(&start, &end, 0.5);
        
        if let theme::Color::Rgb(r, g, b) = mid {
            assert!(r > 0 && r < 255);
            assert_eq!(g, 0);
            assert!(b > 0 && b < 255);
        } else {
            panic!("Expected RGB color");
        }
    }

    /// Tests ANSI color conversion
    #[test]
    fn test_ansi_color_conversion() {
        let color = theme::Color::Rgb(255, 0, 0); // Red in basic ANSI colors
        let (r, g, b) = color_to_rgb(&color).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    /// Tests header layout calculation
    #[test]
    fn test_header_layout() -> Result<(), LayoutError> {
        let mut header = Header::new("test_header", "Test Content")?;
        let size = Size::new(100, 1);
        let result = header.calculate_layout(size)?;
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 1);
        Ok(())
    }

    /// Tests box drawing functionality
    #[test]
    fn test_box_drawing() {
        let header = Header::new("Test", "Test")
            .unwrap()
            .with_box_style(BoxStyle::Sharp);
        
        let (_h, _v, _tl, _tr, _bl, _br) = header.style.box_style.get_chars();
        
        let output = header.to_string();
        let lines: Vec<&str> = output.lines().collect();
        
        // Check corners
        let top_line = lines[0];
        let top_chars: Vec<char> = top_line.chars().collect();
        assert_eq!(top_chars[0].to_string(), "┌", "Top-left corner should match");
        assert_eq!(top_chars.last().unwrap().to_string(), "┐", "Top-right corner should match");
        
        // Check middle lines
        for line in &lines[1..lines.len() - 1] {
            let chars: Vec<char> = line.chars().collect();
            assert_eq!(chars[0].to_string(), "│", "Left border should be vertical line");
            assert_eq!(chars.last().unwrap().to_string(), "│", "Right border should be vertical line");
        }
        
        // Check bottom corners
        let bottom_line = lines[lines.len() - 1];
        let bottom_chars: Vec<char> = bottom_line.chars().collect();
        assert_eq!(bottom_chars[0].to_string(), "└", "Bottom-left corner should match");
        assert_eq!(bottom_chars.last().unwrap().to_string(), "┘", "Bottom-right corner should match");
        
        // Check horizontal lines
        assert!(top_chars[1..top_chars.len() - 1].iter().all(|c| c.to_string() == "─"),
            "Top line should contain horizontal lines");
        assert!(bottom_chars[1..bottom_chars.len() - 1].iter().all(|c| c.to_string() == "─"),
            "Bottom line should contain horizontal lines");
    }
} 