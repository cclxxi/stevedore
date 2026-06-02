//! Background task that streams logs for one container.
//!
//! Spawned when the user opens the log view and aborted (via the returned
//! [`JoinHandle`]) when they leave it or switch containers, so only one log
//! stream is ever active.

use bollard::container::LogOutput;
use bollard::query_parameters::LogsOptionsBuilder;
use futures_util::StreamExt;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::docker::DockerClient;
use crate::event::AppMessage;

/// Number of historical log lines to fetch before following live output.
/// Kept as a `&str` because `LogsOptionsBuilder::tail` takes `&str`.
const TAIL_LINES: &str = "500";

/// Start streaming logs for `container_id`, returning a handle to cancel it.
pub fn spawn(docker: DockerClient, tx: Sender<AppMessage>, container_id: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let options = LogsOptionsBuilder::new()
            .follow(true)
            .stdout(true)
            .stderr(true)
            .tail(TAIL_LINES)
            .build();

        let mut stream = docker.inner().logs(&container_id, Some(options));
        let mut pending = String::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(output) => {
                    pending.push_str(&decode(&output));
                    if !flush_complete_lines(&mut pending, &tx, &container_id).await {
                        return; // receiver dropped
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(AppMessage::LogLine {
                            container_id: container_id.clone(),
                            line: format!("[stream error] {e}"),
                        })
                        .await;
                    break;
                }
            }
        }

        // Emit any trailing partial line, then signal the stream ended. These
        // sends are best-effort: if the receiver is gone the task is ending anyway.
        if !pending.is_empty() {
            let _ = tx
                .send(AppMessage::LogLine {
                    container_id: container_id.clone(),
                    line: std::mem::take(&mut pending),
                })
                .await;
        }
        let _ = tx.send(AppMessage::LogEnded { container_id }).await;
    })
}

/// Decode a log frame into a UTF-8 string (lossy, to survive binary noise).
///
/// Returns a borrowed `Cow` for the common all-valid-UTF-8 case, avoiding an
/// allocation; only invalid bytes force an owned copy.
fn decode(output: &LogOutput) -> std::borrow::Cow<'_, str> {
    let bytes = match output {
        LogOutput::StdOut { message }
        | LogOutput::StdErr { message }
        | LogOutput::StdIn { message }
        | LogOutput::Console { message } => message,
    };
    String::from_utf8_lossy(bytes)
}

/// Send every complete (newline-terminated) line, keeping the partial remainder.
///
/// Scans `pending` once and drains the processed prefix a single time, instead
/// of re-scanning and draining per line.
///
/// Returns `false` if the receiver has been dropped.
async fn flush_complete_lines(
    pending: &mut String,
    tx: &Sender<AppMessage>,
    container_id: &str,
) -> bool {
    let mut start = 0;
    while let Some(rel) = pending[start..].find('\n') {
        let end = start + rel;
        // Slice excludes the '\n'; trim a trailing '\r' from CRLF endings.
        let line = pending[start..end].trim_end_matches('\r').to_string();
        start = end + 1;
        if tx
            .send(AppMessage::LogLine {
                container_id: container_id.to_string(),
                line,
            })
            .await
            .is_err()
        {
            pending.drain(..start);
            return false;
        }
    }
    pending.drain(..start);
    true
}
