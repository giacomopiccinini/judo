use clap::Parser;
use judo::cli::args::{Cli, Commands, DbCommands, ItemCommands, ListCommands};

#[test]
fn test_cli_no_command() {
    // Test parsing CLI with no command
    let args = Cli::try_parse_from(["judo"]);
    assert!(args.is_ok());
    let cli = args.unwrap();
    assert!(cli.command.is_none());
}

#[test]
fn test_dbs_show_command() {
    // Test parsing "dbs show" command
    let args = Cli::try_parse_from(["judo", "dbs", "show"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Dbs { command }) => match command {
            Some(DbCommands::Show) => {
                // Success
            }
            _ => panic!("Expected DbCommands::Show"),
        },
        _ => panic!("Expected Commands::Dbs"),
    }
}

#[test]
fn test_dbs_add_command_with_name() {
    // Test parsing "dbs add" command with name
    let args = Cli::try_parse_from(["judo", "dbs", "add", "--name", "test_db"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Dbs { command }) => match command {
            Some(DbCommands::Add { name }) => {
                assert_eq!(name, "test_db");
            }
            _ => panic!("Expected DbCommands::Add"),
        },
        _ => panic!("Expected Commands::Dbs"),
    }
}

#[test]
fn test_dbs_add_command_with_short_flag() {
    // Test parsing "dbs add" command with short flag
    let args = Cli::try_parse_from(["judo", "dbs", "add", "-n", "my_database"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Dbs { command }) => match command {
            Some(DbCommands::Add { name }) => {
                assert_eq!(name, "my_database");
            }
            _ => panic!("Expected DbCommands::Add"),
        },
        _ => panic!("Expected Commands::Dbs"),
    }
}

#[test]
fn test_dbs_add_command_missing_name() {
    // Test parsing "dbs add" without name (should fail)
    let args = Cli::try_parse_from(["judo", "dbs", "add"]);
    assert!(args.is_err());
}

