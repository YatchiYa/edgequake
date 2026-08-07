//! SPEC-090 F-090-28 / F-090-31 — role-split PostgreSQL pools.
//! SPEC-112 — application_name, explicit lifetimes, close-all, budget hook.
//!
//! Four in-process pools share one primary `DATABASE_URL`. The **query** pool
//! uses `DATABASE_READ_URL` when set (read replica), otherwise the primary URL.
//!
//! Workload mapping (LAW-P1):
//! - **query** — latency-bound interactive reads
//! - **ingest** — throughput-bound writes (vector/KV/graph/PDF)
//! - **queue** — task claim / lease / list
//! - **admin** — migrate, reconcile, DDL, ANN warmup, inspector

use super::connection::{
    pool_idle_timeout, pool_max_lifetime, session_application_name, with_session_hygiene_labeled,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Role label for metrics and logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolRole {
    Query,
    Ingest,
    Queue,
    Admin,
}

impl PoolRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Ingest => "ingest",
            Self::Queue => "queue",
            Self::Admin => "admin",
        }
    }

    /// SPEC-112 LAW-112-4: `edgequake:<role>` for pg_stat_activity.
    pub fn application_name(self) -> &'static str {
        session_application_name(Some(self.as_str()))
    }

    fn size_env(self) -> &'static str {
        match self {
            Self::Query => "EDGEQUAKE_DB_POOL_SIZE_QUERY",
            Self::Ingest => "EDGEQUAKE_DB_POOL_SIZE_INGEST",
            Self::Queue => "EDGEQUAKE_DB_POOL_SIZE_QUEUE",
            Self::Admin => "EDGEQUAKE_DB_POOL_SIZE_ADMIN",
        }
    }

    fn default_size(self) -> u32 {
        match self {
            Self::Query => 16,
            Self::Ingest => 12,
            Self::Queue => 4,
            Self::Admin => 2,
        }
    }

    fn acquire_timeout(self) -> Duration {
        match self {
            Self::Query => Duration::from_secs(5),
            Self::Ingest => Duration::from_secs(10),
            Self::Queue => Duration::from_secs(5),
            Self::Admin => Duration::from_secs(30),
        }
    }
}

/// Four role-isolated connection pools (SPEC-090 F-090-28).
#[derive(Clone)]
pub struct PgPoolBundle {
    pub query: PgPool,
    pub ingest: PgPool,
    pub queue: PgPool,
    pub admin: PgPool,
    pub query_max: u32,
    pub ingest_max: u32,
    pub queue_max: u32,
    pub admin_max: u32,
    /// True when query pool connected via `DATABASE_READ_URL`.
    pub query_uses_read_url: bool,
}

impl PgPoolBundle {
    /// Build all four pools. `primary_url` is `DATABASE_URL`; query may use read URL.
    pub async fn connect(primary_url: &str) -> Result<Self, sqlx::Error> {
        Self::connect_with_queue_floor(primary_url, None).await
    }

