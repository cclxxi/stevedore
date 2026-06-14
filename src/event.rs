//! Messages sent from background tasks to the main UI loop.
//!
//! All Docker I/O happens off the render thread. Tasks communicate results back
//! to the single-owner [`crate::app::App`] through an mpsc channel of these
//! messages, which keeps the app state free of shared mutation.

use std::collections::HashMap;

use crate::docker::model::{Container, ContainerStats};

/// An update for the main loop to apply to application state.
///
/// Intentionally not `Clone`: these travel one-way over a move channel and are
/// never duplicated, so cloning a potentially large `Stats` map would be a bug.
#[derive(Debug)]
pub enum AppMessage {
    /// A fresh snapshot of the container list.
    Containers(Vec<Container>),
    /// Live resource stats keyed by container id.
    Stats(HashMap<String, ContainerStats>),
    /// A single log line for the given container.
    LogLine { container_id: String, line: String },
    /// The log stream for the given container ended (container stopped or error).
    LogEnded { container_id: String },
    /// A transient success/info message to show on the status line.
    Status(String),
    /// A non-fatal error to show on the status line.
    Error(String),
}
