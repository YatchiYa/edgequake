//! SPEC-091 Wave-4 — serving fence (fail-closed readiness filter).

pub const SERVING_FENCE_ENV: &str = "EDGEQUAKE_SERVING_FENCE";

/// Serving states from chunk_serving_state (migration 109).
pub const SERVING_STATE_READY: &str = "ready";
pub const SERVING_STATE_DECLARED: &str = "declared";
pub const SERVING_STATE_EMBEDDED: &str = "embedded";
pub const SERVING_STATE_GRAPHED: &str = "graphed";
pub const SERVING_STATE_QUARANTINED: &str = "quarantined";
pub const SERVING_STATE_DELETING: &str = "deleting";

/// Read `EDGEQUAKE_SERVING_FENCE`.
///
/// SPEC-091 IP2 / LAW-IP1: **default on** (unset → enabled). Explicit
/// `off` / `false` / `0` disables for soak rollback.
pub fn serving_fence_enabled_from_env() -> bool {
    match std::env::var(SERVING_FENCE_ENV)
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "off" | "false" | "0" | "no" => false,
        // unset, "on", "true", "1", or any other value → fail-closed visibility
        _ => true,
    }
}

/// Whether a chunk row is visible to query paths under the current fence policy.
pub fn chunk_visible_in_query(serving_state: Option<&str>) -> bool {
    if !serving_fence_enabled_from_env() {
        return true;
    }
    serving_state == Some(SERVING_STATE_READY)
}

/// Filter chunk ids by serving state when fence is on.
pub fn filter_ready_chunk_ids<'a, I>(ids: I, states: &[(String, Option<String>)]) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    if !serving_fence_enabled_from_env() {
        return ids.into_iter().map(str::to_string).collect();
    }
    let state_map: std::collections::HashMap<&str, Option<&str>> = states
        .iter()
        .map(|(id, st)| (id.as_str(), st.as_deref()))
        .collect();
    ids.into_iter()
        .filter(|id| chunk_visible_in_query(state_map.get(id).and_then(|opt| *opt)))
        .map(str::to_string)
        .collect()
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
    fn contract_spec091_serving_fence_default_on() {
        let _guard = env_lock();
        std::env::remove_var(SERVING_FENCE_ENV);
        assert!(serving_fence_enabled_from_env());
        assert!(!chunk_visible_in_query(None));
        assert!(!chunk_visible_in_query(Some(SERVING_STATE_DECLARED)));
        assert!(chunk_visible_in_query(Some(SERVING_STATE_READY)));
    }

    #[test]
    fn contract_spec091_serving_fence_explicit_off_allows_all() {
        let _guard = env_lock();
        std::env::set_var(SERVING_FENCE_ENV, "off");
        assert!(!serving_fence_enabled_from_env());
        assert!(chunk_visible_in_query(None));
        assert!(chunk_visible_in_query(Some(SERVING_STATE_DECLARED)));
        std::env::remove_var(SERVING_FENCE_ENV);
    }

    #[test]
    fn contract_spec091_serving_fence_on_ready_only() {
        let _guard = env_lock();
        std::env::set_var(SERVING_FENCE_ENV, "on");
        assert!(!chunk_visible_in_query(Some(SERVING_STATE_EMBEDDED)));
        assert!(chunk_visible_in_query(Some(SERVING_STATE_READY)));
        std::env::remove_var(SERVING_FENCE_ENV);
    }

    #[test]
    fn contract_spec091_filter_ready_chunk_ids() {
        let _guard = env_lock();
        std::env::set_var(SERVING_FENCE_ENV, "on");
        let ids = ["a", "b"];
        let states = vec![
            ("a".to_string(), Some(SERVING_STATE_READY.to_string())),
            ("b".to_string(), Some(SERVING_STATE_DECLARED.to_string())),
        ];
        let ready = filter_ready_chunk_ids(ids.iter().copied(), &states);
        assert_eq!(ready, vec!["a"]);
        std::env::remove_var(SERVING_FENCE_ENV);
    }
}
