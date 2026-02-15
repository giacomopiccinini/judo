//! Judo - A terminal-based todo list application
use anyhow::{Context, Result};
use clap::Parser;
use judo::{
    app::App,
    cli::{
        args::{Cli, Commands, DbCommands, ItemCommands, ListCommands},
        ops,
    },
};

/// Application entry point
///
/// Initializes the terminal, creates the application instance, runs the main loop,
/// and properly restores the terminal on exit.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up the app
    let app = App::new().await;

    // Handle CLI arguments
    match cli.command {
        //Database commands
        Some(Commands::Dbs { command }) => match command {
            Some(DbCommands::Show) => {
                ops::list_dbs(&app).with_context(|| "Failed to list databases")?;
            }
            Some(DbCommands::Add { name }) => {
                ops::add_db(app, name)
                    .await
                    .with_context(|| "Failed to add database")?;
                return Ok(());
            }
            None => {}
        },
        //List commands
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Show { name }) => {
                ops::list_lists(&app, name)
                    .await
                    .with_context(|| "Failed to list to-do lists")?;
                return Ok(());
            }
            Some(ListCommands::Add { name, db }) => {
                ops::add_list(&app, name, &db)
                    .await
                    .with_context(|| "Failed to add to-do list")?;
                return Ok(());
            }
            Some(ListCommands::Delete { name, id, db }) => {
                ops::delete_list(&app, name, id, &db)
                    .await
                    .with_context(|| "Failed to delete to-do list")?;
                return Ok(());
            }
            None => {}
        },
        //Item commands
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Show) => {
                ops::list_items(&app)
                    .await
                    .with_context(|| "Failed to list to-do items")?;
                return Ok(());
            }
            Some(ItemCommands::Add {
                name,
                db,
                list_name,
                list_id,
            }) => {
                ops::add_item(&app, name, &db, list_id, list_name)
                    .await
                    .with_context(|| "Failed to add to-do item")?;
                return Ok(());
            }
            Some(ItemCommands::Delete { id, db }) => {
                ops::delete_item(&app, id, &db)
                    .await
                    .with_context(|| "Failed to delete to-do item")?;
                return Ok(());
            }
            Some(ItemCommands::ToggleDone { id, db }) => {
                ops::toggle_done_item(&app, id, &db)
                    .await
                    .with_context(|| "Failed to toggle to-do item status")?;
                return Ok(());
            }
            None => {}
        },
        None => {}
    }

    // Set the terminal up
    let mut terminal = ratatui::init();

    // Create and run the app
    let app_result = app.run(&mut terminal).await;

    // Restore terminal to original state
    ratatui::restore();

    app_result
}
