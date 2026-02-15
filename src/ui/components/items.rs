use crate::db::config::Config;
use crate::db::models::{NewTodoItem, TodoItem, UIItem, UIList};
use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, HighlightSpacing, List, ListItem, Padding, StatefulWidget, Widget,
};
use sqlx::SqlitePool;
use std::str::FromStr;
use textwrap::wrap;
pub struct ItemsComponent;

impl ItemsComponent {
    /// Return the style for a todo item based on its completion status
    fn item_style(ui_item: &UIItem) -> Style {
        if ui_item.item.is_done {
            // Strike through completed items
            Style::default().add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
        }
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
        ui_list.item_state.select_first();
    }

    /// Select the last item in the list
    pub fn select_last_item(ui_list: &mut UIList) {
        ui_list.item_state.select_last();
    }

    // Format all items in a list ready to be copied
    pub fn format_all_items(ui_list: &mut UIList) -> String {
        ui_list
            .items
            .iter()
            .map(|ui_item| format!("- {}", ui_item.item.name))
            .collect::<Vec<_>>()
            .join("\n")
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
    pub fn render(
        selected_list: Option<&mut UIList>,
        area: Rect,
        buf: &mut Buffer,
        config: Config,
    ) {
        let fg = config.foreground();
        let hl = config.highlight();
        let bg = config.background();
        // Command hints for items
        let list_command_hints = Line::from(vec![
            Span::raw(" "),
            Span::styled(" ↓↑ ", Style::default()),
            Span::styled("[a]", Style::default().fg(Color::from_str(hl).unwrap())),
            Span::styled("dd", Style::default().fg(Color::from_str(fg).unwrap())),
            Span::styled(" [d]", Style::default().fg(Color::from_str(hl).unwrap())),
            Span::styled("el", Style::default().fg(Color::from_str(fg).unwrap())),
            Span::styled(" [m]", Style::default().fg(Color::from_str(hl).unwrap())),
            Span::styled("odify", Style::default().fg(Color::from_str(fg).unwrap())),
            Span::styled(" [c]", Style::default().fg(Color::from_str(hl).unwrap())),
            Span::styled(
                "opy items ",
                Style::default().fg(Color::from_str(fg).unwrap()),
            ),
            Span::raw(" "),
        ])
        .left_aligned();

        // Add "quit" hint, in the bottom right corner
        let quit_hint = Line::from(vec![
            Span::raw(" "),
            Span::styled("[q]", Style::default().fg(Color::from_str(hl).unwrap())),
            Span::styled("uit ", Style::default().fg(Color::from_str(fg).unwrap())),
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
            // Calculate available width for text wrapping
            // Account for: highlight symbol " ▸ " (4 chars) + padding (2+2) + borders (2)
            let highlight_symbol = " ▸ ";
            let highlight_width = highlight_symbol.chars().count();
            let available_width = area.width.saturating_sub(highlight_width as u16 + 6) as usize;

            // Wrap each item's content to fit the available width
            let items: Vec<ListItem> = ui_list
                .items
                .iter()
                .map(|ui_item| {
                    let name = &ui_item.item.name;
                    let style = Self::item_style(ui_item);

                    let wrapped_lines: Vec<Line> = if available_width > 0 {
                        wrap(name, available_width)
                            .iter()
                            .map(|line| Line::from(Span::styled(line.to_string(), style)))
                            .collect()
                    } else {
                        vec![Line::from(Span::styled(name.clone(), style))]
                    };

                    ListItem::new(Text::from(wrapped_lines))
                })
                .collect();

            let list: List = List::new(items)
                .block(block)
                .highlight_symbol(highlight_symbol)
                .highlight_style(
                    // Swap foreground and background for selected item
                    Style::default()
                        .bg(Color::from_str(fg).unwrap())
                        .fg(Color::from_str(bg).unwrap()),
                )
                .highlight_spacing(HighlightSpacing::Always);

            StatefulWidget::render(list, area, buf, &mut ui_list.item_state);
        } else {
            // No list selected - render empty block
            block.render(area, buf);
        }
    }
}
