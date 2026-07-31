//! Postgres LISTEN/NOTIFY bridge for cross-replica cancel (SPEC-120 P0).
//!
//! Channel name: `task_cancel`. Payload: track_id text.
//! NOTIFY is best-effort; lease heartbeat [`LeaseVerdict::CancelRequested`] is the guarantee.

use std::sync::Arc;

use edgequake_tasks::CancellationRegistry;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use tracing::{debug, info, warn};

/// Postgres NOTIFY channel for durable cancel wakes.
pub const TASK_CANCEL_CHANNEL: &str = "task_cancel";

/// Env var to suppress NOTIFY (integration tests proving heartbeat path).
pub const SUPPRESS_CANCEL_NOTIFY_ENV: &str = "EDGEQUAKE_SUPPRESS_CANCEL_NOTIFY";

/// True when tests/ops deliberately disable NOTIFY (heartbeat-only cancel).
pub fn cancel_notify_suppressed() -> bool {
    matches!(
        std::env::var(SUPPRESS_CANCEL_NOTIFY_ENV).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes")
    )
}

/// Publish cancel wake via `pg_notify`. No-op when suppressed or on error.
pub async fn notify_task_cancel(pool: &PgPool, track_id: &str) {
    if cancel_notify_suppressed() {
        debug!(track_id = %track_id, "cancel NOTIFY suppressed");
        return;
    }
    if let Err(e) = sqlx::query("SELECT pg_notify($1, $2)")
        .bind(TASK_CANCEL_CHANNEL)
        .bind(track_id)
        .execute(pool)
        .await
    {
        warn!(
            track_id = %track_id,
            error = %e,
            "pg_notify(task_cancel) failed — heartbeat will still honour cancel"
        );
    }
}

/// [`CancelWake`] backed by Postgres NOTIFY.
pub struct PgCancelWake {
    pool: PgPool,
}

impl PgCancelWake {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl crate::services::task_cancel::CancelWake for PgCancelWake {
    async fn wake_cancel(&self, track_id: &str) {
        notify_task_cancel(&self.pool, track_id).await;
    }
}

/// Spawn a background LISTEN loop that maps NOTIFY → local registry.cancel.
///
/// Returns a JoinHandle; abort on shutdown. Safe to skip when notify suppressed.
pub fn spawn_cancel_notify_listener(
    pool: PgPool,
    registry: CancellationRegistry,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if cancel_notify_suppressed() {
            info!("cancel NOTIFY listener not started (suppressed)");
            return;
        }
        loop {
            match run_listener(&pool, &registry).await {
                Ok(()) => {
                    info!("cancel NOTIFY listener exited cleanly");
                    break;
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        "cancel NOTIFY listener error — reconnecting in 2s"
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    })
}

async fn run_listener(pool: &PgPool, registry: &CancellationRegistry) -> Result<(), sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(TASK_CANCEL_CHANNEL).await?;
    info!(
        channel = TASK_CANCEL_CHANNEL,
        "listening for task_cancel NOTIFY"
    );

    loop {
        let notification = listener.recv().await?;
        let track_id = notification.payload();
        if track_id.is_empty() {
            continue;
        }
        debug!(track_id = %track_id, "cancel NOTIFY received — signalling local registry");
        let _ = registry.cancel(track_id).await;
    }
}

/// Shared optional wake used by HTTP cancel when a PgPool is available.
pub type SharedCancelWake = Arc<dyn crate::services::task_cancel::CancelWake>;
