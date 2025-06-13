use crate::app::{App, Happiness};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

fn ferris(happiness: Happiness) -> String {
    let undecided_ferris = r"
    _~^~^~_      
\) /  o o  \ (/  
  '_       _'    
  \ '-----' /    
";

    let sad_ferris = r"
    _~^~^~_       
\) /  ~ ~  \ (/   
  '_  / \  _'     
  \ '-----' /     
";

    let okayish_ferris = r"
    _~^~^~_       
\) /  o o  \ (/   
  '_  ---  _'     
  \ '-----' /     
";

    let buzzing_ferris = r"
    _~^~^~_       
\/ /  O O  \ \/   
  '_  \_/  _'     
  \ '-----' /     
";

    match happiness {
        Happiness::Undecided => undecided_ferris.into(),
        Happiness::Sad => sad_ferris.into(),
        Happiness::Okayish => okayish_ferris.into(),
        Happiness::Buzzing => buzzing_ferris.into(),
    }
}

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

        let happiness: String = self.happiness.into();
        let ferris = ferris(self.happiness);
        let text = format!(
            "Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
             Happiness level: {}\n
             {}",
            happiness, ferris
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        paragraph.render(area, buf);
    }
}
