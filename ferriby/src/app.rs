use std::fmt::Display;

use crate::{
    event::{AppEvent, Event, EventHandler},
    git::{self, GitSource},
    github::{self, GitHubSource},
};
use chrono::{DateTime, Utc};
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
    /// Repos to monitor.
    pub sources: Vec<Source>,
    /// The currently selected repo.
    pub selected: usize,
    /// Which animation to show.
    pub animation: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(None, None),
            happiness: Happiness::Undecided,
            sources: vec![],
            selected: 0,
            animation: 0,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(sources: Vec<Source>) -> Self {
        let git_intervall_secs = sources
            .iter()
            .find(|source| matches!(source, Source::Git(_)))
            .map(|_| 3.0);

        let gh_intervall_secs = {
            let gh_source = sources.iter().find_map(|source| match source {
                Source::GitHub(x) => Some(x),
                _ => None,
            });
            match gh_source {
                Some(gh_source) if gh_source.pat.is_some() => Some(5.0),
                Some(_) => Some(60.0),
                _ => None,
            }
        };

        Self {
            running: true,
            events: EventHandler::new(git_intervall_secs, gh_intervall_secs),
            happiness: Happiness::Undecided,
            sources,
            selected: 0,
            animation: 0,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::GitTick => self.git_tick().await,
                Event::GitHubTick => self.github_tick().await,
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
            KeyCode::Down => {
                self.happiness = Happiness::Undecided;
                self.selected = (self.selected + 1) % self.sources.len();
            }
            KeyCode::Up => {
                self.happiness = Happiness::Undecided;
                self.selected = {
                    if self.selected == 0 {
                        self.sources.len() - 1
                    } else {
                        self.selected.saturating_sub(1)
                    }
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    async fn git_tick(&mut self) {
        if let Source::Git(source) = &self.sources[self.selected] {
            let last_event = tokio::spawn(git::get_last_event(source.clone())).await;
            match last_event {
                Ok(last_event) => {
                    self.happiness = Happiness::from_last_activity(last_event);
                }
                Err(_) => self.running = false,
            }
        };
    }

    /// Handles the github_tick event of the terminal.
    async fn github_tick(&mut self) {
        if let Source::GitHub(source) = &self.sources[self.selected] {
            let last_event = tokio::spawn(github::get_last_event(source.clone())).await;
            match last_event {
                Ok(last_event) => {
                    self.happiness = Happiness::from_last_activity(last_event);
                }
                Err(_) => self.running = false,
            }
        };
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
    use github::GitHubSource;

    #[test]
    fn github_display() {
        let source = Source::GitHub(GitHubSource {
            owner: "owner_name".into(),
            repo: "repo_name".into(),
            pat: None,
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
