pub struct DBSelector;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget};
use std::str::FromStr;
use crate::db::config::Config;

impl DBSelector {
    pub fn render(area: Rect, buf: &mut Buffer, current_db_name: &str, config: Config) {
        let fg = config.foreground();
        let fg_a = config.foreground_alt();

        // Command hints for db
        let list_command_hints = Line::from(vec![
            Span::raw(" "),
            Span::styled(
                "[C]",
                Style::default().fg(Color::from_str(fg_a).unwrap()),
            ),
            Span::styled(
                "hange",
                Style::default().fg(Color::from_str(fg).unwrap()),
            ),
            Span::raw(" "),
        ])
        .left_aligned();

        let block = Block::default()
            .padding(Padding::new(2, 2, 0, 0))
            .title_top(Line::raw("  D A T A B A S E  ").left_aligned())
            .title_bottom(list_command_hints)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        Paragraph::new(current_db_name)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
