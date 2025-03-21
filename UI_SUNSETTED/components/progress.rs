use std::{
    io::{self, Write},
    time::{Duration, Instant},
};
use crossterm::{
    cursor,
    style::self,
    terminal,
    QueueableCommand,
};

use crate::ui::{
    layout::LayoutManager,
    theme::{Color, ColorRole, Style, Theme, ThemeError, Themeable},
};

/// The number of progress updates to keep in history for speed calculations.
pub const SPEED_WINDOW_SIZE: usize = 10;

/// The characters used for the spinning animation in progress indicators.
pub const SPINNER_CHARS: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

/// Style configuration for progress bars.
pub struct ProgressStyle {
    /// The base style for the progress bar.
    pub style: Style,
    /// The character used for the filled portion of the progress bar.
    pub filled_char: char,
    /// The character used for the empty portion of the progress bar.
    pub empty_char: char,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            style: Style::default(),
            filled_char: '█',
            empty_char: '░',
        }
    }
}

/// A progress bar component that tracks and displays progress information.
///
/// This component maintains a history of progress updates to calculate speed
/// and estimated time remaining. It supports customizable styling and can
/// display both determinate and indeterminate progress.
pub struct Progress {
    /// The total size or number of items to process.
    total_size: usize,
    /// The current progress value.
    current_progress: usize,
    /// History of progress updates with timestamps for speed calculation.
    progress_history: Vec<(Instant, usize)>,
    /// The style configuration for the progress bar.
    style: ProgressStyle,
    /// The layout manager for positioning the progress bar.
    layout: LayoutManager,
}

impl Progress {
    /// Creates a new progress bar with the specified total size.
    pub fn new(total: usize) -> Self {
        Self {
            current_progress: 0,
            total_size: total,
            style: ProgressStyle::default(),
            layout: LayoutManager::new(),
            progress_history: Vec::with_capacity(SPEED_WINDOW_SIZE),
        }
    }

    /// Writes the current progress bar state to the given writer.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.queue(cursor::MoveRight(self.layout.get_current_indentation() as u16))?;
        let filled = (50_f64 * (self.current_progress as f64 / self.total_size as f64)) as usize;
        let empty = 50 - filled;

        // Write filled section
        if let Some(color) = &self.style.style.foreground {
            writer.queue(style::SetForegroundColor(color.clone().into()))?;
        }
        for _ in 0..filled {
            writer.write_all(self.style.filled_char.to_string().as_bytes())?;
        }

        // Write empty section
        if let Some(color) = &self.style.style.background {
            writer.queue(style::SetForegroundColor(color.clone().into()))?;
        }
        for _ in 0..empty {
            writer.write_all(self.style.empty_char.to_string().as_bytes())?;
        }

        // Reset color
        writer.queue(style::ResetColor)?;

        // Write percentage
        if let Some(color) = &self.style.style.foreground {
            writer.queue(style::SetForegroundColor(color.clone().into()))?;
        }
        write!(writer, " {}%", ((self.current_progress as f64 / self.total_size as f64) * 100.0) as u32)?;

        Ok(())
    }

    /// Updates the current progress value and maintains the progress history.
    pub fn update_progress(&mut self, current: usize) {
        self.current_progress = current;
        self.progress_history.push((Instant::now(), current));
        if self.progress_history.len() > SPEED_WINDOW_SIZE {
            self.progress_history.remove(0);
        }
    }

    /// Calculates the estimated time remaining and current speed based on progress history.
    ///
    /// Returns `None` if there isn't enough history to calculate these values.
    pub fn calculate_eta_and_speed(&self) -> Option<(Duration, f64)> {
        if self.progress_history.len() < 2 {
            return None;
        }

        let (start_time, start_progress) = self.progress_history.first()?;
        let (end_time, end_progress) = self.progress_history.last()?;
        let elapsed = end_time.duration_since(*start_time);
        let progress_delta = *end_progress as f64 - *start_progress as f64;

        if progress_delta <= 0.0 || elapsed.as_secs_f64() <= 0.0 {
            return None;
        }

        let speed = progress_delta / elapsed.as_secs_f64();
        let remaining_progress = self.total_size as f64 - *end_progress as f64;
        let eta = Duration::from_secs_f64(remaining_progress / speed);

        Some((eta, speed))
    }

    /// Prints the progress bar with a message and current progress percentage.
    pub fn print_progress<W: Write>(&mut self, mut writer: W, message: &str, progress: f64) -> io::Result<()> {
        let progress = progress.clamp(0.0, 1.0);
        let current = (progress * self.total_size as f64) as usize;
        self.update_progress(current);

        // Get the current spinner character
        let spinner_index = (Instant::now().elapsed().as_millis() / 100) as usize % SPINNER_CHARS.len();
        let spinner = SPINNER_CHARS[spinner_index];

        writer.queue(cursor::MoveRight(self.layout.get_current_indentation() as u16))?;
        write!(writer, "{} {} ", spinner, message)?;
        self.write(&mut writer)?;

        if let Some((eta, speed)) = self.calculate_eta_and_speed() {
            write!(writer, " ({:.1} items/s, ETA: {:.1}s)", speed, eta.as_secs_f64())?;
        }

        writer.write_all(b"\n")?;
        Ok(())
    }

    /// Clears the progress bar from the display.
    pub fn clear_progress<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        writer.queue(cursor::MoveToColumn(0))?;
        Ok(())
    }
}

impl Themeable for Progress {
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
        self.style.style = theme.styles.text.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::Duration;

    #[test]
    fn test_progress_display() {
        let mut progress = Progress::new(100);
        let mut buffer = Cursor::new(Vec::new());

        progress.update_progress(50);
        progress.write(&mut buffer).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("50%"));
        assert!(output.contains('█'));
        assert!(output.contains('░'));
    }

    #[test]
    fn test_progress_bounds() {
        let mut progress = Progress::new(100);
        let mut buffer = Cursor::new(Vec::new());

        // Test with 0%
        progress.update_progress(0);
        progress.write(&mut buffer).unwrap();
        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("0%"));

        // Test with 100%
        let mut buffer = Cursor::new(Vec::new());
        progress.update_progress(100);
        progress.write(&mut buffer).unwrap();
        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(output.contains("100%"));
    }

    #[test]
    fn test_progress_speed() {
        let mut progress = Progress::new(100);
        
        // Update progress a few times
        progress.update_progress(0);
        std::thread::sleep(Duration::from_millis(10));
        progress.update_progress(50);
        std::thread::sleep(Duration::from_millis(10));
        progress.update_progress(100);

        if let Some((eta, speed)) = progress.calculate_eta_and_speed() {
            assert!(speed > 0.0);
            assert!(eta.as_secs_f64() >= 0.0);
        }
    }
} 