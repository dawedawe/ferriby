use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" ferriby ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let text = format!(
            "Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Happy: {}",
            self.exists
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        paragraph.render(area, buf);
    }
}
