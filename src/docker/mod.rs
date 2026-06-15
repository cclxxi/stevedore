//! Thin wrapper around the bollard Docker client.
//!
//! Everything the rest of the app needs to talk to Docker goes through
//! [`DockerClient`]. It owns a cloneable bollard [`Docker`] handle (cheap to
//! clone — it's `Arc`-backed internally) so background tasks can each hold one.

pub mod actions;
pub mod logs;
pub mod model;
pub mod stats;

use bollard::Docker;
use bollard::query_parameters::ListContainersOptionsBuilder;

use crate::error::AppError;
use model::Container;

/// Cloneable handle to the Docker daemon.
#[derive(Clone)]
pub struct DockerClient {
    inner: Docker,
}

impl DockerClient {
    /// Connect to the local Docker daemon using platform defaults.
    ///
    /// This only constructs the client; call [`version`](Self::version) to
    /// actually reach the daemon and fail fast with a friendly hint.
    pub fn connect() -> Result<Self, AppError> {
        let inner = Docker::connect_with_local_defaults()
            .map_err(|e| AppError::DockerUnreachable(e.to_string()))?;
        Ok(DockerClient { inner })
    }

    /// Return the daemon version string, e.g. "29.4.3 (API 1.51)".
    pub async fn version(&self) -> Result<String, AppError> {
        let v = self
            .inner
            .version()
            .await
            .map_err(|e| AppError::DockerUnreachable(e.to_string()))?;
        let version = v.version.unwrap_or_else(|| "unknown".to_string());
        let api = v.api_version.unwrap_or_default();
        Ok(if api.is_empty() {
            version
        } else {
            format!("{version} (API {api})")
        })
    }

    /// List containers. When `all` is false, only running containers are returned.
    pub async fn list_containers(&self, all: bool) -> Result<Vec<Container>, AppError> {
        let options = ListContainersOptionsBuilder::new().all(all).build();
        let summaries = self
            .inner
            .list_containers(Some(options))
            .await
            .map_err(|e| AppError::Docker(e.to_string()))?;
        Ok(summaries.into_iter().map(Container::from_summary).collect())
    }

    /// Access the underlying bollard client for streaming calls.
    pub(crate) fn inner(&self) -> &Docker {
        &self.inner
    }
}
