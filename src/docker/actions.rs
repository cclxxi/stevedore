//! Container lifecycle actions: start / stop / restart / remove.
//!
//! Each action is a short, one-shot Docker call. The UI spawns it as a
//! fire-and-forget task ([`spawn`]) and learns the outcome through an
//! [`AppMessage`] — success as [`AppMessage::Status`], failure as
//! [`AppMessage::Error`] — never by mutating [`crate::app::App`] from the task.

use bollard::query_parameters::{
    RemoveContainerOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions,
};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::docker::DockerClient;
use crate::event::AppMessage;

/// A lifecycle action the user can run against the selected container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    Restart,
    Remove,
}

impl Action {
    /// Lowercase verb used in prompts and error messages ("stop", "remove").
    pub fn verb(self) -> &'static str {
        match self {
            Action::Start => "start",
            Action::Stop => "stop",
            Action::Restart => "restart",
            Action::Remove => "remove",
        }
    }

    /// Past-tense verb used in success messages ("Stopped", "Removed").
    fn done(self) -> &'static str {
        match self {
            Action::Start => "Started",
            Action::Stop => "Stopped",
            Action::Restart => "Restarted",
            Action::Remove => "Removed",
        }
    }

    /// Whether the action destroys state and must be confirmed first.
    pub fn is_destructive(self) -> bool {
        matches!(self, Action::Remove)
    }
}

/// Run `action` against the container in the background, reporting the outcome.
///
/// `name` is only used to build a human-readable status line; the Docker call
/// itself targets the stable `container_id`.
pub fn spawn(
    docker: DockerClient,
    tx: Sender<AppMessage>,
    action: Action,
    container_id: String,
    name: String,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let message = match run(&docker, action, &container_id).await {
            Ok(()) => AppMessage::Status(format!("{} {name}", action.done())),
            Err(e) => AppMessage::Error(format!("Failed to {} {name}: {e}", action.verb())),
        };
        let _ = tx.send(message).await;
    })
}

/// Perform the actual Docker call for `action`, using daemon-default options.
async fn run(docker: &DockerClient, action: Action, id: &str) -> Result<(), String> {
    let client = docker.inner();
    let result = match action {
        Action::Start => {
            client
                .start_container(id, None::<StartContainerOptions>)
                .await
        }
        Action::Stop => {
            client
                .stop_container(id, None::<StopContainerOptions>)
                .await
        }
        Action::Restart => {
            client
                .restart_container(id, None::<RestartContainerOptions>)
                .await
        }
        Action::Remove => {
            client
                .remove_container(id, None::<RemoveContainerOptions>)
                .await
        }
    };
    result.map_err(|e| e.to_string())
}
