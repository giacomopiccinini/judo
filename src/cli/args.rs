use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    ///Manage databases
    Dbs {
        #[command(subcommand)]
        command: Option<DbCommands>
    },

    ///Manage todo lists
    Lists {
        #[command(subcommand)]
        command: Option<ListCommands>
    },

    ///Manage todo items
    Items {#[command(subcommand)]
        command: Option<ItemCommands>}
}

#[derive(Subcommand, Debug)]
pub enum DbCommands {
    ///List all databases
    Show,

    ///Add a new database with the given name
    Add {
        ///Name of the new database
        #[arg(short, long)]
        name: String
    },
}

#[derive(Subcommand, Debug)]
pub enum ListCommands {
    ///List all todo lists in all databases
    Show,

    ///Add a new todo list with the given name to the specified database (default DB if omitted)
    Add {
        ///Name of the new todo list
        #[arg(short, long)]
        name: String,

        ///Name of the database to add the new list to (default DB if omitted)
        #[arg(short, long)]
        db: Option<String>
    },

    ///Delete an existing todo list with the given name or ID from the specified database (default DB if omitted)
    Delete {
        ///Name of the list to be deleted (do not use with -i|--id)
        #[arg(short, long)]
        name: Option<String>,

        ///ID of the list to be deleted (do not use with -n|--name)
        #[arg(short, long)]
        id: Option<i64>,

        ///Name of the database that contains the target list (default DB if omitted)
        #[arg(short, long)]
        db: Option<String>
    },
}

#[derive(Subcommand, Debug)]
pub enum ItemCommands {
    ///List all todo items in a table which shows what list and database each belongs to
    Show,

    ///Add a new todo item with the given name to the specified list (by ID or name) and database (default DB if omitted)
    Add {
        ///Name of the new todo item
        #[arg(short, long)]
        name: String,

        ///Name of the database that contains the list to hold the new todo item (default DB if omitted)
        #[arg(short, long)]
        db: Option<String>,

        ///Name of the list to hold the new todo item (do not use with -i|--id)
        #[arg(short, long)]
        list_name: Option<String>,

        ///ID of the list to hold the new todo item (do not use with -n|--name)
        #[arg(short = 'i', long)]
        list_id: Option<i64>
    },

    ///Delete an existing todo item with the given ID from the given database (default DB if omitted)
    Delete {
        ///ID of the target todo item
        #[arg(short, long)]
        id: i64,    // I opted not to allow deleting by name, as this does not seem practical to use 
                    // and would likely cause more issues than it is worth for users

        ///Name of the database that contains the todo item to be deleted (default DB if omitted)
        #[arg(short, long)]
        db: Option<String>,
    },

    ///Toggle whether a todo item is marked as done or not
    ToggleDone {
        ///ID of the target item
        #[arg(short, long)]
        id: i64,

        ///Name of the database containing the target item
        #[arg(short, long)]
        db: Option<String>
    }
}

