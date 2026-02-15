use crate::app::state::{App, CurrentScreen};
use crate::ui::components::{ItemsComponent, ListsComponent};
use crate::ui::cursor::CursorState;
use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    /// Handle key press from user in main screen
    pub async fn handle_main_screen_key(app: &mut App, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => app.exit = true, // Quit application
            (KeyCode::Char('s'), KeyModifiers::NONE) => app.lists_component.select_next(), // Navigate down in lists
            (KeyCode::Char('w'), KeyModifiers::NONE) => app.lists_component.select_previous(), // Navigate up in lists
            (KeyCode::Char('A'), KeyModifiers::SHIFT) => app.enter_add_list_screen(), // Add new list
            (KeyCode::Char('a'), KeyModifiers::NONE) => app.enter_add_item_screen(), // Add new item
            (KeyCode::Char('C'), KeyModifiers::SHIFT) => app.enter_change_db_screen(), // Change database
            (KeyCode::Char('M'), KeyModifiers::SHIFT) => {
                if let Some(selected_list) = app.lists_component.get_selected_list() {
                    app.enter_modify_list_screen(&selected_list.list.clone())
                }
            } // Modify existing list
            (KeyCode::Char('m'), KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list() {
                    app.enter_modify_item_screen(&selected_list.clone())
                }
            } // Modify existing item
            (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
                if let Err(e) =
                    ListsComponent::delete_selected_list_static(&mut app.lists_component, &app.pool)
                        .await
                {
                    // Log error but don't crash the application
                    eprintln!("Failed to delete list: {}", e);
                }
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut()
                    && let Err(e) =
                        ItemsComponent::delete_selected_item(selected_list, &app.pool).await
                {
                    eprintln!("Failed to delete item: {}", e);
                }
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut()
                    && let Err(e) = ItemsComponent::toggle_item_done(selected_list, &app.pool).await
                {
                    eprintln!("Failed to toggle item: {}", e);
                }
            }
            (KeyCode::Char('s'), KeyModifiers::ALT) => {
                // Ctrl+S: Move selected list down
                if let Err(e) =
                    ListsComponent::move_selected_list_down(&mut app.lists_component, &app.pool)
                        .await
                {
                    eprintln!("Failed to move list down: {}", e);
                }
            }
            (KeyCode::Char('w'), KeyModifiers::ALT) => {
                // Ctrl+W: Move selected list up
                if let Err(e) =
                    ListsComponent::move_selected_list_up(&mut app.lists_component, &app.pool).await
                {
                    eprintln!("Failed to move list up: {}", e);
                }
            }
            (KeyCode::Up, KeyModifiers::ALT) => {
                // Ctrl+Up: Move selected item up
                if let Some(selected_list) = app.lists_component.get_selected_list_mut()
                    && let Err(e) =
                        ItemsComponent::move_selected_item_up(selected_list, &app.pool).await
                {
                    eprintln!("Failed to move item up: {}", e);
                }
            }
            (KeyCode::Down, KeyModifiers::ALT) => {
                // Ctrl+Down: Move selected item down
                if let Some(selected_list) = app.lists_component.get_selected_list_mut()
                    && let Err(e) =
                        ItemsComponent::move_selected_item_down(selected_list, &app.pool).await
                {
                    eprintln!("Failed to move item down: {}", e);
                }
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    ItemsComponent::select_next_item(selected_list);
                }
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    ItemsComponent::select_previous_item(selected_list);
                }
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    ItemsComponent::remove_item_selection(selected_list);
                }
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    ItemsComponent::select_first_item(selected_list);
                }
            }
            // Copy all items
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    let content = ItemsComponent::format_all_items(selected_list);

                    // Spawn thread to keep clipboard alive until content is read
                    #[cfg(target_os = "linux")]
                    std::thread::spawn(move || {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            // Use Linux extension to wait until clipboard is read
                            let _ = clipboard.set().wait().text(content);
                        }
                    });
                    #[cfg(not(target_os = "linux"))]
                    std::thread::spawn(move || {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            let _ = clipboard.set_text(content);
                        }
                    });
                }
            }
            // Select first list
            (KeyCode::Char('T'), KeyModifiers::SHIFT) => {
                app.lists_component.select_first();
            }
            // Select last list
            (KeyCode::Char('B'), KeyModifiers::SHIFT) => {
                app.lists_component.select_last();
            }
            // Select the first item in the list
            (KeyCode::Char('t'), KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    ItemsComponent::select_first_item(selected_list);
                }
            }
            // Select the last item in the list
            (KeyCode::Char('b'), KeyModifiers::NONE) => {
                if let Some(selected_list) = app.lists_component.get_selected_list_mut() {
                    ItemsComponent::select_last_item(selected_list);
                }
            }
            _ => {}
        }
    }

    /// Handle key press from user in add list screen
    pub async fn handle_add_or_modify_list_screen_key(app: &mut App, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => app.exit_add_or_modify_list_without_saving(),
            (KeyCode::Backspace, KeyModifiers::NONE) => app.input_state.remove_char_before_cursor(),
            (KeyCode::Delete, KeyModifiers::NONE) => app.input_state.delete_char_after_cursor(),
            (KeyCode::Char(value), KeyModifiers::NONE) => app.input_state.add_char(value),
            (KeyCode::Left, KeyModifiers::NONE) => app.input_state.move_cursor_left(),
            (KeyCode::Right, KeyModifiers::NONE) => app.input_state.move_cursor_right(),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => app.input_state.move_cursor_to_start(),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => app.input_state.move_cursor_to_end(),
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let list_name = app.input_state.get_text().to_string();
                // Only do something if the list has a name
                if !list_name.trim().is_empty() {
                    if app.input_state.is_modifying {
                        if let Err(e) = ListsComponent::update_list(
                            &mut app.lists_component,
                            list_name,
                            &app.pool,
                        )
                        .await
                        {
                            eprintln!("Failed to update list: {}", e);
                        } else {
                            app.current_screen = CurrentScreen::Main;
                            app.input_state.clear();
                        }
                    } else if let Err(e) =
                        ListsComponent::create_list(&mut app.lists_component, list_name, &app.pool)
                            .await
                    {
                        eprintln!("Failed to create list: {}", e);
                    } else {
                        app.lists_component.select_last();
                        app.current_screen = CurrentScreen::Main;
                        app.input_state.clear();
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle key press from user in add item screen
    pub async fn handle_add_or_modify_item_screen_key(app: &mut App, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => app.exit_add_item_without_saving(),
            (KeyCode::Backspace, KeyModifiers::NONE) => app.input_state.remove_char_before_cursor(),
            (KeyCode::Delete, KeyModifiers::NONE) => app.input_state.delete_char_after_cursor(),
            (KeyCode::Left, KeyModifiers::NONE) => app.input_state.move_cursor_left(),
            (KeyCode::Right, KeyModifiers::NONE) => app.input_state.move_cursor_right(),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => app.input_state.move_cursor_to_start(),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => app.input_state.move_cursor_to_end(),
            (KeyCode::Char(value), KeyModifiers::NONE) => app.input_state.add_char(value),
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let item_name = app.input_state.get_text().to_string();
                if !item_name.trim().is_empty()
                    && let Some(selected_list) = app.lists_component.get_selected_list_mut()
                {
                    if app.input_state.is_modifying {
                        if let Err(e) =
                            ItemsComponent::update_item(selected_list, item_name, &app.pool).await
                        {
                            eprintln!("Failed to update item: {}", e);
                        } else {
                            app.current_screen = CurrentScreen::Main;
                            app.input_state.clear();
                        }
                    } else if let Err(e) =
                        ItemsComponent::create_item(selected_list, item_name, &app.pool).await
                    {
                        eprintln!("Failed to create item: {}", e);
                    } else {
                        ItemsComponent::select_last_item(selected_list);
                        app.current_screen = CurrentScreen::Main;
                        app.input_state.clear();
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle change of db
    pub async fn handle_change_db_screen_key(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => app.exit_change_db_without_saving(),
            KeyCode::Up => app.select_previous_db(),
            KeyCode::Down => app.select_next_db(),
            KeyCode::Enter => {
                if let Err(e) = app.switch_to_selected_db().await {
                    eprintln!("Failed to switch database: {}", e);
                }
            }
            KeyCode::Char('A') => app.enter_add_db_screen(),
            KeyCode::Char('S') => {
                // Set selected database as default
                if let Err(e) = app.set_selected_db_as_default().await {
                    eprintln!("Failed to set database as default: {}", e);
                }
            }
            _ => {}
        }
    }

    /// Handle key press from user in add database screen
    pub async fn handle_add_db_screen_key(app: &mut App, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => app.exit_add_db_without_saving(),
            (KeyCode::Backspace, KeyModifiers::NONE) => app.input_state.remove_char_before_cursor(),
            (KeyCode::Delete, KeyModifiers::NONE) => app.input_state.delete_char_after_cursor(),
            (KeyCode::Char(value), KeyModifiers::NONE) => app.input_state.add_char(value),
            (KeyCode::Left, KeyModifiers::NONE) => app.input_state.move_cursor_left(),
            (KeyCode::Right, KeyModifiers::NONE) => app.input_state.move_cursor_right(),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => app.input_state.move_cursor_to_start(),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => app.input_state.move_cursor_to_end(),
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let db_name = app.input_state.get_text().to_string();
                if !db_name.trim().is_empty() {
                    if let Err(e) = app.create_new_database(db_name, false).await {
                        eprintln!("Failed to create database: {}", e);
                    } else {
                        app.current_screen = CurrentScreen::ChangeDB;
                        app.input_state.clear();
                    }
                }
            }
            _ => {}
        }
    }
}
