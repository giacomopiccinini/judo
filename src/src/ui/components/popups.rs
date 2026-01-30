use crate::db::config::Config;
use crate::ui::cursor::CursorState;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Widget, Wrap,
};
use std::str::FromStr;

pub struct AddListPopUp;
pub struct ModifyListPopUp;

fn render_list_popup_kernel<T: CursorState>(
    config: Config,
    state: &T,
    area: Rect,
    buf: &mut Buffer,
    popup_title: &str,
) {
    let fg = config.foreground();
    let fg_a = config.foreground_alt();
    let bg = config.background();
    // Command hints for add list popup
    let add_or_modify_list_command_hints = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "[Esc]",
            Style::default().fg(Color::from_str(fg_a).unwrap()),
        ),
        // Span::styled(
        //     "sc",
        //     Style::default().fg(Color::from_str(fg).unwrap()),
        // ),
        Span::raw(" "),
    ]);

    // Calculate popup dimensions
    let popup_width = (area.width * 3) / 4; // 75% of the area width
    let popup_height = 4; // Fixed height for just the input field

    // Center horizontally within the area
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;

    // Center vertically within the area
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;

    // Define the pop-up area
    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the background of the popup area first
    Clear.render(popup_area, buf);
    Block::default()
        .style(Style::default().bg(Color::from_str(bg).unwrap()))
        .render(popup_area, buf);

    // Define the popup block with styling
    let popup_block = Block::new()
        .padding(Padding::new(2, 2, 1, 1))
        .title(format!("  {}  ", popup_title))
        .title_style(Style::new().fg(Color::from_str(fg).unwrap()))
        .title_bottom(add_or_modify_list_command_hints)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::from_str(fg).unwrap()))
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1));

    // Define the text to render
    let text_spans = state.create_cursor_text_spans(config);
    let text_line = Line::from(text_spans);

    // Render the input field
    Paragraph::new(text_line)
        .wrap(Wrap { trim: true })
        .block(popup_block)
        .render(popup_area, buf);
}

impl AddListPopUp {
    /// Render popup for entering a new list name
    pub fn render<T: CursorState>(config: Config, state: &T, area: Rect, buf: &mut Buffer) {
        render_list_popup_kernel(config, state, area, buf, "Add List");
    }
}

impl ModifyListPopUp {
    /// Render popup for entering a new list name
    pub fn render<T: CursorState>(config: Config, state: &T, area: Rect, buf: &mut Buffer) {
        render_list_popup_kernel(config, state, area, buf, "Modify List");
    }
}

pub struct AddItemPopUp;
pub struct ModifyItemPopUp;

/// Render popup for entering a new item name
pub fn render_item_popup_kernel<T: CursorState>(
    config: Config,
    state: &T,
    area: Rect,
    buf: &mut Buffer,
    popup_title: &str,
) {
    let fg = config.foreground();
    let fg_a = config.foreground_alt();
    let bg = config.background();
    // Command hints for add item popup
    let add_item_command_hints = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "[Esc]",
            Style::default().fg(Color::from_str(fg_a).unwrap()),
        ),
        // Span::styled(
        //     "sc",
        //     Style::default().fg(Color::from_str(fg).unwrap()),
        // ),
        Span::raw(" "),
    ]);

    // Calculate popup dimensions
    let popup_width = (area.width * 3) / 4; // 75% of the area width
    let popup_height = 4; // Fixed height for just the input field

    // Center horizontally within the area
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;

    // Center vertically within the area
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;

    // Define the pop-up area
    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the background of the popup area first
    Clear.render(popup_area, buf);
    Block::default()
        .style(Style::default().bg(Color::from_str(bg).unwrap()))
        .render(popup_area, buf);

    // Define the popup block with styling
    let popup_block = Block::new()
        .padding(Padding::new(2, 2, 1, 1))
        .title(format!("  {}  ", popup_title))
        .title_style(Style::new().fg(Color::from_str(fg).unwrap()))
        .title_bottom(add_item_command_hints)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::from_str(fg).unwrap()))
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1));

    // Define the text to render
    let text_spans = state.create_cursor_text_spans(config);
    let text_line = Line::from(text_spans);

    // Render the input field
    Paragraph::new(text_line)
        .wrap(Wrap { trim: true })
        .block(popup_block)
        .render(popup_area, buf);
}

