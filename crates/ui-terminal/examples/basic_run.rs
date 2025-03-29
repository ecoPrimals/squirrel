use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dashboard_core::service::DashboardService;
use dashboard_core::service::DefaultDashboardService;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, sync::Arc, time::Duration};
use ui_terminal; // Use the crate name

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the dashboard service instance wrapped in Arc
    let dashboard_service: Arc<dyn DashboardService + Send + Sync + 'static> = DefaultDashboardService::default();

    // Define update rates
    let tick_rate = Duration::from_millis(250); // e.g., 4 Hz TUI updates
    let data_update_rate = Duration::from_secs(2); // Fetch new data every 2 seconds

    // Run the UI, passing the Arc<dyn DashboardService>
    // No generics needed in the call anymore
    let res = ui_terminal::run_ui(&mut terminal, dashboard_service, tick_rate, data_update_rate).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running UI: {:?}", err);
        return Err(err.into());
    }

    Ok(())
} 