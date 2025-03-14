use crossterm::{
    cursor,
    QueueableCommand,
    style::{Color, SetForegroundColor, ResetColor},
    event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture},
    terminal::{enable_raw_mode, disable_raw_mode},
};
use std::io::{self, Write};

use crate::ui::layout::LayoutManager;
use crate::ui::theme::{Theme, Themeable, ColorRole, ThemeError, Style};

#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Confirm,
}

#[derive(Debug)]
pub struct Dialog {
    layout: LayoutManager,
    style: Style,
    dialog_type: DialogType,
    title: String,
    message: String,
    buttons: Vec<String>,
    selected_button: usize,
}

impl Dialog {
    pub fn new(dialog_type: DialogType, title: String, message: String) -> Self {
        let buttons = match dialog_type.clone() {
            DialogType::Info => vec!["OK".to_string()],
            DialogType::Warning => vec!["OK".to_string()],
            DialogType::Error => vec!["OK".to_string()],
            DialogType::Confirm => vec!["Yes".to_string(), "No".to_string()],
        };

        Dialog {
            layout: LayoutManager::new(),
            style: Style::new(),
            dialog_type,
            title,
            message,
            buttons,
            selected_button: 0,
        }
    }

    pub fn with_buttons(mut self, buttons: Vec<String>) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn with_selected_button(mut self, selected: usize) -> Self {
        if selected < self.buttons.len() {
            self.selected_button = selected;
        }
        self
    }

    pub fn show<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        // Calculate dialog dimensions
        let width = self.calculate_width();
        let (border_top, border_bottom) = self.draw_borders(width);

        // Enable raw mode and mouse capture
        enable_raw_mode()?;
        writer.queue(EnableMouseCapture)?;

        // Store initial button positions for mouse interaction
        let button_positions = self.calculate_button_positions();

        // Draw dialog
        self.draw_dialog(writer, width, &border_top, &border_bottom)?;

        // Handle input
        let result = self.handle_input(writer, &button_positions)?;

        // Disable raw mode and mouse capture
        writer.queue(DisableMouseCapture)?;
        disable_raw_mode()?;

