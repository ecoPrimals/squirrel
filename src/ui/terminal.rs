use std::fmt;
use std::io::{self, Stdout};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal as RatatuiTerminal;
use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::core::error::Result;
use crate::ui::components::{Component, ComponentRegistry};
use crate::ui::events::Event;
use crate::ui::theme::Theme;

pub struct TerminalUI {
    terminal: RatatuiTerminal<CrosstermBackend<Stdout>>,
    registry: ComponentRegistry,
    theme: Theme,
}

impl TerminalUI {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = RatatuiTerminal::new(backend)?;
        let registry = ComponentRegistry::new();
        let theme = Theme::default();

        Ok(Self {
            terminal,
            registry,
            theme,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.size();
            for component in self.registry.components() {
                if let Err(e) = component.render(frame, area) {
                    eprintln!("Error rendering component: {}", e);
                }
            }
        })?;
        Ok(())
    }

    pub fn handle_event(&mut self, event: Box<dyn Event>) -> Result<bool> {
        match event {
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
            }
            _ => {}
        }

        for component in self.registry.components_mut() {
            if let Err(e) = component.handle_event(event) {
                eprintln!("Error handling event: {}", e);
            }
        }

        Ok(false)
    }

    pub fn register_component(&mut self, component: Box<dyn Component>) -> Result<()> {
        self.registry.register(component)
    }
} 