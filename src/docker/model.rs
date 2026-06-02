//! Lean internal data models, decoupled from bollard response types.
//!
//! The TUI renders these structs, never the raw bollard models. This keeps the
//! UI independent of the Docker client crate and makes the data easy to test.

use bollard::models::{ContainerSummary, ContainerSummaryStateEnum};

/// A single container as shown in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    /// Human-readable status, e.g. "Up 3 hours" or "Exited (0) 2 days ago".
    pub status: String,
    /// Pre-formatted port mappings, e.g. "0.0.0.0:8080->80/tcp".
    pub ports: String,
}

/// Simplified lifecycle state used for coloring and filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerState {
    Running,
    Paused,
    Restarting,
    Created,
    Exited,
    Dead,
    Removing,
    Unknown,
}

impl ContainerState {
    /// Whether the container is actively running (used by the running-only filter).
    pub fn is_running(self) -> bool {
        matches!(self, ContainerState::Running | ContainerState::Restarting)
    }

    /// Short label for display.
    pub fn label(self) -> &'static str {
        match self {
            ContainerState::Running => "running",
            ContainerState::Paused => "paused",
            ContainerState::Restarting => "restarting",
            ContainerState::Created => "created",
            ContainerState::Exited => "exited",
            ContainerState::Dead => "dead",
            ContainerState::Removing => "removing",
            ContainerState::Unknown => "unknown",
        }
    }
}

impl From<Option<ContainerSummaryStateEnum>> for ContainerState {
    fn from(value: Option<ContainerSummaryStateEnum>) -> Self {
        match value {
            Some(ContainerSummaryStateEnum::RUNNING) => ContainerState::Running,
            Some(ContainerSummaryStateEnum::PAUSED) => ContainerState::Paused,
            Some(ContainerSummaryStateEnum::RESTARTING) => ContainerState::Restarting,
            Some(ContainerSummaryStateEnum::CREATED) => ContainerState::Created,
            Some(ContainerSummaryStateEnum::EXITED | ContainerSummaryStateEnum::STOPPING) => {
                ContainerState::Exited
            }
            Some(ContainerSummaryStateEnum::DEAD) => ContainerState::Dead,
            Some(ContainerSummaryStateEnum::REMOVING) => ContainerState::Removing,
            _ => ContainerState::Unknown,
        }
    }
}

impl Container {
    /// Build a lean [`Container`] from a bollard [`ContainerSummary`].
    pub fn from_summary(summary: ContainerSummary) -> Self {
        let id = summary.id.unwrap_or_default();
        let name = summary
            .names
            .and_then(|names| names.into_iter().next())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| short_id(&id).to_string());

        Container {
            name,
            image: summary.image.unwrap_or_else(|| "<none>".to_string()),
            state: summary.state.into(),
            status: summary.status.unwrap_or_default(),
            ports: format_ports(summary.ports.as_deref().unwrap_or(&[])),
            id,
        }
    }

    /// First 12 characters of the container id, like the Docker CLI shows.
    pub fn short_id(&self) -> &str {
        short_id(&self.id)
    }
}

/// Truncate a container id to its short 12-char form (ids are ASCII hex).
fn short_id(id: &str) -> &str {
    id.get(..12).unwrap_or(id)
}

/// Lowercase protocol name for a port, defaulting to "tcp".
fn proto_str(typ: Option<bollard::models::PortSummaryTypeEnum>) -> &'static str {
    use bollard::models::PortSummaryTypeEnum;
    match typ {
        Some(PortSummaryTypeEnum::UDP) => "udp",
        Some(PortSummaryTypeEnum::SCTP) => "sctp",
        _ => "tcp",
    }
}

/// Format port mappings into a compact, deduplicated string.
fn format_ports(ports: &[bollard::models::PortSummary]) -> String {
    let mut formatted: Vec<String> = ports
        .iter()
        .map(|p| {
            let proto = proto_str(p.typ);
            match (p.public_port, p.ip.as_deref()) {
                (Some(public), Some(ip)) => {
                    format!("{ip}:{public}->{}/{proto}", p.private_port)
                }
                (Some(public), None) => format!("{public}->{}/{proto}", p.private_port),
                (None, _) => format!("{}/{proto}", p.private_port),
            }
        })
        .collect();
    formatted.sort();
    formatted.dedup();
    formatted.join(", ")
}

/// Live resource usage for a single container.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub mem_used: u64,
    pub mem_limit: u64,
    pub mem_percent: f64,
    pub net_rx: u64,
    pub net_tx: u64,
}
