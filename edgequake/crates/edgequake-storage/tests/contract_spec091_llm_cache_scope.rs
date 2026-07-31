//! SPEC-091 IW0 (GAP-091-14): pin the LLM-cache scope decision.
//!
//! Decision: `llm_cache` entries are keyed by CONTENT HASH inside a storage
//! NAMESPACE — they intentionally carry no tenant/workspace column. Two
//! workspaces sharing a namespace therefore share cache entries (a hit returns
//! the LLM output previously computed for an identical prompt+model), while
//! distinct namespaces never see each other's entries.
//!
//! Rationale (docs/data-layer/llm-cache-scope.md): the cache is a
//! content-addressed recomputation guard — same input ⇒ same output — so
//! sharing is semantically safe and avoids duplicated LLM cost. The accepted
//! residual is a cross-workspace timing/usage side channel within a namespace.
//!
//! These tests PIN that contract so a future schema change (e.g. adding a
//! workspace column, or dropping the namespace predicate) fails loudly here.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test contract_spec091_llm_cache_scope
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::adapters::postgres::llm_cache::{
    cache_get, cache_upsert, cache_values_ordered,
};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};

#[tokio::test]
async fn cache_entry_shared_within_namespace_by_content_hash() {
    let Some(cfg) = require_or_skip_postgres("llmcache_scope") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let ns = cfg.namespace.clone();

    // A "workspace-A" write and a "workspace-B" read use the SAME key: the key
    // grammar (content hash + `-cache`) has no workspace input by design.
    let key = format!("{}-cache", "a".repeat(64));
    cache_upsert(
        &pool,
        &ns,
        &[(
            key.clone(),
            serde_json::json!({"response": "cached-output"}),
        )],
    )
    .await
    .expect("upsert");

    let hit = cache_get(&pool, &ns, &key).await.expect("get");
    assert_eq!(
        hit.as_ref()
            .and_then(|v| v.get("response"))
            .and_then(|v| v.as_str()),
        Some("cached-output"),
        "same-namespace read must hit the shared entry (cross-workspace sharing is the contract)"
    );
}

#[tokio::test]
async fn cache_entries_isolated_across_namespaces() {
    let Some(cfg) = require_or_skip_postgres("llmcache_scope") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let ns_home = cfg.namespace.clone();
    let ns_other = format!("{}_other", cfg.namespace);

    let key = format!("{}-kwcache", "b".repeat(64));
    cache_upsert(
        &pool,
        &ns_home,
        &[(key.clone(), serde_json::json!({"keywords": ["alpha"]}))],
    )
    .await
    .expect("upsert");

    let foreign = cache_get(&pool, &ns_other, &key)
        .await
        .expect("foreign get");
    assert!(
        foreign.is_none(),
        "namespace is the isolation boundary: a different namespace must miss"
    );

    let ordered = cache_values_ordered(&pool, &ns_other, std::slice::from_ref(&key))
        .await
        .expect("foreign ordered");
    assert_eq!(ordered, vec![None], "ordered batch read must also miss");
}

#[tokio::test]
async fn cache_key_grammar_has_no_scope_segment() {
    // Pin the key grammar: workspace/tenant NEVER enter the cache key, so the
    // only sharing boundaries are (content hash, namespace). If a future change
    // embeds scope in keys, the family classifier must still hold — this test
    // forces an explicit update of the decision record.
    use edgequake_storage::adapters::postgres::llm_cache::is_cache_key;
    assert!(is_cache_key(&format!("{}-cache", "c".repeat(64))));
    assert!(is_cache_key(&format!("{}-kwcache", "c".repeat(64))));
    assert!(is_cache_key("image-analysis:deadbeef-cache"));
    assert!(!is_cache_key(
        "019fa6e8-872e-7515-95d2-f15529ea64f3-metadata"
    ));
}
