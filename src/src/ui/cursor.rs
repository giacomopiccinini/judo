use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::str::FromStr;
use crate::db::config::Config;

/// Trait for managing cursor-based text input
pub trait CursorState {
    /// Get the current text content
    fn get_text(&self) -> &str;

    /// Get a mutable reference to the text content
    fn get_text_mut(&mut self) -> &mut String;

    /// Get the current cursor position
    fn get_cursor_pos(&self) -> usize;

    /// Set the cursor position
    fn set_cursor_pos(&mut self, pos: usize);

    /// Add a character at the cursor position
    /// Beware because some chars could be multi-byte.
    /// That's why we need to distinguish the char and byte position
    fn add_char(&mut self, c: char) {
        let char_pos = self.get_cursor_pos();
        let chars: Vec<char> = self.get_text().chars().collect();

        // Convert character position to byte position
        let byte_pos = if char_pos == 0 {
            0
        } else if char_pos >= chars.len() {
            self.get_text().len()
        } else {
            chars[..char_pos].iter().map(|c| c.len_utf8()).sum()
        };

        self.get_text_mut().insert(byte_pos, c);
        self.set_cursor_pos(char_pos + 1);
    }

    /// Remove the character before the cursor (backspace)
    fn remove_char_before_cursor(&mut self) {
        let char_pos = self.get_cursor_pos();
        if char_pos > 0 {
            let chars: Vec<char> = self.get_text().chars().collect();
            let byte_pos = chars[..char_pos - 1].iter().map(|c| c.len_utf8()).sum();
            self.get_text_mut().remove(byte_pos);
            self.set_cursor_pos(char_pos - 1);
        }
    }

    /// Delete the character after the cursor (delete key)
    fn delete_char_after_cursor(&mut self) {
        let pos = self.get_cursor_pos();
        let text_len = self.get_text().chars().count();

        if pos < text_len {
            let mut chars: Vec<char> = self.get_text().chars().collect();
            chars.remove(pos);
            *self.get_text_mut() = chars.into_iter().collect();
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        let pos = self.get_cursor_pos();
        if pos > 0 {
            self.set_cursor_pos(pos - 1);
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        let pos = self.get_cursor_pos();
        let text_len = self.get_text().chars().count();
        if pos < text_len {
            self.set_cursor_pos(pos + 1);
        }
    }

    /// Clear the text and reset cursor
    fn clear(&mut self) {
        self.get_text_mut().clear();
        self.set_cursor_pos(0);
    }

    /// Create text spans for rendering with cursor visualization
    fn create_cursor_text_spans(&self, config: Config) -> Vec<Span<'static>> {
        let fg = config.foreground();
        let bg = config.background();
        let text = self.get_text();
        let cursor_pos = self.get_cursor_pos();
        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len();

        // Ensure cursor position is within bounds
        let safe_cursor_pos = cursor_pos.min(text_len);

        // Text before cursor
        let text_before: String = chars[..safe_cursor_pos].iter().collect();

        // Character at cursor position (or space if at end)
        let cursor_char = if safe_cursor_pos >= text_len {
            "█".to_string()
        } else {
            chars[safe_cursor_pos].to_string()
        };

        // Text after cursor
        let text_after: String = if safe_cursor_pos >= text_len {
            String::new()
        } else {
            chars[(safe_cursor_pos + 1)..].iter().collect()
        };

        vec![
            Span::styled(
                text_before,
                Style::default().fg(Color::from_str(fg).unwrap()),
            ),
            if cursor_char == "█" {
                Span::styled(
                    cursor_char,
                    Style::default()
                        .fg(Color::from_str(fg).unwrap())
                        .bg(Color::from_str(bg).unwrap()),
                )
            } else {
                Span::styled(
                    cursor_char,
                    Style::default()
                        .fg(Color::from_str(bg).unwrap())
                        .bg(Color::from_str(fg).unwrap()),
                )
            },
            Span::styled(
                text_after,
                Style::default().fg(Color::from_str(fg).unwrap()),
            ),
        ]
    }
}