    /// Like [`connect`], but floors the **queue** pool to at least `queue_floor`
    /// (SPEC-112: `claim_next` workers must not exceed queue `max_connections`).
    pub async fn connect_with_queue_floor(
        primary_url: &str,
        queue_floor: Option<u32>,
    ) -> Result<Self, sqlx::Error> {
        let query_url = std::env::var("DATABASE_READ_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| primary_url.to_string());
        let query_uses_read_url = query_url != primary_url;

        let query_max = role_max_connections_with_floor(PoolRole::Query, None);
        let ingest_max = role_max_connections_with_floor(PoolRole::Ingest, None);
        let queue_max = role_max_connections_with_floor(PoolRole::Queue, queue_floor);
        let admin_max = role_max_connections_with_floor(PoolRole::Admin, None);

        if let Some(floor) = queue_floor {
            if queue_max > role_max_connections(PoolRole::Queue) {
                tracing::warn!(
                    queue_max,
                    queue_floor = floor,
                    "SPEC-112: raising queue pool max to cover WORKER_THREADS / claim_next concurrency"
                );
            }
        }

        let query = connect_role(PoolRole::Query, &query_url, query_max).await?;
        let ingest = connect_role(PoolRole::Ingest, primary_url, ingest_max).await?;
        let queue = connect_role(PoolRole::Queue, primary_url, queue_max).await?;
        let admin = connect_role(PoolRole::Admin, primary_url, admin_max).await?;

        tracing::info!(
            query_max,
            ingest_max,
            queue_max,
            admin_max,
            query_uses_read_url,
            idle_timeout_secs = pool_idle_timeout().as_secs(),
            max_lifetime_secs = pool_max_lifetime().as_secs(),
            "SPEC-090/112: PgPoolBundle ready (query/ingest/queue/admin)"
        );

        Ok(Self {
            query,
            ingest,
            queue,
            admin,
            query_max,
            ingest_max,
            queue_max,
            admin_max,
            query_uses_read_url,
        })
    }

    /// Backward-compat primary pool (ingest) for callers not yet role-aware.
    pub fn primary(&self) -> &PgPool {
        &self.ingest
    }

    pub fn for_role(&self, role: PoolRole) -> &PgPool {
        match role {
            PoolRole::Query => &self.query,
            PoolRole::Ingest => &self.ingest,
            PoolRole::Queue => &self.queue,
            PoolRole::Admin => &self.admin,
        }
    }

    /// Configured max for a role (metrics / health).
    pub fn max_for_role(&self, role: PoolRole) -> u32 {
        match role {
            PoolRole::Query => self.query_max,
            PoolRole::Ingest => self.ingest_max,
            PoolRole::Queue => self.queue_max,
            PoolRole::Admin => self.admin_max,
        }
    }

    /// Sum of configured max connections (for PG `max_connections` headroom checks).
    pub fn total_max_connections(&self) -> u32 {
        self.query_max + self.ingest_max + self.queue_max + self.admin_max
    }

    /// SPEC-112 LAW-112-5: close all role pools (call after HTTP drain).
    pub async fn close(&self) {
        // Close in parallel — each pool drains its own idle queue.
        tokio::join!(
            self.query.close(),
            self.ingest.close(),
            self.queue.close(),
            self.admin.close(),
        );
        tracing::info!("SPEC-112: PgPoolBundle closed (query/ingest/queue/admin)");
    }
}

fn role_max_connections(role: PoolRole) -> u32 {
    role_max_connections_with_floor(role, None)
}

fn role_max_connections_with_floor(role: PoolRole, queue_floor: Option<u32>) -> u32 {
    let base = std::env::var(role.size_env())
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| role.default_size())
        .clamp(1, 128);
    match (role, queue_floor) {
        (PoolRole::Queue, Some(floor)) => base.max(floor.clamp(1, 128)),
        _ => base,
    }
}

async fn connect_role(role: PoolRole, url: &str, max: u32) -> Result<PgPool, sqlx::Error> {
    with_session_hygiene_labeled(
        PgPoolOptions::new()
            .max_connections(max)
            .min_connections(1)
            .acquire_timeout(role.acquire_timeout())
            .idle_timeout(Some(pool_idle_timeout()))
            .max_lifetime(Some(pool_max_lifetime())),
        role.application_name(),
    )
    .connect(url)
    .await
}

/// Env size for a role (tests / sizing helpers).
pub fn pool_role_max_connections(role: PoolRole) -> u32 {
    role_max_connections(role)
}

/// SPEC-112: queue pool sized with an explicit worker floor.
pub fn pool_role_max_connections_with_queue_floor(role: PoolRole, queue_floor: u32) -> u32 {
    role_max_connections_with_floor(role, Some(queue_floor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_sum_under_typical_pg_limit() {
        let sum = PoolRole::Query.default_size()
            + PoolRole::Ingest.default_size()
            + PoolRole::Queue.default_size()
            + PoolRole::Admin.default_size();
        assert_eq!(sum, 34);
        assert!(sum < 100, "leave headroom under common max_connections=100");
    }

    #[test]
    fn application_names_are_labeled() {
        assert_eq!(PoolRole::Query.application_name(), "edgequake:query");
        assert_eq!(PoolRole::Ingest.application_name(), "edgequake:ingest");
        assert_eq!(PoolRole::Queue.application_name(), "edgequake:queue");
        assert_eq!(PoolRole::Admin.application_name(), "edgequake:admin");
    }

    #[test]
    fn queue_floor_raises_only_queue_role() {
        let raised = role_max_connections_with_floor(PoolRole::Queue, Some(16));
        assert!(raised >= 16);
        assert_eq!(
            role_max_connections_with_floor(PoolRole::Query, Some(16)),
            role_max_connections(PoolRole::Query)
        );
    }
}
