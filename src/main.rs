//! stevedore — a super-lightweight TUI for monitoring Docker containers and logs.

mod app;
mod docker;
mod error;
mod event;
mod ui;
mod util;

use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::interval;

use app::{App, Context};
use docker::DockerClient;
use error::AppError;
use event::AppMessage;

/// How often the container list is refreshed.
const REFRESH_INTERVAL: Duration = Duration::from_secs(2);
/// Redraw cadence, so live data appears even without input.
const TICK_INTERVAL: Duration = Duration::from_millis(250);
/// Capacity of the task → UI message channel.
const CHANNEL_CAPACITY: usize = 256;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("stevedore: {error}");
        if let Some(hint) = error.hint() {
            eprintln!("hint: {hint}");
        }
        std::process::exit(1);
    }
}

/// Set up Docker, spawn background tasks, and run the TUI to completion.
async fn run() -> Result<(), AppError> {
    let docker = DockerClient::connect()?;
    // First real contact with the daemon — fails fast with a friendly hint.
    let version = docker.version().await?;

    let (tx, rx) = mpsc::channel::<AppMessage>(CHANNEL_CAPACITY);
    let refresh = tokio::spawn(refresh_loop(docker.clone(), tx.clone()));
    let stats = tokio::spawn(docker::stats::run(docker.clone(), tx.clone()));

    let ctx = Context { docker, tx };
    let mut app = App::new(version);

    let mut terminal = ratatui::try_init()?;
    let result = event_loop(&mut terminal, &mut app, &ctx, rx).await;
    ratatui::restore();

    // Stop background tasks promptly rather than waiting for the runtime to
    // drop them on the next interval tick.
    refresh.abort();
    stats.abort();
    result
}

/// The main render + input loop.
async fn event_loop(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    ctx: &Context,
    mut rx: Receiver<AppMessage>,
) -> Result<(), AppError> {
    let mut input = EventStream::new();
    let mut tick = interval(TICK_INTERVAL);

    loop {
        if app.should_quit {
            return Ok(());
        }
        terminal.draw(|frame| ui::render(frame, app))?;

        tokio::select! {
            maybe_input = input.next() => match maybe_input {
                Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                    app.on_key(key, ctx);
                }
                Some(Ok(_)) => {}
                Some(Err(_)) | None => return Ok(()),
            },
            maybe_msg = rx.recv() => {
                if let Some(message) = maybe_msg {
                    app.apply_message(message);
                }
            }
            _ = tick.tick() => {}
        }
    }
}

/// Periodically refresh the full container list (all states).
async fn refresh_loop(docker: DockerClient, tx: Sender<AppMessage>) {
    let mut tick = interval(REFRESH_INTERVAL);
    loop {
        tick.tick().await;
        let message = match docker.list_containers(true).await {
            Ok(list) => AppMessage::Containers(list),
            Err(error) => AppMessage::Error(error.to_string()),
        };
        if tx.send(message).await.is_err() {
            return; // UI gone — stop refreshing.
        }
    }
}
