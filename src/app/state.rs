use crate::app::events::EventHandler;
use crate::db::config::{Config, DBConfig};
use crate::db::connections::init_db;
use crate::db::models::{TodoList, UIList};
use crate::ui::components::{
    AddDBPopUp, AddItemPopUp, AddListPopUp, ChangeDBPopUp, DBSelector, InputState, ItemsComponent,
    ListsComponent, Logo, ModifyItemPopUp, ModifyListPopUp,
};
use crate::ui::cursor::CursorState;
use crate::ui::layout::AppLayout;
use anyhow::{Context, Result};
use crossterm::event::{self, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use sqlx::SqlitePool;

/// Enum representing the different screens in the application
#[derive(Debug, Clone, PartialEq)]
pub enum CurrentScreen {
    /// Main screen showing lists and items
    Main,
    /// Pop-up screen for adding a new list
    AddList,
    /// Pop-up screen for modifying an existing list
    ModifyList,
    /// Pop-up screen for adding a new item
    AddItem,
    /// Pop-up screen for adding a new item
    ModifyItem,
    /// Pop-up for changing database
    ChangeDB,
    /// Pop-up for adding a new database
    AddDB,
}

/// Main application state
pub struct App {
    /// Configuration of available databases
    pub config: Config,
    /// Config of currently selected database
    pub current_db_config: DBConfig,
    /// Current active screen (Main, AddList, ModifyList, or AddItem)
    pub current_screen: CurrentScreen,
    /// Database connection pool
    pub pool: SqlitePool,
    /// Lists component for managing todo lists
    pub lists_component: ListsComponent,
    /// State of user-provided input
    pub input_state: InputState,
    /// Selected database index for DB selector
    pub selected_db_index: usize,
    /// Flag to indicate if the application should exit
    pub exit: bool,
}

impl App {
    /// Create new app instance
    ///
    /// Initializes the database connection, loads existing lists from the database,
    /// and sets up the initial UI state.
    pub async fn new() -> Self {
        // Read the config (creates default if missing)
        let config = Config::read().expect("Failed to read config file");

        // Extract the default db and its connection string
        let default_db_config = config
            .get_default()
            .expect("Couldn't fetch default database");
        let pool = init_db(&default_db_config.connection_str)
            .await
            .expect("Failed to connect to database");

        // Start from main screen
        let current_screen = CurrentScreen::Main;

        // Create lists component and load data
        let mut lists_component = ListsComponent::new();
        lists_component
            .load_lists(&pool)
            .await
            .expect("Failed to read lists");

        Self {
            config,
            current_db_config: default_db_config,
            current_screen,
            pool,
            lists_component,
            input_state: InputState::new(),
            selected_db_index: 0,
            exit: false,
        }
    }

    /// Run the application
    ///
    /// Main event loop that handles terminal drawing and user input.
    /// Continues until the user exits the application.
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            // Draw the current state of the application
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            // Handle keyboard input based on current screen
            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key_event(key).await;
            }
        }
        Ok(())
    }

    /// Create a new database with the given name
    pub async fn create_new_database(
        &mut self,
        db_name: String,
        set_as_default: bool,
    ) -> Result<()> {
        // Use data directory to standardize storage
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
            .join("judo");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir).with_context(|| "Failed to create data directory")?;

        // Create path to new db file
        let db_file = format!("{}.db", db_name);
        let path = data_dir.join(db_file);

        // Create connection string (only SQLite is admissible)
        let connection_str = format!("sqlite:{}", path.display());

        // Create new database config
        let new_db_config = DBConfig {
            name: db_name.clone(),
            connection_str: connection_str.clone(),
        };

        // Initialize the new database (this creates the file and runs migrations)
        init_db(&connection_str)
            .await
            .with_context(|| "Failed to initialize new database")?;

        // Add to config
        self.config.dbs.push(new_db_config);

        // Set as default if requested
        if set_as_default {
            self.config.default = db_name.clone();
        }

        // Write updated config to file
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("judo");
        let config_path = config_dir.join("judo.toml");

        self.config
            .write(&config_path)
            .with_context(|| "Failed to save config")?;

        // Update selected index to point to the new database
        self.selected_db_index = self.config.dbs.len() - 1;

        Ok(())
    }

    /// Handle key events and delegate to appropriate handler
    async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.current_screen {
            CurrentScreen::Main => EventHandler::handle_main_screen_key(self, key).await,
            CurrentScreen::AddList | CurrentScreen::ModifyList => {
                EventHandler::handle_add_or_modify_list_screen_key(self, key).await
            }
            CurrentScreen::AddItem => {
                EventHandler::handle_add_or_modify_item_screen_key(self, key).await
            }
            CurrentScreen::ModifyItem => {
                EventHandler::handle_add_or_modify_item_screen_key(self, key).await
            }
            CurrentScreen::ChangeDB => EventHandler::handle_change_db_screen_key(self, key).await,
            CurrentScreen::AddDB => EventHandler::handle_add_db_screen_key(self, key).await,
        }
    }

    /// Enter the "Add List" screen by opening the corresponding pop-up
    pub fn enter_add_list_screen(&mut self) {
        self.input_state = InputState::default();
        self.current_screen = CurrentScreen::AddList;
    }

    /// Enter the "Modify List" screen by opening the corresponding pop-up
    pub fn enter_modify_list_screen(&mut self, selected_list: &TodoList) {
        self.input_state = InputState {
            current_input: selected_list.name.clone(),
            cursor_pos: 0,
            is_modifying: true,
        };
        self.current_screen = CurrentScreen::ModifyList;
    }

    /// Enter the "Add Item" screen by opening the corresponding pop-up
    pub fn enter_add_item_screen(&mut self) {
        if self.lists_component.selected().is_some() {
            self.input_state = InputState::default();
            self.current_screen = CurrentScreen::AddItem;
        }
    }

    /// Enter the "Modify Item" screen by opening the corresponding pop-up
    pub fn enter_modify_item_screen(&mut self, ui_list: &UIList) {
        if self.lists_component.selected().is_some()
            && let Some(j) = ui_list.item_state.selected()
        {
            let selected_item = ui_list.items[j].item.clone();

            self.input_state = InputState {
                current_input: selected_item.name.clone(),
                cursor_pos: 0,
                is_modifying: true,
            };
            self.current_screen = CurrentScreen::ModifyItem;
        }
    }

    /// Exit the Add List screen without saving
    pub fn exit_add_or_modify_list_without_saving(&mut self) {
        self.current_screen = CurrentScreen::Main;
        self.input_state.clear();
    }

    /// Exit the Add Item screen without saving
    pub fn exit_add_item_without_saving(&mut self) {
        self.current_screen = CurrentScreen::Main;
        self.input_state.clear();
    }

    /// Enter the "Change DB" screen by opening the corresponding pop-up
    pub fn enter_change_db_screen(&mut self) {
        // Find the index of the current database in the config
        self.selected_db_index = self
            .config
            .dbs
            .iter()
            .position(|db| db.name == self.current_db_config.name)
            .unwrap_or(0);
        self.current_screen = CurrentScreen::ChangeDB;
    }

    /// Exit the Change DB screen without saving
    pub fn exit_change_db_without_saving(&mut self) {
        self.current_screen = CurrentScreen::Main;
    }

    /// Enter the "Add DB" screen by opening the corresponding pop-up
    pub fn enter_add_db_screen(&mut self) {
        self.current_screen = CurrentScreen::AddDB;
    }

    /// Exit the Add DB screen without saving
    pub fn exit_add_db_without_saving(&mut self) {
        self.current_screen = CurrentScreen::ChangeDB;
        self.input_state.clear();
    }

    /// Move selection up in DB list
    pub fn select_previous_db(&mut self) {
        if self.config.dbs.is_empty() {
            return;
        }
        self.selected_db_index = if self.selected_db_index == 0 {
            self.config.dbs.len() - 1
        } else {
            self.selected_db_index - 1
        };
    }

    /// Move selection down in DB list
    pub fn select_next_db(&mut self) {
        if self.config.dbs.is_empty() {
            return;
        }
        self.selected_db_index = (self.selected_db_index + 1) % self.config.dbs.len();
    }

    /// Switch to the selected database
    pub async fn switch_to_selected_db(&mut self) -> Result<()> {
        if let Some(selected_db) = self.config.dbs.get(self.selected_db_index) {
            // Initialize connection to the new database
            let new_pool = init_db(&selected_db.connection_str)
                .await
                .with_context(|| "Failed to connect to database")?;

            // Update app state
            self.current_db_config = selected_db.clone();
            self.pool = new_pool;

            // Reload all lists from the new database
            self.lists_component = ListsComponent::new();
            self.lists_component
                .load_lists(&self.pool)
                .await
                .with_context(|| "Failed to load lists")?;

            // Return to main screen
            self.current_screen = CurrentScreen::Main;
        }
        Ok(())
    }

    /// Set the selected database as default
    pub async fn set_selected_db_as_default(&mut self) -> Result<()> {
        if let Some(selected_db) = self.config.dbs.get(self.selected_db_index) {
            // Update the default in config
            self.config.default = selected_db.name.clone();

            // Write updated config to file
            let config_dir = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
                .join("judo");
            let config_path = config_dir.join("judo.toml");

            self.config
                .write(&config_path)
                .with_context(|| "Failed to save config")?;
        }
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render background
        AppLayout::render_background(self.config.clone(), area, buf);

        // Calculate layout areas
        let (lists_area, items_area, logo_area, db_selector_area, closed_selector_area) =
            AppLayout::calculate_main_layout(area);

        // Render logo
        Logo::render(logo_area, buf);

        // Render db selector only when not in database-related popups
        if !matches!(
            self.current_screen,
            CurrentScreen::ChangeDB | CurrentScreen::AddDB
        ) {
            DBSelector::render(
                closed_selector_area,
                buf,
                &self.current_db_config.name,
                self.config.clone(),
            );
        }

        // Render the main areas
        self.lists_component
            .render(lists_area, buf, self.config.clone());

        // Render items with the selected list
        let selected_list = self.lists_component.get_selected_list_mut();
        ItemsComponent::render(selected_list, items_area, buf, self.config.clone());

        // Render popup screens if active
        match self.current_screen {
            CurrentScreen::AddList => {
                AddListPopUp::render(self.config.clone(), &self.input_state, lists_area, buf)
            }
            CurrentScreen::ModifyList => {
                ModifyListPopUp::render(self.config.clone(), &self.input_state, lists_area, buf)
            }
            CurrentScreen::AddItem => {
                AddItemPopUp::render(self.config.clone(), &self.input_state, items_area, buf)
            }
            CurrentScreen::ModifyItem => {
                ModifyItemPopUp::render(self.config.clone(), &self.input_state, items_area, buf)
            }
            CurrentScreen::ChangeDB => {
                ChangeDBPopUp::render(&self.config, self.selected_db_index, db_selector_area, buf)
            }
            CurrentScreen::AddDB => AddDBPopUp::render(
                self.config.clone(),
                &self.input_state,
                db_selector_area,
                buf,
            ),
            _ => {}
        }
    }
}
