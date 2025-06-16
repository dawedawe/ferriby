use std::fmt::Display;

use crate::event::{AppEvent, Event, EventHandler};
use chrono::{DateTime, Utc};
use ferriby_sources::{
    git::{self, GitSource},
    github::{self, GitHubSource},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

#[derive(Debug, Clone)]
pub enum Source {
    GitHub(GitHubSource),
    Git(GitSource),
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::GitHub(gh_source) => write!(f, "{}/{}", gh_source.owner, gh_source.repo),
            Source::Git(git_source) => write!(f, "{}", git_source.path),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Happiness {
    Undecided,
    Sad,
    Okayish,
    Buzzing,
}

impl Happiness {
    fn from_last_activity(last_activity: Option<DateTime<Utc>>) -> Self {
        if let Some(last_activity) = last_activity {
            let now = chrono::Utc::now();
            if now < last_activity {
                panic!("commits from the future");
            }
            let diff = now - last_activity;
            match diff {
                _ if diff < chrono::TimeDelta::hours(24) => Happiness::Buzzing,
                _ if diff < chrono::TimeDelta::hours(24 * 7) => Happiness::Okayish,
                _ => Happiness::Sad,
            }
        } else {
            Happiness::Undecided
        }
    }
}

impl From<Happiness> for String {
    fn from(happiness: Happiness) -> Self {
        match happiness {
            Happiness::Undecided => "undecided".into(),
            Happiness::Sad => "sad".into(),
            Happiness::Okayish => "okayish".into(),
            Happiness::Buzzing => "buzzing".into(),
        }
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// How happy we are.
    pub happiness: Happiness,
    /// Event handler.
    pub events: EventHandler,
    /// GitHub source.
    pub source: Source,
    /// Which animation to show.
    pub animation: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(60),
            happiness: Happiness::Okayish,
            source: Source::Git(GitSource::default()),
            animation: 0,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(source: Source) -> Self {
        let intervall_secs = match (&source, std::env::var("FERRIBY_GH_PAT")) {
            (Source::Git(_), _) => 3,
            (Source::GitHub(_), Ok(e)) if !e.is_empty() => 5,
            _ => 60,
        };

        Self {
            running: true,
            events: EventHandler::new(intervall_secs),
            happiness: Happiness::Undecided,
            source,
            animation: 0,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick().await,
                Event::AnimationTick => self.animation_tick(),
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_events(key_event)?
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    async fn tick(&mut self) {
        let last_event = match &self.source {
            Source::GitHub(source) => tokio::spawn(github::get_last_event(source.clone())).await,
            Source::Git(source) => tokio::spawn(git::get_last_event(source.clone())).await,
        };
        match last_event {
            Ok(last_event) => {
                self.happiness = Happiness::from_last_activity(last_event);
            }
            Err(_) => self.running = false,
        }
    }

    /// Handles the animation_tick event of the terminal.
    fn animation_tick(&mut self) {
        self.animation = self.animation.wrapping_add(1);
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferriby_sources::github::GitHubSource;

    #[test]
    fn github_display() {
        let source = Source::GitHub(GitHubSource {
            owner: "owner_name".into(),
            repo: "repo_name".into(),
        });
        let s = format!("{}", source);
        assert_eq!("owner_name/repo_name", s);
    }

    #[test]
    fn git_display() {
        let source = Source::Git(GitSource {
            path: "abc/cde/fgh".into(),
        });
        let s = format!("{}", source);
        assert_eq!("abc/cde/fgh", s);
    }
}
