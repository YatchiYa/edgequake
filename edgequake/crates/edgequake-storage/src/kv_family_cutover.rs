//! SPEC-091 Wave-2 — per KV family cutover flags.

/// Authority mode for a KV key family during migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvFamilyMode {
    Kv,
    Relational,
}

/// Env prefix: `EDGEQUAKE_KV_FAMILY_<NAME>` where NAME is uppercase with underscores.
pub const KV_FAMILY_ENV_PREFIX: &str = "EDGEQUAKE_KV_FAMILY_";

/// Known KV families from SPEC-091 target spec (Wave 2 cutover).
pub const KV_FAMILY_CHUNK: &str = "CHUNK";
pub const KV_FAMILY_METADATA: &str = "METADATA";
pub const KV_FAMILY_WSDOC: &str = "WSDOC";
/// `staging:hash:` keys route through [`KV_FAMILY_DOC_HASH`] at runtime — no
/// separate env flag (GAP-091-21a, SPEC-091 IW3).
pub const KV_FAMILY_DOC_HASH: &str = "DOC_HASH";
pub const KV_FAMILY_COMPENSATION_QUARANTINE: &str = "COMPENSATION_QUARANTINE";
pub const KV_FAMILY_CHECKPOINT: &str = "CHECKPOINT";
pub const KV_FAMILY_ARTIFACT: &str = "ARTIFACT";
pub const KV_FAMILY_INJECTION: &str = "INJECTION";
/// LLM/keyword/multimodal caches (`{hash}-cache`, `{hash}-kwcache`) →
/// `public.llm_cache` (migration 124).
pub const KV_FAMILY_CACHE: &str = "CACHE";

fn family_env_key(family: &str) -> String {
    format!("{KV_FAMILY_ENV_PREFIX}{}", family.to_ascii_uppercase())
}

/// Read per-family flag (default `relational` since SPEC-091 Wave D — the KV
/// relation is no longer created at runtime; `kv` is a rollback-only setting
/// for deployments that have not run the drop migration).
pub fn kv_family_mode_from_env(family: &str) -> KvFamilyMode {
    let key = family_env_key(family);
    match std::env::var(&key)
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "kv" => KvFamilyMode::Kv,
        _ => KvFamilyMode::Relational,
    }
}

pub fn kv_family_reads_kv(mode: KvFamilyMode) -> bool {
    matches!(mode, KvFamilyMode::Kv)
}

pub fn kv_family_reads_relational(mode: KvFamilyMode) -> bool {
    matches!(mode, KvFamilyMode::Relational)
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
    fn contract_spec091_kv_family_default_relational() {
        let _guard = env_lock();
        std::env::remove_var(family_env_key(KV_FAMILY_WSDOC));
        assert_eq!(
            kv_family_mode_from_env(KV_FAMILY_WSDOC),
            KvFamilyMode::Relational
        );
    }

    #[test]
    fn contract_spec091_kv_family_rollback_kv() {
        let _guard = env_lock();
        std::env::set_var(family_env_key(KV_FAMILY_WSDOC), "kv");
        assert_eq!(kv_family_mode_from_env(KV_FAMILY_WSDOC), KvFamilyMode::Kv);
        std::env::remove_var(family_env_key(KV_FAMILY_WSDOC));
    }
}