#[test]
fn test_lists_show_command_no_filter() {
    // Test parsing "lists show" without filters
    let args = Cli::try_parse_from(["judo", "lists", "show"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Show { name }) => {
                assert!(name.is_none());
            }
            _ => panic!("Expected ListCommands::Show"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_lists_show_command_with_name_filter() {
    // Test parsing "lists show" with name filter
    let args = Cli::try_parse_from(["judo", "lists", "show", "--name", "my_list"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Show { name }) => {
                assert_eq!(name, Some("my_list".to_string()));
            }
            _ => panic!("Expected ListCommands::Show"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_lists_add_command_minimal() {
    // Test parsing "lists add" with only name
    let args = Cli::try_parse_from(["judo", "lists", "add", "--name", "shopping"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Add { name, db }) => {
                assert_eq!(name, "shopping");
                assert!(db.is_none());
            }
            _ => panic!("Expected ListCommands::Add"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_lists_add_command_with_db() {
    // Test parsing "lists add" with name and db
    let args = Cli::try_parse_from(["judo", "lists", "add", "-n", "work", "-d", "office_db"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Add { name, db }) => {
                assert_eq!(name, "work");
                assert_eq!(db, Some("office_db".to_string()));
            }
            _ => panic!("Expected ListCommands::Add"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_lists_delete_by_name() {
    // Test parsing "lists delete" with name
    let args = Cli::try_parse_from(["judo", "lists", "delete", "--name", "old_list"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Delete { name, id, db }) => {
                assert_eq!(name, Some("old_list".to_string()));
                assert!(id.is_none());
                assert!(db.is_none());
            }
            _ => panic!("Expected ListCommands::Delete"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_lists_delete_by_id() {
    // Test parsing "lists delete" with ID
    let args = Cli::try_parse_from(["judo", "lists", "delete", "--id", "42"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Delete { name, id, db }) => {
                assert!(name.is_none());
                assert_eq!(id, Some(42));
                assert!(db.is_none());
            }
            _ => panic!("Expected ListCommands::Delete"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_lists_delete_with_db() {
    // Test parsing "lists delete" with name and db
    let args = Cli::try_parse_from(["judo", "lists", "delete", "-n", "temp", "-d", "test_db"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Delete { name, id, db }) => {
                assert_eq!(name, Some("temp".to_string()));
                assert!(id.is_none());
                assert_eq!(db, Some("test_db".to_string()));
            }
            _ => panic!("Expected ListCommands::Delete"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_items_show_command() {
    // Test parsing "items show"
    let args = Cli::try_parse_from(["judo", "items", "show"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Show) => {
                // Success
            }
            _ => panic!("Expected ItemCommands::Show"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_add_minimal() {
    // Test parsing "items add" with minimal arguments
    let args = Cli::try_parse_from(["judo", "items", "add", "--name", "Buy milk"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Add {
                name,
                db,
                list_name,
                list_id,
            }) => {
                assert_eq!(name, "Buy milk");
                assert!(db.is_none());
                assert!(list_name.is_none());
                assert!(list_id.is_none());
            }
            _ => panic!("Expected ItemCommands::Add"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_add_with_list_name() {
    // Test parsing "items add" with list name
    let args = Cli::try_parse_from([
        "judo",
        "items",
        "add",
        "--name",
        "Task 1",
        "--list-name",
        "shopping",
    ]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Add {
                name,
                db,
                list_name,
                list_id,
            }) => {
                assert_eq!(name, "Task 1");
                assert!(db.is_none());
                assert_eq!(list_name, Some("shopping".to_string()));
                assert!(list_id.is_none());
            }
            _ => panic!("Expected ItemCommands::Add"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_add_with_list_id() {
    // Test parsing "items add" with list ID
    let args = Cli::try_parse_from(["judo", "items", "add", "-n", "Task 2", "-i", "5"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Add {
                name,
                db,
                list_name,
                list_id,
            }) => {
                assert_eq!(name, "Task 2");
                assert!(db.is_none());
                assert!(list_name.is_none());
                assert_eq!(list_id, Some(5));
            }
            _ => panic!("Expected ItemCommands::Add"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_add_with_db_and_list() {
    // Test parsing "items add" with full arguments
    let args = Cli::try_parse_from([
        "judo",
        "items",
        "add",
        "--name",
        "Important task",
        "--db",
        "work_db",
        "--list-name",
        "projects",
    ]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Add {
                name,
                db,
                list_name,
                list_id,
            }) => {
                assert_eq!(name, "Important task");
                assert_eq!(db, Some("work_db".to_string()));
                assert_eq!(list_name, Some("projects".to_string()));
                assert!(list_id.is_none());
            }
            _ => panic!("Expected ItemCommands::Add"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_delete_command() {
    // Test parsing "items delete"
    let args = Cli::try_parse_from(["judo", "items", "delete", "--id", "10"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Delete { id, db }) => {
                assert_eq!(id, 10);
                assert!(db.is_none());
            }
            _ => panic!("Expected ItemCommands::Delete"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_delete_with_db() {
    // Test parsing "items delete" with db
    let args = Cli::try_parse_from(["judo", "items", "delete", "-i", "15", "-d", "archive_db"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Delete { id, db }) => {
                assert_eq!(id, 15);
                assert_eq!(db, Some("archive_db".to_string()));
            }
            _ => panic!("Expected ItemCommands::Delete"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_delete_missing_id() {
    // Test parsing "items delete" without required ID (should fail)
    let args = Cli::try_parse_from(["judo", "items", "delete"]);
    assert!(args.is_err());
}

#[test]
fn test_items_toggle_done_command() {
    // Test parsing "items toggle-done"
    let args = Cli::try_parse_from(["judo", "items", "toggle-done", "--id", "7"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::ToggleDone { id, db }) => {
                assert_eq!(id, 7);
                assert!(db.is_none());
            }
            _ => panic!("Expected ItemCommands::ToggleDone"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_items_toggle_done_with_db() {
    // Test parsing "items toggle-done" with db
    let args = Cli::try_parse_from([
        "judo",
        "items",
        "toggle-done",
        "-i",
        "3",
        "-d",
        "personal_db",
    ]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::ToggleDone { id, db }) => {
                assert_eq!(id, 3);
                assert_eq!(db, Some("personal_db".to_string()));
            }
            _ => panic!("Expected ItemCommands::ToggleDone"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_cli_with_special_characters() {
    // Test parsing with special characters in names
    let args = Cli::try_parse_from([
        "judo",
        "lists",
        "add",
        "--name",
        "My List! @#$%",
        "--db",
        "test_db-123",
    ]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Add { name, db }) => {
                assert_eq!(name, "My List! @#$%");
                assert_eq!(db, Some("test_db-123".to_string()));
            }
            _ => panic!("Expected ListCommands::Add"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_cli_with_empty_string_name() {
    // Test parsing with empty string name
    let args = Cli::try_parse_from(["judo", "items", "add", "--name", ""]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Add { name, .. }) => {
                assert_eq!(name, "");
            }
            _ => panic!("Expected ItemCommands::Add"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_cli_with_unicode_characters() {
    // Test parsing with unicode characters
    let args = Cli::try_parse_from(["judo", "items", "add", "--name", "🚀 Rocket task 测试"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Items { command }) => match command {
            Some(ItemCommands::Add { name, .. }) => {
                assert_eq!(name, "🚀 Rocket task 测试");
            }
            _ => panic!("Expected ItemCommands::Add"),
        },
        _ => panic!("Expected Commands::Items"),
    }
}

#[test]
fn test_cli_with_very_long_name() {
    // Test parsing with very long name
    let long_name = "A".repeat(1000);
    let args = Cli::try_parse_from(["judo", "lists", "add", "--name", &long_name]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Add { name, .. }) => {
                assert_eq!(name, long_name);
            }
            _ => panic!("Expected ListCommands::Add"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}

#[test]
fn test_cli_with_negative_id() {
    // Test parsing with negative ID (should fail because IDs are positive integers)
    let args = Cli::try_parse_from(["judo", "items", "delete", "--id", "-5"]);
    assert!(args.is_err()); // Negative IDs should be rejected by clap
}

#[test]
fn test_cli_with_zero_id() {
    // Test parsing with zero ID
    let args = Cli::try_parse_from(["judo", "lists", "delete", "--id", "0"]);
    assert!(args.is_ok());
    let cli = args.unwrap();

    match cli.command {
        Some(Commands::Lists { command }) => match command {
            Some(ListCommands::Delete { id, .. }) => {
                assert_eq!(id, Some(0));
            }
            _ => panic!("Expected ListCommands::Delete"),
        },
        _ => panic!("Expected Commands::Lists"),
    }
}
