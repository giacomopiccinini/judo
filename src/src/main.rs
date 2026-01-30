//! Judo - A terminal-based todo list application

use color_eyre::Result;
use judo::app::App;

/// Application entry point
///
/// Initializes the terminal, creates the application instance, runs the main loop,
/// and properly restores the terminal on exit.
#[tokio::main]
async fn main() -> Result<()> {
    // Set the terminal up
    let mut terminal = ratatui::init();

    // Set up the app
    let app = App::new().await;

    // Create and run the app
    let app_result = app.run(&mut terminal).await;

    // Restore terminal to original state
    ratatui::restore();

    app_result
}
