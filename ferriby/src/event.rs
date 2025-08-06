use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use std::time::Duration;
use tokio::{
    sync::mpsc,
    task::{JoinHandle, JoinSet},
};

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted when it's time to check git.
    GitTick,
    /// An event that is emitted when it's time to check GitHub.
    GitHubTick,
    /// An event that is emitted when it's time to check GitLab.
    GitLabTick,
    /// An event that is emitted when it's time to check Forgejo.
    ForgejoTick,
    /// Event emitted when it's time to animate ferris.
    AnimationTick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

/// Application events.
///
/// You can extend this enum with your own custom events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Quit the application.
    Quit,
}

/// The intervals of the sources
#[derive(Clone, Debug, Default)]
pub struct IntervalSecs {
    /// The interval for git checks.
    pub git: Option<f32>,
    /// The interval for GitHub checks.
    pub github: Option<f32>,
    /// The interval for GitLab checks.
    pub gitlab: Option<f32>,
    /// The interval for Forgejo checks.
    pub forgejo: Option<f32>,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// The intervals
    interval_secs: IntervalSecs,
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// The EventTask task
    actor_task: JoinHandle<Result<(), color_eyre::eyre::Error>>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new(interval_secs: IntervalSecs) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone(), interval_secs.clone());
        let actor_task = tokio::spawn(async { actor.run().await });
        Self {
            interval_secs,
            sender,
            receiver,
            actor_task,
        }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }

    /// Restart the EventTask actor to have fast updates after a change of the selected source
    pub fn restart(&mut self) {
        self.actor_task.abort();
        let actor = EventTask::new(self.sender.clone(), self.interval_secs.clone());
        self.actor_task = tokio::spawn(async { actor.run().await });
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        let intervals = IntervalSecs::default();
        Self::new(intervals)
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    interval_secs: IntervalSecs,
}

impl EventTask {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::UnboundedSender<Event>, interval_secs: IntervalSecs) -> Self {
        Self {
            sender,
            interval_secs,
        }
    }

    async fn key_thread(sender: mpsc::UnboundedSender<Event>) {
        let mut reader = crossterm::event::EventStream::new();
        loop {
            let crossterm_event = reader.next().fuse();
            if let Some(Ok(evt)) = crossterm_event.await {
                let _ = sender.send(Event::Crossterm(evt));
            }
        }
    }

    async fn tick_thread(sender: mpsc::UnboundedSender<Event>, event: Event, interval_secs: f32) {
        let tick_rate = Duration::from_secs_f32(interval_secs);
        let mut tick = tokio::time::interval(tick_rate);
        loop {
            let _ = sender.send(event.clone());
            let tick_delay = tick.tick();
            let _ = tick_delay.await;
        }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> color_eyre::Result<()> {
        let mut set = JoinSet::new();
        let keyevent_sender = self.sender.clone();
        set.spawn(async move { EventTask::key_thread(keyevent_sender).await });

        let animation_sender = self.sender.clone();
        set.spawn(async move {
            EventTask::tick_thread(animation_sender, Event::AnimationTick, 0.7).await
        });

        if let Some(secs) = self.interval_secs.git {
            let tick_sender = self.sender.clone();
            set.spawn(
                async move { EventTask::tick_thread(tick_sender, Event::GitTick, secs).await },
            );
        };

        if let Some(secs) = self.interval_secs.github {
            let tick_sender = self.sender.clone();
            set.spawn(
                async move { EventTask::tick_thread(tick_sender, Event::GitHubTick, secs).await },
            );
        };

        if let Some(secs) = self.interval_secs.gitlab {
            let tick_sender = self.sender.clone();
            set.spawn(
                async move { EventTask::tick_thread(tick_sender, Event::GitLabTick, secs).await },
            );
        };

        if let Some(secs) = self.interval_secs.forgejo {
            let tick_sender = self.sender.clone();
            set.spawn(async move {
                EventTask::tick_thread(tick_sender, Event::ForgejoTick, secs).await
            });
        };

        let _ = set.join_all().await;
        Ok(())
    }
}
