//! Error types for connection-time failures.
//!
//! Runtime failures (a single failed stats/log call) are surfaced to the user
//! through the status line as `AppMessage::Error` strings rather than crashing
//! the TUI. This module covers the fatal, startup-time errors only.

use thiserror::Error;

/// Fatal errors that prevent the application from starting.
#[derive(Debug, Error)]
pub enum AppError {
    /// Could not reach or talk to the Docker daemon (startup-time check).
    #[error("cannot reach the Docker daemon: {0}")]
    DockerUnreachable(String),

    /// A Docker API call failed at runtime (listing, stats, etc.).
    #[error("docker error: {0}")]
    Docker(String),

    /// Terminal initialization failed.
    #[error("terminal error: {0}")]
    Terminal(#[from] std::io::Error),
}

impl AppError {
    /// A short, user-facing hint shown when the daemon is unreachable.
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            AppError::DockerUnreachable(_) => Some(
                "Is Docker running? Check that the daemon is up and that your user \
                 can access the socket (try the `docker` group or sudo).",
            ),
            AppError::Docker(_) | AppError::Terminal(_) => None,
        }
    }
}
