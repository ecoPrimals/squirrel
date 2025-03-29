pub mod app;
pub mod error;
pub mod event;
pub mod ui;
pub mod util;
pub mod widgets;

// Re-export main run function and error type
pub use error::{Error, Result};

use dashboard_core::service::DashboardService;
use ratatui::{backend::Backend, Terminal};
use std::sync::Arc;
use std::time::Duration;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;

/// Main function to run the terminal UI.
pub async fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    provider: Arc<dyn DashboardService + Send + Sync + 'static>,
    tick_rate: Duration,
    update_rate: Duration,
) -> Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    // Optionally clear the screen: terminal.clear()?;

    // Create app and event handler
    let mut app = app::App::new(provider.clone());
    let mut event_handler = event::EventHandler::new(tick_rate);
    let mut last_update_time = std::time::Instant::now();

    // Main loop
    loop {
        if app.state.should_quit {
            break;
        }

        // Render the UI
        terminal.draw(|frame| {
            ui::render::<B>(&mut app, frame)
        })?;

        // Handle events
        match event_handler.next()? {
            event::Event::Tick => app.on_tick(),
            event::Event::Key(key) => app.on_key(key),
            event::Event::Mouse(_) => { /* Ignore mouse events for now */ }
            event::Event::Resize(_, _) => { /* Ignore resize events for now */ }
        }

        // Trigger data update based on update_rate
        if last_update_time.elapsed() >= update_rate {
            app.update().await;
            last_update_time = std::time::Instant::now();
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
} 