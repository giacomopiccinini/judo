use std::io::Write;

use crate::app::App;
use crate::db::config::DBConfig;
use crate::db::connections::{self, get_db_pool};
use crate::db::models::{NewTodoItem, NewTodoList, TodoItem, TodoList};
use anyhow::{Context, Result};
use sqlx::{Pool, Sqlite};
use tabwriter::TabWriter;

/// Database operations

/// Lists all configured databases in a formatted table
///
/// Displays database names and their connection strings in a tabular format
pub fn list_dbs(app: &App) -> Result<()> {
    let db_list = &app.config.dbs;

    let mut tw = TabWriter::new(vec![]);
    writeln!(tw, "Name\tConnection string").with_context(|| "Failed to write table header")?;
    writeln!(tw, "----\t-----------------").with_context(|| "Failed to write table separator")?;
    for db in db_list {
        writeln!(tw, "{}\t{}", db.name, db.connection_str)
            .with_context(|| format!("Failed to write database entry for '{}'", db.name))?;
    }
    tw.flush().with_context(|| "Failed to flush table writer")?;
    let output = String::from_utf8(
        tw.into_inner()
            .with_context(|| "Failed to get table writer buffer")?,
    )
    .with_context(|| "Failed to convert table output to string")?;
    print!("{output}");
    Ok(())
}

