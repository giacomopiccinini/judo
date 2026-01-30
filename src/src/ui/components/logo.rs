pub struct Logo;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};

// Use the judo ascii logo
const ASCII_LOGO: &str = r#"
     ██╗██╗   ██╗██████╗  ██████╗ 
     ██║██║   ██║██╔══██╗██╔═══██╗
     ██║██║   ██║██║  ██║██║   ██║
██   ██║██║   ██║██║  ██║██║   ██║
╚█████╔╝╚██████╔╝██████╔╝╚██████╔╝
 ╚════╝  ╚═════╝ ╚═════╝  ╚═════╝ 
        "#;

impl Logo {
    pub fn render(area: Rect, buf: &mut Buffer) {
        // Define a block and pad
        let block = Block::default().padding(Padding::horizontal(2));

        Paragraph::new(ASCII_LOGO)
            .bold()
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
