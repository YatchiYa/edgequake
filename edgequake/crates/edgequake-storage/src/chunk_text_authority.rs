//! SPEC-091 Wave-1 — chunk text authority flag.

/// Where chunk text is authoritative during cutover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkTextAuthority {
    /// Legacy KV keys only (rollback escape hatch).
    Kv,
    /// Dual-write KV + relational chunks (default since SPEC-091 W1
    /// completion: new writes land in both stores, KV stays read-authoritative
    /// with shadow compare until the operator flips to `relational` after the
    /// backfill verification gate).
    Dual,
    /// Relational `chunks.content` only.
    Relational,
}

pub const CHUNK_TEXT_AUTHORITY_ENV: &str = "EDGEQUAKE_CHUNK_TEXT_AUTHORITY";

/// Read `EDGEQUAKE_CHUNK_TEXT_AUTHORITY` (default `relational` since SPEC-091
/// Wave D — the KV relation is no longer created at runtime; `dual`/`kv` are
/// rollback-only settings for deployments that have not run the drop).
pub fn chunk_text_authority_from_env() -> ChunkTextAuthority {
    match std::env::var(CHUNK_TEXT_AUTHORITY_ENV)
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "kv" => ChunkTextAuthority::Kv,
        "dual" => ChunkTextAuthority::Dual,
        _ => ChunkTextAuthority::Relational,
    }
}

pub fn chunk_text_authority_writes_kv(mode: ChunkTextAuthority) -> bool {
    matches!(mode, ChunkTextAuthority::Kv | ChunkTextAuthority::Dual)
}

pub fn chunk_text_authority_writes_relational(mode: ChunkTextAuthority) -> bool {
    matches!(
        mode,
        ChunkTextAuthority::Dual | ChunkTextAuthority::Relational
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn contract_spec091_chunk_text_authority_default_relational() {
        let _guard = env_lock();
        std::env::remove_var(CHUNK_TEXT_AUTHORITY_ENV);
        assert_eq!(
            chunk_text_authority_from_env(),
            ChunkTextAuthority::Relational
        );
    }

    #[test]
    fn contract_spec091_chunk_text_authority_dual() {
        let _guard = env_lock();
        std::env::set_var(CHUNK_TEXT_AUTHORITY_ENV, "dual");
        assert!(chunk_text_authority_writes_kv(ChunkTextAuthority::Dual));
        assert!(chunk_text_authority_writes_relational(
            ChunkTextAuthority::Dual
        ));
        std::env::remove_var(CHUNK_TEXT_AUTHORITY_ENV);
    }
}
