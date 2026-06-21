use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use tokio::sync::watch;
use tracing::{debug, info};

use crate::profile::AdaptiveCoverProfile;

/// Generates adaptive background cover traffic at a configured intensity.
pub struct AdaptiveCoverTrafficEngine {
    profile: RwLock<AdaptiveCoverProfile>,
    running: AtomicBool,
    shutdown: RwLock<Option<watch::Sender<bool>>>,
    task: RwLock<Option<tokio::task::JoinHandle<()>>>,
    ticks: AtomicBool,
}

impl AdaptiveCoverTrafficEngine {
    pub fn new(profile: AdaptiveCoverProfile) -> Arc<Self> {
        Arc::new(Self {
            profile: RwLock::new(profile),
            running: AtomicBool::new(false),
            shutdown: RwLock::new(None),
            task: RwLock::new(None),
            ticks: AtomicBool::new(false),
        })
    }

    pub fn profile(&self) -> AdaptiveCoverProfile {
        *self.profile.read()
    }

    pub fn set_profile(&self, profile: AdaptiveCoverProfile) {
        *self.profile.write() = profile;
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn start(self: &Arc<Self>) -> anyhow::Result<()> {
        if self.is_running() {
            return Ok(());
        }

        let profile = self.profile();
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        *self.shutdown.write() = Some(shutdown_tx);

        let engine = Arc::clone(self);
        let task = tokio::spawn(async move {
            info!(?profile, "adaptive cover traffic started");
            let interval = cover_interval(profile);

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() { break; }
                    }
                    _ = tokio::time::sleep(interval) => {
                        let current = engine.profile();
                        debug!(pps = current.packets_per_second(), bps = current.bandwidth_bps(), "cover tick");
                        engine.ticks.store(true, Ordering::SeqCst);
                    }
                }
            }
            engine.running.store(false, Ordering::SeqCst);
        });

        *self.task.write() = Some(task);
        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub async fn stop(&self) -> anyhow::Result<()> {
        if let Some(tx) = self.shutdown.write().take() {
            let _ = tx.send(true);
        }
        if let Some(task) = self.task.write().take() {
            task.abort();
        }
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }
}

fn cover_interval(profile: AdaptiveCoverProfile) -> Duration {
    let pps = profile.packets_per_second().max(1);
    Duration::from_millis(1000 / u64::from(pps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn start_stop_engine() {
        let engine = AdaptiveCoverTrafficEngine::new(AdaptiveCoverProfile::Balanced);
        engine.start().await.expect("start");
        assert!(engine.is_running());
        engine.stop().await.expect("stop");
        assert!(!engine.is_running());
    }

    #[test]
    fn profile_bandwidth_increases_with_aggression() {
        assert!(
            AdaptiveCoverProfile::Maximum.bandwidth_bps()
                > AdaptiveCoverProfile::Conservative.bandwidth_bps()
        );
    }
}
