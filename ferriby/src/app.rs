use std::fmt::Display;

use crate::{
    event::{AppEvent, Event, EventHandler, IntervalSecs},
    forgejo::ForgejoSource,
    git::GitSource,
    github::GitHubSource,
    gitlab::GitLabSource,
};
use chrono::{DateTime, Utc};
use crossterm::event::KeyEventKind;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use tokio::task::JoinError;

pub trait ActivitySource {
    fn get_last_activity(self) -> impl Future<Output = Option<DateTime<Utc>>>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Git(GitSource),
    GitHub(GitHubSource),
    GitLab(GitLabSource),
    Forgejo(ForgejoSource),
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Git(source) => write!(f, "git: {}", source.path),
            Source::GitHub(source) => write!(f, "github: {}/{}", source.owner, source.repo),
            Source::GitLab(source) => {
                write!(f, "{}: {}", source.hostname, source.project_name)
            }
            Source::Forgejo(source) => {
                write!(
                    f,
                    "{}: {}/{}",
                    source
                        .base_url
                        .host_str()
                        .expect("expected a Url with host part"),
                    source.owner,
                    source.repo
                )
            }
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
            events: EventHandler::new(IntervalSecs::default()),
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
        let git_interval_secs = sources
            .iter()
            .find(|source| matches!(source, Source::Git(_)))
            .map(|_| 3.0);

        let gh_interval_secs = {
            let source = sources.iter().find_map(|source| match source {
                Source::GitHub(x) => Some(x),
                _ => None,
            });
            match source {
                Some(source) if source.pat.is_some() => Some(5.0),
                Some(_) => Some(60.0),
                _ => None,
            }
        };

        let gl_interval_secs = {
            let source = sources.iter().find_map(|source| match source {
                Source::GitLab(x) => Some(x),
                _ => None,
            });
            match source {
                Some(source) if source.pat.is_some() => Some(5.0),
                Some(_) => Some(60.0),
                _ => None,
            }
        };

        let fj_interval_secs = {
            let source = sources.iter().find_map(|source| match source {
                Source::Forgejo(x) => Some(x),
                _ => None,
            });
            match source {
                Some(source) if source.pat.is_some() => Some(5.0),
                Some(_) => Some(60.0),
                _ => None,
            }
        };

        let intervals = IntervalSecs {
            git: git_interval_secs,
            github: gh_interval_secs,
            gitlab: gl_interval_secs,
            forgejo: fj_interval_secs,
        };

        Self {
            running: true,
            events: EventHandler::new(intervals),
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
                Event::GitLabTick => self.gitlab_tick().await,
                Event::ForgejoTick => self.forgejo_tick().await,
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
            KeyCode::Down if key_event.kind == KeyEventKind::Press => {
                self.happiness = Happiness::Undecided;
                self.selected = (self.selected + 1) % self.sources.len();
                self.events.restart();
            }
            KeyCode::Up if key_event.kind == KeyEventKind::Press => {
                self.happiness = Happiness::Undecided;
                self.selected = {
                    if self.selected == 0 {
                        self.sources.len() - 1
                    } else {
                        self.selected.saturating_sub(1)
                    }
                };
                self.events.restart();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle the last_activity
    fn handle_last_activity(&mut self, last_activity: Result<Option<DateTime<Utc>>, JoinError>) {
        match last_activity {
            Ok(last_event) => {
                self.happiness = Happiness::from_last_activity(last_event);
            }
            Err(_) => self.running = false,
        }
    }

    /// Handles the git_tick event.
    async fn git_tick(&mut self) {
        if let Source::Git(source) = &self.sources[self.selected] {
            let last_activity = tokio::spawn(source.clone().get_last_activity()).await;
            self.handle_last_activity(last_activity);
        };
    }

    /// Handles the github_tick.
    async fn github_tick(&mut self) {
        if let Source::GitHub(source) = &self.sources[self.selected] {
            let last_activity = tokio::spawn(source.clone().get_last_activity()).await;
            self.handle_last_activity(last_activity);
        };
    }

    /// Handles the gitlab_tick.
    async fn gitlab_tick(&mut self) {
        if let Source::GitLab(source) = &self.sources[self.selected] {
            let last_activity = tokio::spawn(source.clone().get_last_activity()).await;
            self.handle_last_activity(last_activity);
        };
    }

    /// Handles the codeberg_tick event.
    async fn forgejo_tick(&mut self) {
        if let Source::Forgejo(source) = &self.sources[self.selected] {
            let last_activity = tokio::spawn(source.clone().get_last_activity()).await;
            self.handle_last_activity(last_activity);
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
    use crate::github::GitHubSource;
    use reqwest::Url;

    #[test]
    fn github_display() {
        let source = Source::GitHub(GitHubSource {
            owner: "owner_name".into(),
            repo: "repo_name".into(),
            pat: None,
        });
        let s = format!("{source}");
        assert_eq!("github: owner_name/repo_name", s);
    }

    #[test]
    fn git_display() {
        let source = Source::Git(GitSource {
            path: "abc/cde/fgh".into(),
        });
        let s = format!("{source}");
        assert_eq!("git: abc/cde/fgh", s);
    }

    #[test]
    fn forgejo_display() {
        let source = Source::Forgejo(ForgejoSource {
            base_url: Url::parse("http://localhost").unwrap(),
            owner: "owner_name".into(),
            repo: "repo_name".into(),
            pat: None,
        });
        let s = format!("{source}");
        assert_eq!("localhost: owner_name/repo_name", s);
    }
}
