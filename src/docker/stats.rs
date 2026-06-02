//! Background task that polls live container stats.
//!
//! Docker's CPU percentage is a delta between two consecutive reads. A single
//! one-shot read has a zeroed `precpu`, so instead of holding a long-lived
//! stream per container we keep the previous raw counters ourselves and compute
//! the delta on each tick. This keeps connections short and the footprint small.

use std::collections::HashMap;
use std::time::Duration;

use bollard::models::ContainerStatsResponse;
use bollard::query_parameters::StatsOptionsBuilder;
use futures_util::StreamExt;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;

use crate::docker::DockerClient;
use crate::docker::model::ContainerStats;
use crate::event::AppMessage;

/// How often live stats are refreshed.
const STATS_INTERVAL: Duration = Duration::from_secs(2);

/// Raw CPU counters kept between ticks to compute the next delta.
#[derive(Clone, Copy)]
struct RawCpu {
    total_usage: u64,
    system_usage: u64,
}

/// Run the stats polling loop until the channel closes.
pub async fn run(docker: DockerClient, tx: Sender<AppMessage>) {
    let mut prev: HashMap<String, RawCpu> = HashMap::new();
    let mut ticker = interval(STATS_INTERVAL);

    loop {
        ticker.tick().await;

        let running = match docker.list_containers(false).await {
            Ok(list) => list,
            Err(e) => {
                if tx.send(AppMessage::Error(e.to_string())).await.is_err() {
                    return;
                }
                continue;
            }
        };

        // Fetch one-shot stats for every running container concurrently.
        let fetches = running.iter().map(|c| {
            let docker = docker.clone();
            let id = c.id.clone();
            async move { (id.clone(), fetch_one_shot(&docker, &id).await) }
        });
        let results = futures_util::future::join_all(fetches).await;

        let mut snapshot = HashMap::with_capacity(results.len());
        let mut next_prev = HashMap::with_capacity(results.len());
        for (id, response) in results {
            let Some(response) = response else { continue };
            let (stats, raw) = compute(&response, prev.get(&id).copied());
            if let Some(raw) = raw {
                next_prev.insert(id.clone(), raw);
            }
            snapshot.insert(id, stats);
        }
        prev = next_prev;

        if tx.send(AppMessage::Stats(snapshot)).await.is_err() {
            return; // UI gone — stop the task.
        }
    }
}

/// Pull a single stats reading for one container.
async fn fetch_one_shot(docker: &DockerClient, id: &str) -> Option<ContainerStatsResponse> {
    let options = StatsOptionsBuilder::new()
        .stream(false)
        .one_shot(true)
        .build();
    let mut stream = docker.inner().stats(id, Some(options));
    match stream.next().await {
        Some(Ok(response)) => Some(response),
        _ => None,
    }
}

/// Compute display stats from a reading and the previous raw CPU counters.
///
/// Returns the stats to show plus the raw counters to keep for the next delta.
fn compute(
    response: &ContainerStatsResponse,
    prev: Option<RawCpu>,
) -> (ContainerStats, Option<RawCpu>) {
    let mut stats = ContainerStats::default();
    let mut raw = None;

    if let Some(cpu) = &response.cpu_stats {
        let total = cpu.cpu_usage.as_ref().and_then(|u| u.total_usage);
        let system = cpu.system_cpu_usage;
        if let (Some(total), Some(system)) = (total, system) {
            raw = Some(RawCpu {
                total_usage: total,
                system_usage: system,
            });
            if let Some(prev) = prev {
                stats.cpu_percent = cpu_percent(total, system, prev, online_cpus(response));
            }
        }
    }

    if let Some(mem) = &response.memory_stats {
        let used = mem.usage.unwrap_or(0).saturating_sub(cache_bytes(mem));
        let limit = mem.limit.unwrap_or(0);
        stats.mem_used = used;
        stats.mem_limit = limit;
        stats.mem_percent = if limit > 0 {
            (used as f64 / limit as f64) * 100.0
        } else {
            0.0
        };
    }

    if let Some(networks) = &response.networks {
        for net in networks.values() {
            stats.net_rx += net.rx_bytes.unwrap_or(0);
            stats.net_tx += net.tx_bytes.unwrap_or(0);
        }
    }

    (stats, raw)
}

/// Docker's CPU percentage formula using deltas against the previous read.
fn cpu_percent(total: u64, system: u64, prev: RawCpu, online_cpus: f64) -> f64 {
    let cpu_delta = total.saturating_sub(prev.total_usage) as f64;
    let system_delta = system.saturating_sub(prev.system_usage) as f64;
    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * online_cpus * 100.0
    } else {
        0.0
    }
}

/// Number of online CPUs, defaulting to 1 when the daemon doesn't report it.
fn online_cpus(response: &ContainerStatsResponse) -> f64 {
    response
        .cpu_stats
        .as_ref()
        .and_then(|c| c.online_cpus)
        .filter(|&n| n > 0)
        .unwrap_or(1) as f64
}

/// Subtract page cache from memory usage to match `docker stats` reporting.
fn cache_bytes(mem: &bollard::models::ContainerMemoryStats) -> u64 {
    mem.stats
        .as_ref()
        .and_then(|s| s.get("inactive_file").or_else(|| s.get("cache")).copied())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_percent_uses_delta_against_previous_read() {
        // Arrange: container used 10ms of 100ms total system time across 2 CPUs.
        let prev = RawCpu {
            total_usage: 0,
            system_usage: 0,
        };

        // Act
        let pct = cpu_percent(10, 100, prev, 2.0);

        // Assert: 10/100 * 2 * 100 = 20%
        assert!((pct - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cpu_percent_is_zero_without_progress() {
        let prev = RawCpu {
            total_usage: 50,
            system_usage: 200,
        };
        assert_eq!(cpu_percent(50, 200, prev, 4.0), 0.0);
    }

    #[test]
    fn cpu_percent_handles_counter_reset_without_panicking() {
        // total below previous (counter reset) must saturate, not underflow.
        let prev = RawCpu {
            total_usage: 100,
            system_usage: 100,
        };
        assert_eq!(cpu_percent(0, 50, prev, 1.0), 0.0);
    }

    #[test]
    fn compute_populates_memory_percent() {
        let response = ContainerStatsResponse {
            memory_stats: Some(bollard::models::ContainerMemoryStats {
                usage: Some(500),
                limit: Some(1000),
                ..Default::default()
            }),
            ..Default::default()
        };

        let (stats, _) = compute(&response, None);

        assert_eq!(stats.mem_used, 500);
        assert_eq!(stats.mem_limit, 1000);
        assert!((stats.mem_percent - 50.0).abs() < f64::EPSILON);
    }
}
