use crate::db::models::{NewTodoItem, TodoItem, UIItem, UIList};
use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, HighlightSpacing, List, ListItem, Padding, StatefulWidget, Widget,
};
use sqlx::SqlitePool;
use std::str::FromStr;
pub struct ItemsComponent;

impl ItemsComponent {
    /// Apply styling to a todo item based on its completion status
    fn style_item(ui_item: &UIItem) -> Line<'_> {
        let name = ui_item.item.name.clone();

        let bullet = Span::from("• ");
        let content = if ui_item.item.is_done {
            // Strike through completed items
            Span::styled(name, Style::default().add_modifier(Modifier::CROSSED_OUT))
        } else {
            Span::from(name)
        };

        Line::from(vec![bullet, content])
    }

    /// Select next element in the list of to-do items
    pub fn select_next_item(ui_list: &mut UIList) {
        ui_list.item_state.select_next();
    }

    /// Select previous element in the list of to-do items
    pub fn select_previous_item(ui_list: &mut UIList) {
        ui_list.item_state.select_previous();
    }

    /// Remove item selection (deselect current item)
    pub fn remove_item_selection(ui_list: &mut UIList) {
        ui_list.item_state.select(None);
    }

    /// Select the first item in the list
    pub fn select_first_item(ui_list: &mut UIList) {
        if ui_list.item_state.selected().is_none() {
            ui_list.item_state.select_first();
        }
    }

    /// Toggle the "is done" status of the currently selected item
    pub async fn toggle_item_done(ui_list: &mut UIList, pool: &SqlitePool) -> Result<()> {
        if let Some(j) = ui_list.item_state.selected() {
            ui_list.items[j].item.toggle_done(pool).await?;
        }
        Ok(())
    }

    /// Create a new item in the given list
    pub async fn create_item(ui_list: &mut UIList, name: String, pool: &SqlitePool) -> Result<()> {
        let new_item = NewTodoItem {
            name,
            list_id: ui_list.list.id,
            priority: None,
            due_date: None,
        };

        TodoItem::create(pool, new_item).await?;
        ui_list.update_items(pool).await?;
        Ok(())
    }

    /// Update an existing item
    pub async fn update_item(ui_list: &mut UIList, name: String, pool: &SqlitePool) -> Result<()> {
        if let Some(j) = ui_list.item_state.selected() {
            let mut item = ui_list.items[j].item.clone();
            item.update_name(pool, name).await?;

            // Update list elements
            ui_list.update_items(pool).await?;
        }
        Ok(())
    }

    /// Delete the currently selected item
    pub async fn delete_selected_item(ui_list: &mut UIList, pool: &SqlitePool) -> Result<()> {
        if let Some(j) = ui_list.item_state.selected() {
            let item = ui_list.items[j].item.clone();
            item.delete(pool).await?;

            // Update list elements
            ui_list.update_items(pool).await?;

            // Adjust selection after deletion - check bounds first
            if ui_list.items.is_empty() {
                ui_list.item_state.select(None);
            } else if j >= ui_list.items.len() {
                ui_list.item_state.select(Some(ui_list.items.len() - 1));
            }
        }
        Ok(())
    }

    /// Move the currently selected item up
    pub async fn move_selected_item_up(ui_list: &mut UIList, pool: &SqlitePool) -> Result<()> {
        if let Some(j) = ui_list.item_state.selected() {
            let mut item = ui_list.items[j].item.clone();
            item.move_up(pool).await?;

            // Update list elements to reflect the new order
            ui_list.update_items(pool).await?;

            // Adjust selection to follow the moved item
            if j > 0 {
                ui_list.item_state.select(Some(j - 1));
            }
        }
        Ok(())
    }

    /// Move the currently selected item down
    pub async fn move_selected_item_down(ui_list: &mut UIList, pool: &SqlitePool) -> Result<()> {
        if let Some(j) = ui_list.item_state.selected() {
            let mut item = ui_list.items[j].item.clone();
            item.move_down(pool).await?;

            // Update list elements to reflect the new order
            ui_list.update_items(pool).await?;

            // Adjust selection to follow the moved item
            if j + 1 < ui_list.items.len() {
                ui_list.item_state.select(Some(j + 1));
            }
        }
        Ok(())
    }

    /// Render the list of todo items for the selected list
    pub fn render(selected_list: Option<&mut UIList>, area: Rect, buf: &mut Buffer) {
        // Command hints for items
        let list_command_hints = Line::from(vec![
            Span::raw(" "),
            Span::styled(" ↓↑ ", Style::default()),
            Span::styled(
                "[a]",
                Style::default().fg(Color::from_str("#FFA69E").unwrap()),
            ),
            Span::styled(
                "dd",
                Style::default().fg(Color::from_str("#FCF1D5").unwrap()),
            ),
            Span::styled(
                " [d]",
                Style::default().fg(Color::from_str("#FFA69E").unwrap()),
            ),
            Span::styled(
                "el",
                Style::default().fg(Color::from_str("#FCF1D5").unwrap()),
            ),
            Span::styled(
                " [m]",
                Style::default().fg(Color::from_str("#FFA69E").unwrap()),
            ),
            Span::styled(
                "odify ",
                Style::default().fg(Color::from_str("#FCF1D5").unwrap()),
            ),
            Span::raw(" "),
        ])
        .left_aligned();

        // Add "quit" hint, in the bottom right corner
        let quit_hint = Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "[q]",
                Style::default().fg(Color::from_str("#FFA69E").unwrap()),
            ),
            Span::styled(
                "uit ",
                Style::default().fg(Color::from_str("#FCF1D5").unwrap()),
            ),
            Span::raw(" "),
        ])
        .right_aligned();

        let block = Block::default()
            .padding(Padding::new(2, 2, 1, 1))
            .title_top(Line::raw("  I T E M S  ").left_aligned())
            .title_bottom(list_command_hints)
            .title_bottom(quit_hint)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        if let Some(ui_list) = selected_list {
            // Extract the corresponding items with styling
            let items: Vec<ListItem> = ui_list
                .items
                .iter()
                .map(|ui_item| ListItem::from(Self::style_item(ui_item)))
                .collect();

            let list: List = List::new(items)
                .block(block)
                .highlight_symbol(" ▸ ")
                .highlight_style(
                    // Swap foreground and background for selected item
                    Style::default()
                        .bg(Color::from_str("#FCF1D5").unwrap())
                        .fg(Color::from_str("#002626").unwrap()),
                )
                .highlight_spacing(HighlightSpacing::Always);

            StatefulWidget::render(list, area, buf, &mut ui_list.item_state);
        } else {
            // No list selected - render empty block
            block.render(area, buf);
        }
    }
}
