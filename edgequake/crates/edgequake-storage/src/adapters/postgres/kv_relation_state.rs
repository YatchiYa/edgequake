//! SPEC-091 Doc 23 — process-local cache for whether `eq_*_kv` still exists.
//!
//! Post-migration-125, every SQL against a missing relation burns a pool
//! checkout and a failing `SELECT` (mapped to Ok/empty). This cache makes
//! subsequent hot-path calls O(0) SQL to the dropped table (LAW-KVH1/KVH2).

use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::PgPool;
use tokio::sync::OnceCell;

use super::schema;
use crate::error::Result;

/// Whether the namespace KV base table is known to exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvRelationPresence {
    Present,
    Absent,
}

/// Per-adapter posture + raw-SQL attempt counter (tests / soak probes).
#[derive(Debug)]
pub struct KvRelationState {
    cell: OnceCell<KvRelationPresence>,
    /// Count of statements issued against the KV base/stats table.
    sql_attempts: AtomicU64,
}

impl Default for KvRelationState {
    fn default() -> Self {
        Self::new()
    }
}

impl KvRelationState {
    pub fn new() -> Self {
        Self {
            cell: OnceCell::new(),
            sql_attempts: AtomicU64::new(0),
        }
    }

    /// Cached posture if already resolved (no I/O).
    pub fn cached(&self) -> Option<KvRelationPresence> {
        self.cell.get().copied()
    }

    /// Seed from boot census (`kv_store_dropped`) without probing.
    pub fn seed(&self, presence: KvRelationPresence) {
        let _ = self.cell.set(presence);
    }

    /// Convenience: seed Absent when cutover posture says the store is dropped.
    pub fn seed_from_dropped(&self, kv_store_dropped: bool) {
        if kv_store_dropped {
            self.seed(KvRelationPresence::Absent);
        }
    }

    /// Mark Absent after the first `42P01` (no-op if already set).
    pub fn note_undefined_table(&self) {
        let _ = self.cell.set(KvRelationPresence::Absent);
    }

    /// Resolve posture via `information_schema` (never hits `eq_*_kv` itself).
    pub async fn get_or_probe(
        &self,
        pool: &PgPool,
        qualified_table: &str,
    ) -> Result<KvRelationPresence> {
        if let Some(p) = self.cached() {
            return Ok(p);
        }
        let exists = schema::relation_exists(pool, qualified_table).await?;
        let presence = if exists {
            KvRelationPresence::Present
        } else {
            KvRelationPresence::Absent
        };
        let _ = self.cell.set(presence);
        Ok(self.cached().unwrap_or(presence))
    }

    pub fn record_sql_attempt(&self) {
        self.sql_attempts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn sql_attempts(&self) -> u64 {
        self.sql_attempts.load(Ordering::Relaxed)
    }

    pub fn reset_sql_attempts(&self) {
        self.sql_attempts.store(0, Ordering::Relaxed);
    }
}
