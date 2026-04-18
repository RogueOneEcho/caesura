//! Cooperative shutdown for graceful SIGINT handling.

use crate::prelude::*;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal::ctrl_c;

/// Cooperative shutdown signal for graceful SIGINT handling.
///
/// Registered as a singleton in the DI container. Call [`listen`](Shutdown::listen)
/// once at startup to spawn the signal handler task.
pub struct Shutdown {
    /// Whether a graceful shutdown has been requested.
    requested: AtomicBool,
}

#[injectable]
impl Shutdown {
    /// Create a new [`Shutdown`] in the non-requested state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
        }
    }

    /// Check whether a graceful shutdown has been requested.
    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Relaxed)
    }

    /// Mark shutdown as requested.
    fn request(&self) {
        self.requested.store(true, Ordering::Relaxed);
    }

    /// Spawn a task that listens for SIGINT.
    ///
    /// - First signal: sets the shutdown flag and logs a message
    /// - Second signal: calls [`exit`]
    pub fn listen(self: &Ref<Self>) {
        let shutdown = self.clone();
        tokio::spawn(async move {
            ctrl_c().await.ok();
            shutdown.request();
            info!(
                "{} graceful shutdown.\nFinishing current item.\nPress Ctrl+C again to exit immediately.",
                "Triggered".bold()
            );
            ctrl_c().await.ok();
            info!("{} immediate shutdown", "Triggered".bold());
            exit(1);
        });
    }
}
