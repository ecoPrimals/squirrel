use std::io;
use std::sync::Arc;
use std::time::Duration;

use crossterm::{
    event::{Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, Terminal};
use dashboard_core::service::DashboardService;

pub mod app;
pub mod error;
pub mod ui;
pub mod widgets;
pub mod util;
pub mod input;

/// Run the terminal UI
pub async fn run_ui<S>(
    service: Arc<S>,
    ui_tick_rate: Duration,
    data_tick_rate: Duration,
) -> anyhow::Result<()>
where
    S: DashboardService + Send + Sync + 'static + ?Sized,
{
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run it
    let mut app = app::App::new(service);
    let mut last_update = std::time::Instant::now();
    
    loop {
        // Update terminal UI
        terminal.draw(|frame| {
            ui::render::<CrosstermBackend<io::Stdout>, _>(&app, frame)
        })?;

        // Handle events
        if input::poll(ui_tick_rate)? {
            if let Event::Key(key) = input::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    // Check for quit
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    
                    // Let the app handle the key event
                    app.handle_key_event(key);
                }
            }
        }
        
        // Check if we need to update data
        let now = std::time::Instant::now();
        if now.duration_since(last_update) >= data_tick_rate {
            app.update_data().await?;
            last_update = now;
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    
    Ok(())
}

/// Run the AI chat terminal UI
/// 
/// This function implements a specialized AI chat UI using the chat-specific components.
pub async fn run_ai_chat_ui<S>(
    service: Arc<S>,
    ui_tick_rate: Duration,
    data_tick_rate: Duration,
) -> anyhow::Result<()>
where
    S: DashboardService + Send + Sync + 'static + ?Sized,
{
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create chat app and run it
    let mut app = app::chat::ChatApp::new(service);
    let mut last_update = std::time::Instant::now();
    
    loop {
        // Process any received messages from async tasks - do this exactly once per frame
        app.process_received_messages();
        
        // Update terminal UI
        terminal.draw(|frame| {
            ui::chat::render::<CrosstermBackend<io::Stdout>, _>(frame, &app)
        })?;

        // Handle events - this section is crucial
        if input::poll(ui_tick_rate)? {
            if let Event::Key(key) = input::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    // Check for quit ONLY if in normal mode
                    if key.code == KeyCode::Char('q') && should_process_as_global_command(&app.state.input_mode, key.code) {
                        break;
                    }
                    
                    // Let the app handle the key event - this handles all keys including 'q' when in edit mode
                    app.handle_key_event(key);
                    
                    // If app should quit (due to any command), break the loop
                    if app.should_quit() {
                        break;
                    }
                }
            }
        }
        
        // Handle data updates on a different interval
        if last_update.elapsed() >= data_tick_rate {
            last_update = std::time::Instant::now();
            
            if let Err(e) = app.update_connection_status().await {
                eprintln!("Error updating connection status: {}", e);
            }
        }
    }
    
    // Cleanup and restore terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    Ok(())
}

// Helper function to determine if a key event should be processed globally or as text input
pub fn should_process_as_global_command(input_mode: &widgets::chat::InputMode, key: crossterm::event::KeyCode) -> bool {
    match input_mode {
        widgets::chat::InputMode::Normal => true,
        widgets::chat::InputMode::Editing => {
            // In editing mode, only specific keys should be processed globally
            // Everything else should be treated as text input
            matches!(key, 
                crossterm::event::KeyCode::Esc |
                crossterm::event::KeyCode::Enter |
                crossterm::event::KeyCode::Tab |
                crossterm::event::KeyCode::BackTab |
                crossterm::event::KeyCode::Home |
                crossterm::event::KeyCode::End
            )
        }
    }
}
