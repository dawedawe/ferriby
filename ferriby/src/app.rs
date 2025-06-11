use crate::event::{AppEvent, Event, EventHandler};
use ferriby_sources::github;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Signal.
    pub exists: bool,
    /// Event handler.
    pub events: EventHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            exists: false,
            events: EventHandler::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
        let last_event =
            tokio::spawn(github::get_last_gh_repo_event("dawedawe", "nvim_config")).await;
        let last_event = last_event.unwrap();
        let happy = match last_event {
            Some(last_event) => {
                let now = chrono::Utc::now();
                let diff = now - last_event;
                diff < chrono::TimeDelta::hours(6)
            }
            None => false,
        };

        self.exists = happy;
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
