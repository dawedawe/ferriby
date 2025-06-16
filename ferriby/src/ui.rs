use crate::app::{App, Happiness};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

fn ferris(happiness: Happiness, animation: usize) -> String {
    let undecided_ferris = {
        let ferrises = [
            r"
    _~^~^~_        
   / o  o  \       
  '_       _'      
  \ '-----' /      
",
            r"
    _~^~^~_       
   /  o  o \      
  '_       _'     
  \ '-----' /     
",
        ];
        ferrises[animation % ferrises.len()]
    };

    let sad_ferris = {
        let ferrises = [
            r"
    _~^~^~_       
\) / .  .  \ (/   
  '_  / \  _'     
  \ '-----' \     
",
            r"
    _~^~^~_       
\) /  .  . \ (/   
  '_  / \  _'     
  / '-----' /     
",
        ];
        ferrises[animation % ferrises.len()]
    };

    let okayish_ferris = {
        let ferrises = [
            r"
    _~^~^~_       
\) /  o o  \ (/   
  '_   ==  _'     
  \ '-----' /     
",
            r"
    _~^~^~_       
\) /  o o  \ (/   
  '_  ==   _'     
  \ '-----' /     
",
        ];

        ferrises[animation % ferrises.len()]
    };

    let buzzing_ferris = {
        let ferrises = [
            r"
    _~^~^~_       
\/ /  o O  \ \/   
  '_  \_/  _'     
  \ '-----' /     
",
            r"
\/  _~^^^~_  \/   
 \ /  O o  \ /    
  '_  *o*  _'     
  / '-----' \     
",
            r"
    _~^~^~_       
\/ /  o O  \ \/   
  '_  \_/  _'     
  \ '-----' /     
",
            r"
    _~^~^~_       
\  /  O -  \  /   
  '_  \_/  _'     
  \ '-----' /     
",
        ];

        ferrises[animation % ferrises.len()]
    };

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
        let ferris = ferris(self.happiness, self.animation);
        let text = format!(
            "Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
             Source: {}\n\
             Happiness level: {}\n\
             {}",
            self.source, happiness, ferris
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        paragraph.render(area, buf);
    }
}
