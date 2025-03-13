use groundhog_mcp::ui::components::App;
use std::error::Error;
use crossterm::{
    event::{self, Event},
    terminal::{enable_raw_mode, disable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new("Groundhog MCP Demo".to_string());
    app.indicators.push("Ready".to_string());
    app.indicators.push("Press 'q' to quit".to_string());

    // Run the event loop
    loop {
        // Draw the user interface
        terminal.draw(|f| app.render(f))?;

        // Handle events
        if let Event::Key(key) = event::read()? {
            app.on_key(key);
            if app.should_quit {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
} 