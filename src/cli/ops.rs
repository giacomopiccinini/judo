use std::io::Write;

use crate::db::config::DBConfig;
use crate::db::connections::{self, get_db_pool};
use crate::app::App;
use crate::db::models::{NewTodoItem, NewTodoList, TodoItem, TodoList};
use sqlx::{Pool, Sqlite};
use tabwriter::TabWriter;


//Database operations

pub fn list_dbs(app: &App) {
    let db_list = &app.config.dbs;

    let mut tw = TabWriter::new(vec![]);
    writeln!(tw, "Name\tConnection string").unwrap();
    writeln!(tw, "----\t-----------------").unwrap();
    for db in db_list {
        writeln!(tw, "{}\t{}", db.name, db.connection_str).unwrap();
    }
    tw.flush().unwrap();
    let output = String::from_utf8(tw.into_inner().unwrap()).unwrap();
    print!("{output}")
}

pub async fn add_db(mut app: App, name: String) {
    app.create_new_database(name, false).await.unwrap()
}


//List operations

pub async fn list_lists(app: &App) {
    let dbs = &app.config.dbs;
    let mut tw = TabWriter::new(vec![]);
    writeln!(tw, "Name\tID\tDB\tNo of items").unwrap();
    writeln!(tw, "----\t--\t--\t-----------").unwrap();
    
    for db in dbs {
        let db_pool = connections::get_db_pool(
            db.connection_str.as_str()
        ).await.unwrap();
        let lists = TodoList::get_all(&db_pool).await.unwrap();
        
        for list in lists {
            let num = list.get_all_items(&db_pool).await.unwrap().len();
            writeln!(tw, "{}\t{}\t{}\t{}", list.name, list.id, db.name, num).unwrap();
        }
    }
    tw.flush().unwrap();
    let output = String::from_utf8(tw.into_inner().unwrap()).unwrap();
    print!("{output}")
}

pub async fn add_list(app: &App, name: String, db_name: &Option<String>) {
    let pool = get_db_pool_from_option(app, db_name).await;
    
    let list = NewTodoList { name: name };
    TodoList::create(&pool, list).await.unwrap();
}

pub async fn delete_list(app: &App, name: Option<String>, id: Option<i64>, db_name: &Option<String>) {
    let pool = get_db_pool_from_option(app, db_name).await;

    let target_list = get_list_by_name_or_id(app, name, id, db_name).await;
    target_list.delete(&pool).await.unwrap();
}


//Item operations

pub async fn list_items(app: &App) {
    let dbs = &app.config.dbs;
    let mut tw = TabWriter::new(vec![]);
    writeln!(tw, "Name\tID\tList name\tList ID\tDB\tDone?").unwrap();
    writeln!(tw, "----\t--\t---------\t-------\t--\t-----").unwrap();

    for db in dbs {
        let pool = get_db_pool(db.connection_str.as_str()).await.unwrap();
        let lists = TodoList::get_all(&pool).await.unwrap();
        for list in lists {
            let items = TodoItem::get_by_list_id(&pool, list.id).await.unwrap();
            for item in items {
                writeln!(tw, "{}\t{}\t{}\t{}\t{}\t{}", item.name, item.id, list.name, list.id, db.name, item.is_done).unwrap()
            }
        }
    }
    tw.flush().unwrap();
    let output = String::from_utf8(tw.into_inner().unwrap()).unwrap();
    print!("{output}")
}

pub async fn add_item(app: &App, name: String, db_name: &Option<String>, list_id: Option<i64>, list_name: Option<String>) {
    let pool = get_db_pool_from_option(app, db_name).await;
    let target_list = get_list_by_name_or_id(app, list_name, list_id, db_name).await;

    let new_item = NewTodoItem{name: name, list_id: target_list.id, priority: None, due_date: None};
    TodoItem::create(&pool, new_item).await.unwrap();
}

pub async fn delete_item(app: &App, id: i64, db_name: &Option<String>) {
    let db = get_db_from_option(app, db_name);
    let pool = get_db_pool_from_option(app, db_name).await;
    let item = match TodoItem::get_by_id(&pool, id).await.unwrap() {
        Some(this) => this,
        None => {
            eprintln!("Error: Item with ID '{}' not found in database '{}'", id, db.name);
            std::process::exit(exitcode::DATAERR)
        }
    };

    item.delete(&pool).await.unwrap();
}

pub async fn toggle_done_item(app: &App, id: i64, db_name: &Option<String>) {
    let db = get_db_from_option(app, db_name);
    let pool = get_db_pool_from_option(app, db_name).await;
    let item = TodoItem::get_by_id(&pool, id).await.unwrap();
    match item {
        Some(mut this) => this.toggle_done(&pool).await.unwrap(),
        None => {
            eprintln!("Error: Item with ID '{}' not found in database '{}'", id, db.name);
            std::process::exit(exitcode::DATAERR);
        }
    }
}


//General

async fn get_list_by_name_or_id(app: &App, name: Option<String>, id: Option<i64>, db_name: &Option<String>) -> TodoList {
    let db = get_db_from_option(app, db_name);
    let pool = get_db_pool_from_option(app, db_name).await;
    match (id, name) {
        (Some(list_id), None) => {
            return match TodoList::get_by_id(&pool, list_id).await.unwrap() {
                Some(list) => list,
                None => {
                    eprintln!("Error: List with ID '{}' not found in database '{}'", list_id, db.name);
                    std::process::exit(exitcode::DATAERR)
                }
            }
        }
        (None, Some(list_name)) => {
            let lists = TodoList::get_all(&pool).await.unwrap();
            for list in lists {
                if list.name == list_name {
                    return list;
                }
            }
            eprintln!("Error: List with name '{}' not found in database '{}'", list_name, db.name);
            std::process::exit(exitcode::DATAERR)
        }
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

async fn get_db_pool_from_option(app: &App, db_option: &Option<String>) -> Pool<Sqlite> {
    let target_db = get_db_from_option(app, db_option);
    return get_db_pool(target_db.connection_str.as_str()).await.unwrap();
}

// Returns the specified DB or the default if omitted
fn get_db_from_option(app: &App, db: &Option<String>) -> DBConfig {
    return match db {
        Some(name) => app.config.clone().get_db_by_name((name).to_string()).unwrap(),
        None => app.config.get_default().unwrap()
    };
}