# Judo

[![Crates.io](https://img.shields.io/crates/v/judo.svg)](https://crates.io/crates/judo)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

```
     ██╗██╗   ██╗██████╗  ██████╗ 
     ██║██║   ██║██╔══██╗██╔═══██╗
     ██║██║   ██║██║  ██║██║   ██║
██   ██║██║   ██║██║  ██║██║   ██║
╚█████╔╝╚██████╔╝██████╔╝╚██████╔╝
 ╚════╝  ╚═════╝ ╚═════╝  ╚═════╝ 
```

A terminal-based todo list application.

## Table of Contents

- [What Judo Looks Like](#what-judo-looks-like)
- [What It Does](#what-it-does)
- [Why Another Todo App](#why-another-todo-app)
- [Installation](#installation)
- [Usage](#usage)
- [Key Bindings](#key-bindings)
- [Configuration](#configuration)
- [Data Storage](#data-storage)

## What Judo Looks Like
![](https://github.com/giacomopiccinini/judo/blob/main/assets/demo.gif)

## What It Does

Judo (*Just Do It*) is a simple TUI for managing todo lists. You can create multiple lists, add items to them, mark items as complete, and delete items or entire lists when you're done.

The interface shows your lists on the left side and the items from the selected list on the right side. You can manage multiple databases, switch between them, and create new ones on the fly. All your data is saved locally on your computer, so your todos persist between sessions.

## Why Another Todo App

**Q: Who needs yet another todo app?**  
A: No one, really.

**Q: Then why did you create Judo in the first place?**  
A: I am often having conversations in Slack, taking notes on todo's and sending them to my private channel. Which looks embarassing, actually. So, there you go. Plus, I wanted to understand how to work with TUIs.

**Q: Why Rust?**  
A: No particular reason other than I wanted to familiarise more with it. No one cares about "blazing fast" performance for such a simple app. 


## Installation

Install Judo using Cargo:

```bash
cargo install judo
```

Then run it with:

```bash
judo
```

## Usage

When you start Judo, you'll see the main interface with two panels:

- **Left panel**: Your todo lists
- **Right panel**: Items from the selected list

Navigate between lists and items using the keyboard. You can switch between different databases, create new ones, and manage your todos across multiple databases. All changes are automatically saved to your local database.

### Command line usage
Judo also provides subcommands for usage on the command line or inside of shell scripts. Add `help`, `-h`, or `--help` to any subcommand to show its usage:

```
$ judo help
Judo - TUI for ToDo lists

Usage: judo [COMMAND]

Commands:
  dbs    Manage databases
  lists  Manage todo lists
  items  Manage todo items
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
#### Example:
```
$ judo lists show
Name          ID  DB          No of items
----          --  --          -----------
home          1   dojo        2
work          2   dojo        3
another list  1   another db  1
```

## Key Bindings

### Main Screen

#### List Navigation
| Key | Action |
|-----|--------|
| `w` | Move up in lists |
| `s` | Move down in lists |
| `↑` | Move up in items |
| `↓` | Move down in items |
| `←` | Deselect current item |
| `→` | Select first item in list |

#### Actions
| Key | Action |
|-----|--------|
| `A` | Add new list |
| `a` | Add new item to selected list |
| `M` | Modify selected list |
| `m` | Modify selected item |
| `D` | Delete selected list |
| `d` | Delete selected item |
| `C` | Change/switch database |
| `Enter` | Toggle item completion status |
| `Ctrl + w` | Move list up in ordering |
| `Ctrl + s` | Move list down in ordering |
| `Ctrl + ↑` | Move item up in ordering |
| `Ctrl + ↓` | Move item down in ordering |
| `q` | Quit application |

### Database Management Screen
| Key | Action |
|-----|--------|
| `↑` | Move up in database list |
| `↓` | Move down in database list |
| `Enter` | Switch to selected database |
| `A` | Add new database |
| `S` | Set selected database as default |
| `Esc` | Return to main screen |

### Add List/Item/Database Screens
| Key | Action |
|-----|--------|
| `Enter` | Save and return to previous screen |
| `Esc` | Cancel and return to previous screen |
| `Backspace` | Delete last character |
| `Delete` | Delete character after cursor |
| `←` | Move cursor left |
| `→` | Move cursor right |

## Configuration

Judo uses a configuration file to manage multiple databases and colour themes. The configuration is stored in:

- **Ubuntu**: `~/.config/judo/judo.toml`
- **macOS**: `~/Library/Application Support/judo/`
- **Windows**: `%APPDATA%\judo\judo.toml`

The configuration file is automatically created on first run with a default database. You can add new databases through the UI or manually edit the configuration file. The default database is called "dojo". Similarly, Judo comes equipped with a default colour theme (background: *dark green*, foreground: *white-ish*, highlight: *reddish*) which you can alter in the config file. 

### Example Configuration

```toml
default = "dojo"

[[dbs]]
name = "dojo"
connection_str = "sqlite:/path/to/data/judo.db"

[[dbs]]
name = "work"
connection_str = "sqlite:/path/to/data/work.db"

[[dbs]]
name = "personal"
connection_str = "sqlite:/path/to/data/personal.db"
```

## Data Storage

Your todo lists and items are stored in local SQLite databases on your computer. This means:

- Your data persists between application sessions
- No internet connection required
- Your todos remain private on your machine
- You can have multiple databases for different contexts (work, personal, projects, etc.)
- You can backup individual database files if needed
- Configuration and databases are stored in standard system directories

The databases and configuration are created automatically when you first run the application. Database files are stored in:

- **Ubuntu**: `~/.local/share/judo/`
- **macOS**: `~/Library/Application Support/judo/`
- **Windows**: `%APPDATA%\judo\`
