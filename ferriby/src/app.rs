use crate::event::{AppEvent, Event, EventHandler};
use chrono::{DateTime, Utc};
use ferriby_sources::github::{self, GitHubSource};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

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
    pub source: GitHubSource,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(60),
            happiness: Happiness::Okayish,
            source: GitHubSource {
                owner: "rust".into(),
                repo: "rust".into(),
            },
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(source: GitHubSource) -> Self {
        let gh_intervall_secs = match std::env::var("FERRIBY_GH_PAT") {
            Ok(e) if !e.is_empty() => 5,
            _ => 60,
        };

        Self {
            running: true,
            events: EventHandler::new(gh_intervall_secs),
            happiness: Happiness::Okayish,
            source,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick().await,
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
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    async fn tick(&mut self) {
        let last_event = tokio::spawn(github::get_last_gh_repo_event(self.source.clone())).await;
        match last_event {
            Ok(last_event) => {
                self.happiness = Happiness::from_last_activity(last_event);
            }
            Err(_) => self.running = false,
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
