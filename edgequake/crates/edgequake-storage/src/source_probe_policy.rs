//! AGE source-id probe limit policy (SPEC-089 / INV-R1).
//!
//! SSOT for list reconcile vs cascade discovery — API must not fork these
//! semantics (held-claim / pool-starvation class: `0 → 256` list probes).

/// Max chunk indices probed for GIN `@>` on the list hot path.
pub const SOURCE_CHUNK_PROBE_LIMIT: usize = 256;

/// Default probes for cascade discovery when chunk_count is unknown.
pub const SOURCE_DISCOVERY_DEFAULT_PROBE_LIMIT: usize = 64;

/// Policy for resolving AGE source-id probe series bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceProbePolicy {
    /// Documents list/detail reconcile: `0` chunks → skip AGE (`0`).
    ListReconcile,
    /// Cascade delete/reprocess discovery: `0` / unknown → default 64 probes.
    CascadeDiscovery,
}

/// Resolve probe series upper bound for a known max chunk count.
///
/// - [`SourceProbePolicy::ListReconcile`]: `0 → 0` (caller skips AGE).
/// - [`SourceProbePolicy::CascadeDiscovery`]: `0 → SOURCE_DISCOVERY_DEFAULT_PROBE_LIMIT`,
///   else `clamp(1..=SOURCE_CHUNK_PROBE_LIMIT)`.
pub fn probe_limit_for(policy: SourceProbePolicy, max_chunk_count: usize) -> usize {
    match policy {
        SourceProbePolicy::ListReconcile => {
            if max_chunk_count == 0 {
                0
            } else {
                max_chunk_count.clamp(1, SOURCE_CHUNK_PROBE_LIMIT)
            }
        }
        SourceProbePolicy::CascadeDiscovery => {
            if max_chunk_count == 0 {
                SOURCE_DISCOVERY_DEFAULT_PROBE_LIMIT
            } else {
                max_chunk_count.clamp(1, SOURCE_CHUNK_PROBE_LIMIT)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_reconcile_skips_on_zero() {
        assert_eq!(probe_limit_for(SourceProbePolicy::ListReconcile, 0), 0);
        assert_eq!(probe_limit_for(SourceProbePolicy::ListReconcile, 12), 12);
        assert_eq!(
            probe_limit_for(SourceProbePolicy::ListReconcile, 400),
            SOURCE_CHUNK_PROBE_LIMIT
        );
    }

    #[test]
    fn cascade_discovery_defaults_on_zero() {
        assert_eq!(
            probe_limit_for(SourceProbePolicy::CascadeDiscovery, 0),
            SOURCE_DISCOVERY_DEFAULT_PROBE_LIMIT
        );
        assert_eq!(probe_limit_for(SourceProbePolicy::CascadeDiscovery, 20), 20);
    }
}