/// Creates a new database with the given name
pub async fn add_db(mut app: App, name: String) -> Result<()> {
    app.create_new_database(name, false)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

/// List operations

/// Lists all todo lists across all configured databases
///
/// Displays lists in a formatted table showing:
/// - List name and ID
/// - Database name
/// - Number of items in each list
pub async fn list_lists(app: &App, name: Option<String>) -> Result<()> {
    // Filter dbs if name is passed
    let dbs: Vec<_> = app
        .config
        .dbs
        .iter()
        .filter(|db| name.as_ref().map_or(true, |n| &db.name == n))
        .collect();

    let mut tw = TabWriter::new(vec![]);
    writeln!(tw, "Name\tID\tDB\tNo of items").with_context(|| "Failed to write table header")?;
    writeln!(tw, "----\t--\t--\t-----------").with_context(|| "Failed to write table separator")?;

    // Iterate through all databases
    for db in dbs {
        let db_pool = connections::get_db_pool(db.connection_str.as_str())
            .await
            .with_context(|| format!("Failed to get database pool for '{}'", db.name))?;
        let lists = TodoList::get_all(&db_pool)
            .await
            .with_context(|| format!("Failed to get lists from database '{}'", db.name))?;

        // For each list, count items and display info
        for list in lists {
            let num = list
                .get_all_items(&db_pool)
                .await
                .with_context(|| format!("Failed to get items for list '{}'", list.name))?
                .len();
            writeln!(tw, "{}\t{}\t{}\t{}", list.name, list.id, db.name, num)
                .with_context(|| format!("Failed to write list entry for '{}'", list.name))?;
        }
    }

    tw.flush().with_context(|| "Failed to flush table writer")?;
    let output = String::from_utf8(
        tw.into_inner()
            .with_context(|| "Failed to get table writer buffer")?,
    )
    .with_context(|| "Failed to convert table output to string")?;
    print!("{output}");
    Ok(())
}

/// Creates a new todo list in the specified database
pub async fn add_list(app: &App, name: String, db_name: &Option<String>) -> Result<()> {
    let pool = get_db_pool_from_option(app, db_name).await?;
    let list = NewTodoList { name: name.clone() };
    TodoList::create(&pool, list)
        .await
        .with_context(|| format!("Failed to create list '{}'", name))?;
    Ok(())
}

/// Deletes a todo list by name or ID from the specified database
pub async fn delete_list(
    app: &App,
    name: Option<String>,
    id: Option<i64>,
    db_name: &Option<String>,
) -> Result<()> {
    let pool = get_db_pool_from_option(app, db_name).await?;

    let target_list = get_list_by_name_or_id(app, name, id, db_name).await?;
    target_list
        .delete(&pool)
        .await
        .with_context(|| format!("Failed to delete list"))?;
    Ok(())
}

/// Item operations

/// Lists all todo items across all databases and lists
///
/// Displays items in a formatted table showing:
/// - Item name, ID, and completion status
/// - Parent list name and ID
/// - Database name
pub async fn list_items(app: &App) -> Result<()> {
    let dbs = &app.config.dbs;
    let mut tw = TabWriter::new(vec![]);
    writeln!(tw, "Name\tID\tList name\tList ID\tDB\tDone?")
        .with_context(|| "Failed to write table header")?;
    writeln!(tw, "----\t--\t---------\t-------\t--\t-----")
        .with_context(|| "Failed to write table separator")?;

    // Iterate through all databases and their lists
    for db in dbs {
        let pool = get_db_pool(db.connection_str.as_str())
            .await
            .with_context(|| format!("Failed to get database pool for '{}'", db.name))?;
        let lists = TodoList::get_all(&pool)
            .await
            .with_context(|| format!("Failed to get lists from database '{}'", db.name))?;
        for list in lists {
            let items = TodoItem::get_by_list_id(&pool, list.id)
                .await
                .with_context(|| format!("Failed to get items for list '{}'", list.name))?;
            // Display each item with its context information
            for item in items {
                writeln!(
                    tw,
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    item.name, item.id, list.name, list.id, db.name, item.is_done
                )
                .with_context(|| format!("Failed to write item entry for '{}'", item.name))?
            }
        }
    }
    tw.flush().with_context(|| "Failed to flush table writer")?;
    let output = String::from_utf8(
        tw.into_inner()
            .with_context(|| "Failed to get table writer buffer")?,
    )
    .with_context(|| "Failed to convert table output to string")?;
    print!("{output}");
    Ok(())
}

/// Creates a new todo item in the specified list and database
pub async fn add_item(
    app: &App,
    name: String,
    db_name: &Option<String>,
    list_id: Option<i64>,
    list_name: Option<String>,
) -> Result<()> {
    let pool = get_db_pool_from_option(app, db_name).await?;
    let target_list = get_list_by_name_or_id(app, list_name, list_id, db_name).await?;

    let new_item = NewTodoItem {
        name: name.clone(),
        list_id: target_list.id,
        priority: None,
        due_date: None,
    };
    TodoItem::create(&pool, new_item)
        .await
        .with_context(|| format!("Failed to create item '{}'", name))?;
    Ok(())
}

/// Deletes a todo item by ID from the specified database
pub async fn delete_item(app: &App, id: i64, db_name: &Option<String>) -> Result<()> {
    let db = get_db_from_option(app, db_name)?;
    let pool = get_db_pool_from_option(app, db_name).await?;
    let item = match TodoItem::get_by_id(&pool, id)
        .await
        .with_context(|| format!("Failed to query item with ID '{}'", id))?
    {
        Some(this) => this,
        None => {
            eprintln!(
                "Error: Item with ID '{}' not found in database '{}'",
                id, db.name
            );
            std::process::exit(exitcode::DATAERR)
        }
    };

    item.delete(&pool)
        .await
        .with_context(|| format!("Failed to delete item with ID '{}'", id))
}

/// Toggles the completion status of a todo item
pub async fn toggle_done_item(app: &App, id: i64, db_name: &Option<String>) -> Result<()> {
    let db = get_db_from_option(app, db_name)?;
    let pool = get_db_pool_from_option(app, db_name).await?;
    let item = TodoItem::get_by_id(&pool, id)
        .await
        .with_context(|| format!("Failed to query item with ID '{}'", id))?;
    match item {
        Some(mut this) => this
            .toggle_done(&pool)
            .await
            .with_context(|| format!("Failed to toggle done status for item with ID '{}'", id)),
        None => {
            eprintln!(
                "Error: Item with ID '{}' not found in database '{}'",
                id, db.name
            );
            std::process::exit(exitcode::DATAERR);
        }
    }
}

/// General utility functions

/// Returns the specified database configuration or the default if omitted
fn get_db_from_option(app: &App, db: &Option<String>) -> Result<DBConfig> {
    return match db {
        Some(name) => app
            .config
            .clone()
            .get_db_by_name((name).to_string())
            .with_context(|| format!("Failed to get database configuration for '{}'", name)),
        None => app
            .config
            .get_default()
            .with_context(|| "Failed to get default database configuration"),
    };
}

/// Retrieves a todo list by either name or ID from the specified database
///
/// Exactly one of `name` or `id` must be provided. Exits with error if:
/// - Both name and ID are provided
/// - Neither name nor ID are provided  
/// - The specified list is not found
async fn get_list_by_name_or_id(
    app: &App,
    name: Option<String>,
    id: Option<i64>,
    db_name: &Option<String>,
) -> Result<TodoList> {
    let db = get_db_from_option(app, db_name)?;
    let pool = get_db_pool_from_option(app, db_name).await?;
    match (id, name) {
        // Search by ID
        (Some(list_id), None) => {
            return match TodoList::get_by_id(&pool, list_id)
                .await
                .with_context(|| format!("Failed to query list with ID '{}'", list_id))?
            {
                Some(list) => Ok(list),
                None => {
                    eprintln!(
                        "Error: List with ID '{}' not found in database '{}'",
                        list_id, db.name
                    );
                    std::process::exit(exitcode::DATAERR)
                }
            };
        }
        // Search by name
        (None, Some(list_name)) => {
            let lists = TodoList::get_all(&pool)
                .await
                .with_context(|| format!("Failed to get all lists from database '{}'", db.name))?;
            for list in lists {
                if list.name == list_name {
                    return Ok(list);
                }
            }
            eprintln!(
                "Error: List with name '{}' not found in database '{}'",
                list_name, db.name
            );
            std::process::exit(exitcode::DATAERR)
        }
        // Error cases
        (Some(_), Some(_)) => {
            eprintln!("Please provide either the name or the ID of the list, not both");
            std::process::exit(exitcode::DATAERR);
        }
        (None, None) => {
            eprintln!("Please provide either the name or the ID of the list");
            std::process::exit(exitcode::DATAERR);
        }
    }
}

/// Gets a database connection pool for the specified database
async fn get_db_pool_from_option(app: &App, db_option: &Option<String>) -> Result<Pool<Sqlite>> {
    let target_db = get_db_from_option(app, db_option)?;
    return get_db_pool(target_db.connection_str.as_str())
        .await
        .with_context(|| format!("Failed to create database pool for '{}'", target_db.name));
}