impl AddItemPopUp {
    /// Render popup for entering a new item
    pub fn render<T: CursorState>(config: Config, state: &T, area: Rect, buf: &mut Buffer) {
        render_item_popup_kernel(config, state, area, buf, "Add Item");
    }
}

impl ModifyItemPopUp {
    /// Render popup for modifying item name
    pub fn render<T: CursorState>(config: Config, state: &T, area: Rect, buf: &mut Buffer) {
        render_item_popup_kernel(config, state, area, buf, "Modify Item");
    }
}

pub struct ChangeDBPopUp;

impl ChangeDBPopUp {
    /// Render popup for selecting database
    pub fn render(config: &Config, selected_index: usize, area: Rect, buf: &mut Buffer) {
        let fg = config.foreground();
        let fg_a = config.foreground_alt();
        let bg = config.background();
        // Command hints for change db popup
        let change_db_command_hints = Line::from(vec![
            Span::raw(" "),
            Span::styled(" ↑↓ ", Style::default()),
            Span::styled(
                "[A]",
                Style::default().fg(Color::from_str(fg_a).unwrap()),
            ),
            Span::styled(
                "dd",
                Style::default().fg(Color::from_str(fg).unwrap()),
            ),
            Span::styled(
                " [S]",
                Style::default().fg(Color::from_str(fg_a).unwrap()),
            ),
            Span::styled(
                "et Default",
                Style::default().fg(Color::from_str(fg).unwrap()),
            ),
            Span::styled(
                " [Esc]",
                Style::default().fg(Color::from_str(fg_a).unwrap()),
            ),
            Span::raw(" "),
        ]);

        Block::default()
            .style(
                Style::default()
                    .bg(Color::from_str(bg).unwrap())
                    .fg(Color::from_str(fg).unwrap()),
            )
            .render(area, buf);

        // Define the popup block with styling
        let popup_block = Block::new()
            .padding(Padding::new(2, 2, 1, 1))
            .title(" Select Database ")
            .title_style(Style::new().fg(Color::from_str(fg).unwrap()))
            .title_bottom(change_db_command_hints)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::from_str(fg).unwrap()))
            .border_type(BorderType::Rounded);

        // Create list items from databases
        let items: Vec<ListItem> = config
            .dbs
            .iter()
            .map(|db| ListItem::from(db.name.clone()))
            .collect();

        // Create a mutable list state for rendering
        let mut temp_list_state = ratatui::widgets::ListState::default();
        temp_list_state.select(Some(selected_index));

        // Render the database list
        let list = List::new(items)
            .block(popup_block)
            .highlight_symbol(" ▸ ") // Selection indicator
            .highlight_style(
                // Swap foreground and background for selected item
                Style::default()
                    .bg(Color::from_str(fg).unwrap())
                    .fg(Color::from_str(bg).unwrap()),
            )
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut temp_list_state);
    }
}

pub struct AddDBPopUp;

impl AddDBPopUp {
    /// Render popup for entering a new database name
    pub fn render<T: CursorState>(config: Config, state: &T, area: Rect, buf: &mut Buffer) {
        let fg = config.foreground();
        let fg_a = config.foreground_alt();
        let bg = config.background();
        // Command hints for add db popup
        let add_db_command_hints = Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "[Esc]",
                Style::default().fg(Color::from_str(fg_a).unwrap()),
            ),
            Span::raw(" "),
        ]);

        // Clear the entire area background first
        Clear.render(area, buf);
        Block::default()
            .style(
                Style::default()
                    .bg(Color::from_str(bg).unwrap())
                    .fg(Color::from_str(fg).unwrap()),
            )
            .render(area, buf);

        // Define the popup block with styling - use full width
        let popup_block = Block::new()
            .padding(Padding::new(2, 2, 1, 1))
            .title(" Add Database ")
            .title_style(Style::new().fg(Color::from_str(fg).unwrap()))
            .title_bottom(add_db_command_hints)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::from_str(fg).unwrap()))
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1));

        // Define the text to render
        let text_spans = state.create_cursor_text_spans(config);
        let text_line = Line::from(text_spans);

        // Render the input field using the full area
        Paragraph::new(text_line)
            .wrap(Wrap { trim: true })
            .block(popup_block)
            .render(area, buf);
    }
}
