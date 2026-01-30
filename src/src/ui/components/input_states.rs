use crate::ui::cursor::CursorState;

/// State of input from user
#[derive(Debug, Clone)]
pub struct InputState {
    /// Buffer for input string
    pub current_input: String,
    /// Position of cursor
    pub cursor_pos: usize,
    /// Flag where true indicates item is being modified and not created from scratched
    pub is_modifying: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
            cursor_pos: 0,
            is_modifying: false,
        }
    }
}

impl CursorState for InputState {
    fn get_text(&self) -> &str {
        &self.current_input
    }

    fn get_text_mut(&mut self) -> &mut String {
        &mut self.current_input
    }

    fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor_pos = pos;
    }
}
