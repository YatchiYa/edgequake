//! SPEC-091 IW5: hermetic property tests for KV key grammar + batch clamps.
//!
//! Run (no Postgres required):
//!   cargo test -p edgequake-storage --test proptest_spec091_key_grammar

use edgequake_storage::kv_keys;
use edgequake_storage::migration_engine::AdaptiveBatchSizer;
use edgequake_storage::vector_upsert_chunk_size;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_doc_metadata_roundtrip(doc_id in r"[a-zA-Z0-9_-]{1,64}") {
        let key = kv_keys::doc_metadata(&doc_id);
        prop_assert_eq!(kv_keys::parse_doc_metadata(&key), Some(doc_id.as_str()));
    }

    #[test]
    fn prop_doc_chunk_roundtrip(doc_id in r"[a-zA-Z0-9_-]{1,64}", index in 0usize..10_000usize) {
        let key = kv_keys::doc_chunk(&doc_id, index);
        prop_assert_eq!(kv_keys::parse_doc_chunk(&key), Some((doc_id.as_str(), index)));
        prop_assert!(key.starts_with(&kv_keys::doc_chunk_prefix(&doc_id)));
    }

    #[test]
    fn prop_staging_doc_metadata_parses_to_bare_id(doc_id in r"[a-zA-Z0-9_-]{1,64}") {
        let key = kv_keys::staging_doc_metadata(&doc_id);
        prop_assert_eq!(kv_keys::parse_doc_metadata(&key), Some(doc_id.as_str()));
    }

    #[test]
    fn prop_workspace_doc_index_roundtrip(
        ws in r"[a-f0-9-]{8,36}",
        doc in r"[a-zA-Z0-9_-]{1,64}",
    ) {
        let key = kv_keys::workspace_doc_index(&ws, &doc);
        prop_assert_eq!(kv_keys::parse_workspace_doc_index(&key), Some((ws.as_str(), doc.as_str())));
        prop_assert_eq!(kv_keys::embedded_workspace_id(&key), Some(ws.as_str()));
    }

    #[test]
    fn prop_staging_hash_roundtrip(
        ws in r"[a-f0-9-]{8,36}",
        hash in r"[a-f0-9]{16,64}",
    ) {
        let key = kv_keys::staging_workspace_hash(&ws, &hash);
        prop_assert_eq!(
            kv_keys::parse_staging_workspace_hash(&key),
            Some((ws.as_str(), hash.as_str()))
        );
    }

    #[test]
    fn prop_assert_key_matches_workspace_accepts_own_scope(
        ws in r"[a-f0-9-]{8,36}",
        doc in r"[a-zA-Z0-9_-]{1,32}",
    ) {
        let key = kv_keys::workspace_doc_index(&ws, &doc);
        prop_assert!(kv_keys::assert_key_matches_workspace(&key, &ws).is_ok());
    }

    #[test]
    fn prop_assert_key_matches_workspace_rejects_foreign_scope(
        ws_a in r"[a-f0-9-]{8,36}",
        ws_b in r"[a-f0-9-]{8,36}",
        doc in r"[a-zA-Z0-9_-]{1,32}",
    ) {
        prop_assume!(ws_a != ws_b);
        let key = kv_keys::workspace_doc_index(&ws_a, &doc);
        prop_assert!(kv_keys::assert_key_matches_workspace(&key, &ws_b).is_err());
    }

    #[test]
    fn prop_adaptive_batch_sizer_stays_in_bounds(
        min in 1u32..256,
        max in 256u32..32_000,
        target in 50u64..500,
        slow in 200u64..2_000,
        duration in 1u64..5_000,
        throttled in proptest::bool::ANY,
    ) {
        let mut sizer = AdaptiveBatchSizer::new(min, max, target, slow);
        for _ in 0..32 {
            sizer.record(duration, throttled);
            prop_assert!(sizer.size() >= min);
            prop_assert!(sizer.size() <= max);
        }
    }
}

#[test]
fn contract_vector_upsert_chunk_size_clamped() {
    std::env::set_var("EDGEQUAKE_VECTOR_UPSERT_CHUNK", "1");
    assert_eq!(vector_upsert_chunk_size(), 100);
    std::env::set_var("EDGEQUAKE_VECTOR_UPSERT_CHUNK", "999999");
    assert_eq!(vector_upsert_chunk_size(), 10_000);
    std::env::remove_var("EDGEQUAKE_VECTOR_UPSERT_CHUNK");
}