        Ok(result)
    }

    fn draw_dialog<W: Write>(&self, writer: &mut W, width: usize, border_top: &str, border_bottom: &str) -> io::Result<()> {
        // Draw dialog box
        self.layout.write_indentation(writer)?;
        writer.queue(SetForegroundColor(self.get_dialog_color()))?;
        writeln!(writer, "{}", border_top)?;

        // Draw title
        self.layout.write_indentation(writer)?;
        write!(writer, "│ ")?;
        writer.queue(SetForegroundColor(self.get_dialog_color()))?;
        write!(writer, "{:^width$}", self.title, width = width - 4)?;
        writeln!(writer, " │")?;

        // Draw separator
        self.layout.write_indentation(writer)?;
        writeln!(writer, "├{}┤", "─".repeat(width - 2))?;

        // Draw message
        for line in self.message.lines() {
            self.layout.write_indentation(writer)?;
            write!(writer, "│ ")?;
            writer.queue(ResetColor)?;
            write!(writer, "{:width$}", line, width = width - 4)?;
            writer.queue(SetForegroundColor(self.get_dialog_color()))?;
            writeln!(writer, " │")?;
        }

        // Draw button separator
        self.layout.write_indentation(writer)?;
        writeln!(writer, "├{}┤", "─".repeat(width - 2))?;

        // Draw buttons
        self.draw_buttons(writer, width)?;

        // Draw bottom border
        self.layout.write_indentation(writer)?;
        writeln!(writer, "{}", border_bottom)?;
        writer.queue(ResetColor)?;
        writer.flush()?;

        Ok(())
    }

    fn draw_buttons<W: Write>(&self, writer: &mut W, width: usize) -> io::Result<()> {
        self.layout.write_indentation(writer)?;
        write!(writer, "│ ")?;
        for (i, button) in self.buttons.iter().enumerate() {
            if i == self.selected_button {
                writer.queue(SetForegroundColor(Color::Black))?;
                writer.queue(crossterm::style::SetBackgroundColor(self.get_dialog_color()))?;
            }
            write!(writer, " {} ", button)?;
            if i == self.selected_button {
                writer.queue(ResetColor)?;
                writer.queue(SetForegroundColor(self.get_dialog_color()))?;
            }
            if i < self.buttons.len() - 1 {
                write!(writer, " | ")?;
            }
        }
        write!(writer, "{:width$}", "", width = width - self.calculate_buttons_width() - 3)?;
        writeln!(writer, "│")?;
        Ok(())
    }

    fn calculate_button_positions(&self) -> Vec<(usize, usize, usize)> {
        let mut positions = Vec::new();
        let mut current_pos = 3; // Start after "│ "

        for (i, button) in self.buttons.iter().enumerate() {
            let button_width = button.len() + 2; // Add 2 for padding
            positions.push((i, current_pos, current_pos + button_width));
            current_pos += button_width + 3; // Add 3 for " | " separator
        }

        positions
    }

    pub fn handle_input<W: Write>(&self, writer: &mut W, button_positions: &[(usize, usize, usize)]) -> io::Result<usize> {
        let mut current_selection = self.selected_button;

        loop {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Enter => break,
                        KeyCode::Left if current_selection > 0 => {
                            current_selection -= 1;
                            self.redraw_buttons(writer, current_selection)?;
                        }
                        KeyCode::Right if current_selection < self.buttons.len() - 1 => {
                            current_selection += 1;
                            self.redraw_buttons(writer, current_selection)?;
                        }
                        KeyCode::Tab => {
                            current_selection = if key.modifiers.contains(KeyModifiers::SHIFT) {
                                if current_selection == 0 {
                                    self.buttons.len() - 1
                                } else {
                                    current_selection - 1
                                }
                            } else {
                                (current_selection + 1) % self.buttons.len()
                            };
                            self.redraw_buttons(writer, current_selection)?;
                        }
                        KeyCode::Esc => {
                            current_selection = self.buttons.len() - 1;
                            break;
                        }
                        KeyCode::Char(c) => match c {
                            'y' | 'Y' if self.dialog_type == DialogType::Confirm => {
                                current_selection = 0;
                                break;
                            }
                            'n' | 'N' if self.dialog_type == DialogType::Confirm => {
                                current_selection = 1;
                                break;
                            }
                            'o' | 'O' if self.buttons.contains(&"OK".to_string()) => {
                                current_selection = self.buttons.iter().position(|b| b == "OK").unwrap();
                                break;
                            }
                            'c' | 'C' if self.buttons.contains(&"Cancel".to_string()) => {
                                current_selection = self.buttons.iter().position(|b| b == "Cancel").unwrap();
                                break;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                Event::Mouse(MouseEvent { kind, column, row: _, modifiers: _ }) => {
                    match kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            // Check if click is on a button
                            if let Some((index, _, _)) = button_positions.iter()
                                .find(|(_, start, end)| column >= *start as u16 && column < *end as u16) {
                                current_selection = *index;
                                self.redraw_buttons(writer, current_selection)?;
                                break;
                            }
                        }
                        MouseEventKind::Moved => {
                            // Highlight button on hover
                            if let Some((index, _, _)) = button_positions.iter()
                                .find(|(_, start, end)| column >= *start as u16 && column < *end as u16) {
                                if current_selection != *index {
                                    current_selection = *index;
                                    self.redraw_buttons(writer, current_selection)?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(current_selection)
    }

    fn calculate_width(&self) -> usize {
        let title_width = self.title.len();
        let message_width = self.message.lines().map(|line| line.len()).max().unwrap_or(0);
        let buttons_width = self.calculate_buttons_width();
        [title_width, message_width, buttons_width].iter().max().unwrap() + 6
    }

    fn calculate_buttons_width(&self) -> usize {
        let button_texts: usize = self.buttons.iter().map(|b| b.len() + 2).sum();
        let separators = if self.buttons.len() > 1 {
            (self.buttons.len() - 1) * 3
        } else {
            0
        };
        button_texts + separators
    }

    fn get_dialog_color(&self) -> Color {
        match self.dialog_type {
            DialogType::Info => Color::Blue,
            DialogType::Warning => Color::Yellow,
            DialogType::Error => Color::Red,
            DialogType::Confirm => Color::Green,
        }
    }

    fn redraw_buttons<W: Write>(&self, writer: &mut W, selected: usize) -> io::Result<()> {
        // Save cursor position and move to buttons line
        writer.queue(cursor::SavePosition)?;
        writer.queue(cursor::MoveUp(2))?;
        
        // Clear the button line
        self.layout.write_indentation(writer)?;
        write!(writer, "│ ")?;
        writer.queue(SetForegroundColor(self.get_dialog_color()))?;
        
        // Redraw buttons
        for (i, button) in self.buttons.iter().enumerate() {
            if i == selected {
                writer.queue(SetForegroundColor(Color::Black))?;
                writer.queue(crossterm::style::SetBackgroundColor(self.get_dialog_color()))?;
            }
            write!(writer, " {} ", button)?;
            if i == selected {
                writer.queue(ResetColor)?;
                writer.queue(SetForegroundColor(self.get_dialog_color()))?;
            }
            if i < self.buttons.len() - 1 {
                write!(writer, " | ")?;
            }
        }
        
        write!(writer, "{:width$}", "", width = self.calculate_width() - self.calculate_buttons_width() - 3)?;
        writeln!(writer, "│")?;
        
        // Restore cursor position
        writer.queue(cursor::RestorePosition)?;
        writer.flush()?;
        Ok(())
    }

    fn draw_borders(&self, width: usize) -> (String, String) {
        let border_top = "┌".to_owned() + &"─".repeat(width - 2) + "┐";
        let border_bottom = "└".to_owned() + &"─".repeat(width - 2) + "┘";
        (border_top, border_bottom)
    }
}

impl Themeable for Dialog {
    fn apply_theme(&mut self, theme: &Theme) -> Result<(), ThemeError> {
        self.style = theme.styles.dialog.clone();
        Ok(())
    }

    fn get_style(&self) -> &Style {
        &self.style
    }

    fn get_color(&self, role: ColorRole) -> crate::ui::theme::Color {
        match (self.dialog_type.clone(), role) {
            (DialogType::Info, _) => crate::ui::theme::Color::Blue,
            (DialogType::Warning, _) => crate::ui::theme::Color::Yellow,
            (DialogType::Error, _) => crate::ui::theme::Color::Red,
            (DialogType::Confirm, _) => crate::ui::theme::Color::Green,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_creation() {
        let dialog = Dialog::new(
            DialogType::Info,
            "Test Title".to_string(),
            "Test Message".to_string(),
        );
        
        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.message, "Test Message");
        assert_eq!(dialog.buttons.len(), 1);
        assert_eq!(dialog.buttons[0], "OK");
    }

    #[test]
    fn test_confirm_dialog() {
        let dialog = Dialog::new(
            DialogType::Confirm,
            "Confirm".to_string(),
            "Are you sure?".to_string(),
        );
        
        assert_eq!(dialog.buttons.len(), 2);
        assert_eq!(dialog.buttons[0], "Yes");
        assert_eq!(dialog.buttons[1], "No");
    }

    #[test]
    fn test_custom_buttons() {
        let dialog = Dialog::new(
            DialogType::Info,
            "Custom".to_string(),
            "Message".to_string(),
        ).with_buttons(vec!["Custom1".to_string(), "Custom2".to_string()]);
        
        assert_eq!(dialog.buttons.len(), 2);
        assert_eq!(dialog.buttons[0], "Custom1");
        assert_eq!(dialog.buttons[1], "Custom2");
    }

    #[test]
    fn test_theme_application() {
        let mut dialog = Dialog::new(
            DialogType::Info,
            "Test".to_string(),
            "Message".to_string(),
        );

        let theme = Theme {
            name: "test".to_string(),
            colors: crate::ui::theme::ColorScheme {
                primary: crate::ui::theme::Color::Blue,
                secondary: crate::ui::theme::Color::Cyan,
                background: crate::ui::theme::Color::Black,
                foreground: crate::ui::theme::Color::White,
                accent: crate::ui::theme::Color::Yellow,
                error: crate::ui::theme::Color::Red,
                warning: crate::ui::theme::Color::DarkYellow,
                success: crate::ui::theme::Color::Green,
            },
            styles: crate::ui::theme::StyleSet {
                header: Style::new(),
                text: Style::new(),
                input: Style::new(),
                button: Style::new(),
                dialog: Style::new().bold(),
            },
            metadata: crate::ui::theme::ThemeMetadata {
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                description: "Test theme".to_string(),
            },
        };

        assert!(dialog.apply_theme(&theme).is_ok());
        assert!(dialog.get_style().attributes.contains(&crate::ui::theme::Attribute::Bold));
    }

    #[test]
    fn test_button_selection() {
        let dialog = Dialog::new(
            DialogType::Confirm,
            "Test".to_string(),
            "Message".to_string(),
        ).with_selected_button(1);
        
        assert_eq!(dialog.selected_button, 1);
    }

    #[test]
    fn test_invalid_button_selection() {
        let dialog = Dialog::new(
            DialogType::Info,
            "Test".to_string(),
            "Message".to_string(),
        ).with_selected_button(5); // Invalid index
        
        assert_eq!(dialog.selected_button, 0); // Should remain at default
    }

    #[test]
    fn test_button_positions() {
        let dialog = Dialog::new(
            DialogType::Confirm,
            "Test".to_string(),
            "Message".to_string(),
        );
        
        let positions = dialog.calculate_button_positions();
        assert_eq!(positions.len(), 2); // Yes and No buttons
        
        // Check Yes button position
        assert_eq!(positions[0].0, 0); // First button
        assert_eq!(positions[0].1, 3); // Starts after "│ "
        assert_eq!(positions[0].2, 8); // "Yes" + padding
        
        // Check No button position
        assert_eq!(positions[1].0, 1); // Second button
        assert_eq!(positions[1].1, 11); // After first button and separator
        assert_eq!(positions[1].2, 15); // "No" + padding
    }
} 