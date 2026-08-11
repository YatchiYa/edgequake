//! SPEC-120 — concurrent KnowledgeGraphMerger merges hit fleet mirror (real path).
//!
//! Proves two docs racing the same entity name through:
//!   graph → PostgresEntitySink → vector upsert → mirror_legacy_batch
//! do not fail with `idx_*_legacy_vector_id` / GraphMerge (`stats.errors == 0`).
//!
//! Dual-FK absorb remains covered by storage `contract_spec120_*` (this test
//! exercises the partner concurrent-ingest path where sink UNIQUE converges FKs).
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use edgequake_api::postgres_entity_sink::PostgresEntitySink;
use edgequake_pipeline::{
    ExtractedEntity, ExtractedRelationship, ExtractionResult, KnowledgeGraphMerger, MergerConfig,
    RelationalEntitySink,
};
use edgequake_storage::traits::FleetEmbeddingIndex;
use edgequake_storage::{
    MemoryGraphStorage, MemoryVectorStorage, PgFleetEmbeddingIndex, VectorStorage,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

const DIM: usize = 1024;

fn require_db() -> Option<String> {
    let base = std::env::var("DATABASE_URL").ok()?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

fn extraction(doc: &str, chunk: &str, seed: f32) -> ExtractionResult {
    let mut entity = ExtractedEntity::new("John Smith", "PERSON", format!("from {doc}"))
        .with_importance(0.9)
        .with_source_chunk_id(chunk)
        .with_source_document_id(doc);
    entity.embedding = Some(vec![seed; DIM]);
    let mut result = ExtractionResult::new(chunk);
    result.add_entity(entity);
    result
}

#[tokio::test]
async fn e2e_spec120_concurrent_merger_same_entity_no_graph_merge() {
    let Some(url) = require_db() else {
        eprintln!("SKIP: DATABASE_URL unset");
        return;
    };
    // Typed authority so mirror fail-closed path is active (production default).
    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "typed_embeddings");

    let pool = PgPool::connect(&url).await.expect("connect");
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(&pool)
    .await
    .expect("workspace");

    let graph = Arc::new(MemoryGraphStorage::new(format!(
        "spec120-merger-{workspace}"
    )));
    let vector = Arc::new(MemoryVectorStorage::new(
        format!("spec120-merger-{workspace}"),
        DIM,
    ));
    vector.initialize().await.expect("vector init");

    let sink: Arc<dyn RelationalEntitySink> =
        Arc::new(PostgresEntitySink::new_fail_closed(Arc::new(pool.clone())));
    let fleet: Arc<dyn FleetEmbeddingIndex> = Arc::new(PgFleetEmbeddingIndex::new(
        pool.clone(),
        format!("e2e-spec120-merger-{workspace}"),
    ));

    let mk_merger = || {
        KnowledgeGraphMerger::new(MergerConfig::default(), graph.clone(), vector.clone())
            .with_tenant_context(Some(tenant.to_string()), Some(workspace.to_string()))
            .with_relational_sink(sink.clone())
            .with_fleet_embedding_index(fleet.clone())
    };

    let merger_a = mk_merger();
    let merger_b = mk_merger();
    let extr_a = vec![extraction("doc-a", "doc-a-chunk-0", 0.11)];
    let extr_b = vec![extraction("doc-b", "doc-b-chunk-0", 0.22)];
    let (a, b) = tokio::join!(merger_a.merge(extr_a), merger_b.merge(extr_b));
    let stats_a = a.expect("merge A must not return Err");
    let stats_b = b.expect("merge B must not return Err");
    assert_eq!(
        stats_a.errors, 0,
        "merge A must not record GraphMerge/storage errors: first={:?}",
        stats_a.first_error
    );
    assert_eq!(
        stats_b.errors, 0,
        "merge B must not record GraphMerge/storage errors: first={:?}",
        stats_b.first_error
    );

    // Spine converged (exact-name UNIQUE + ON CONFLICT).
    let entity_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM entities \
         WHERE workspace_id = $1 AND name = 'JOHN_SMITH'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("entity count");
    assert_eq!(entity_count, 1, "sink must converge to one JOHN_SMITH row");

    // One typed lid owner (LAW-120-2).
    let owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM entity_embeddings \
         WHERE workspace_id = $1 AND legacy_vector_id = 'entity:JOHN_SMITH'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("lid owners");
    assert_eq!(
        owners, 1,
        "exactly one legacy_vector_id owner after concurrent merge"
    );

    // Mirror completeness: at least one merge stamped the fleet row.
    let stamped: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM entity_embeddings WHERE workspace_id = $1",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("stamped");
    assert!(stamped >= 1, "fleet row must exist");
}

/// #374 also surfaces on relationship lids — same absorb module, concurrent merger path.
#[tokio::test]
async fn e2e_spec120_concurrent_merger_same_relationship_no_graph_merge() {
    let Some(url) = require_db() else {
        eprintln!("SKIP: DATABASE_URL unset");
        return;
    };
    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "typed_embeddings");

    let pool = PgPool::connect(&url).await.expect("connect");
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(&pool)
    .await
    .expect("workspace");

    let graph = Arc::new(MemoryGraphStorage::new(format!("spec120-rel-{workspace}")));
    let vector = Arc::new(MemoryVectorStorage::new(
        format!("spec120-rel-{workspace}"),
        DIM,
    ));
    vector.initialize().await.expect("vector init");
    let sink: Arc<dyn RelationalEntitySink> =
        Arc::new(PostgresEntitySink::new_fail_closed(Arc::new(pool.clone())));
    let fleet: Arc<dyn FleetEmbeddingIndex> = Arc::new(PgFleetEmbeddingIndex::new(
        pool.clone(),
        format!("e2e-spec120-rel-{workspace}"),
    ));

    let mk_extraction = |doc: &str, chunk: &str, seed: f32| {
        let mut alice = ExtractedEntity::new("Alice", "PERSON", format!("alice {doc}"))
            .with_source_chunk_id(chunk)
            .with_source_document_id(doc);
        alice.embedding = Some(vec![seed; DIM]);
        let mut bob = ExtractedEntity::new("Bob", "PERSON", format!("bob {doc}"))
            .with_source_chunk_id(chunk)
            .with_source_document_id(doc);
        bob.embedding = Some(vec![seed + 0.01; DIM]);
        let mut rel = ExtractedRelationship::new("Alice", "Bob", "KNOWS")
            .with_description(format!("knows in {doc}"))
            .with_source_chunk_id(chunk)
            .with_source_document_id(doc);
        rel.embedding = Some(vec![seed + 0.02; DIM]);
        let mut result = ExtractionResult::new(chunk);
        result.add_entity(alice);
        result.add_entity(bob);
        result.add_relationship(rel);
        result
    };

    let mk_merger = || {
        KnowledgeGraphMerger::new(MergerConfig::default(), graph.clone(), vector.clone())
            .with_tenant_context(Some(tenant.to_string()), Some(workspace.to_string()))
            .with_relational_sink(sink.clone())
            .with_fleet_embedding_index(fleet.clone())
    };

    let merger_a = mk_merger();
    let merger_b = mk_merger();
    let extr_a = vec![mk_extraction("doc-a", "doc-a-chunk-0", 0.31)];
    let extr_b = vec![mk_extraction("doc-b", "doc-b-chunk-0", 0.41)];
    let (a, b) = tokio::join!(merger_a.merge(extr_a), merger_b.merge(extr_b));
    let stats_a = a.expect("merge A");
    let stats_b = b.expect("merge B");
    assert_eq!(stats_a.errors, 0, "first={:?}", stats_a.first_error);
    assert_eq!(stats_b.errors, 0, "first={:?}", stats_b.first_error);

    let rel_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM relationships WHERE workspace_id = $1")
            .bind(workspace)
            .fetch_one(&pool)
            .await
            .expect("rel count");
    assert_eq!(
        rel_count, 1,
        "sink must converge to one ALICE→BOB:KNOWS row"
    );

    let owners: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM relationship_embeddings \
         WHERE workspace_id = $1 AND legacy_vector_id = 'ALICE->BOB:KNOWS'",
    )
    .bind(workspace)
    .fetch_one(&pool)
    .await
    .expect("lid owners");
    assert_eq!(owners, 1, "exactly one relationship lid owner");
}
