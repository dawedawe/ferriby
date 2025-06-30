use crate::app::{App, Happiness};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
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

impl App {
    fn get_style() -> Style {
        Style::default().fg(Color::Cyan).bg(Color::Black)
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Sources ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let items = self.sources.iter().map(|source| {
            let s: String = format!("{source}");
            ListItem::new(s)
        });

        let list = List::new(items)
            .block(block)
            .style(App::get_style())
            .highlight_symbol(">> ");
        let mut list_state = ListState::default().with_selected(Some(self.selected));
        StatefulWidget::render(list, area, buf, &mut list_state);
    }
    fn render_ferris(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Ferriby ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let happiness: String = self.happiness.into();
        let ferris = ferris(self.happiness, self.animation);
        let text = format!(
            "Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
             Source: {}\n\
             Happiness level: {}\n\
             {}",
            self.sources[self.selected], happiness, ferris
        );

        Paragraph::new(text)
            .block(block)
            .style(App::get_style())
            .centered()
            .render(area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(area);

        self.render_list(chunks[0], buf);
        self.render_ferris(chunks[1], buf);
    }
}
