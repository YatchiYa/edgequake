//! SPEC-074 — retract completeness (delete_by_document) + Wave-2 denorm guard.
//!
//! Memory tests always run. Postgres path runs when DATABASE_URL / POSTGRES_PASSWORD
//! is available (soft-skip otherwise).

#![cfg_attr(not(feature = "postgres"), allow(dead_code))]

#[path = "support/e2e_fixtures.rs"]
mod e2e_fixtures;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use e2e_fixtures::generate_namespace;
use edgequake_storage::{KVStorage, MemoryKVStorage, MemoryVectorStorage, VectorStorage};
use serde_json::json;

const TEST_DIM: usize = 64;

fn emb(seed: f32) -> Vec<f32> {
    (0..TEST_DIM).map(|i| seed + i as f32 * 0.01).collect()
}

#[tokio::test]
async fn memory_delete_by_document_clears_vectors_keeps_neighbor_doc() {
    let ns = generate_namespace();
    let vector = MemoryVectorStorage::new(&ns, TEST_DIM);
    let kv = MemoryKVStorage::new(&ns);
    vector.initialize().await.unwrap();
    kv.initialize().await.unwrap();

    let doc_a = "doc-074-a";
    let doc_b = "doc-074-b";

    kv.upsert(&[(
        format!("chunk:{doc_a}:0"),
        json!({"document_id": doc_a, "text": "alpha"}),
    )])
    .await
    .unwrap();

    vector
        .upsert(&[
            (
                format!("{doc_a}-chunk-0"),
                emb(1.0),
                json!({
                    "type": "chunk",
                    "document_id": doc_a,
                    "workspace_id": "ws-074",
                    "tenant_id": "t-074"
                }),
            ),
            (
                format!("{doc_a}-chunk-1"),
                emb(2.0),
                json!({
                    "type": "chunk",
                    "source_document_id": doc_a,
                    "workspace_id": "ws-074"
                }),
            ),
            (
                format!("{doc_b}-chunk-0"),
                emb(3.0),
                json!({
                    "type": "chunk",
                    "document_id": doc_b,
                    "workspace_id": "ws-074"
                }),
            ),
        ])
        .await
        .unwrap();

    assert_eq!(vector.count().await.unwrap(), 3);
    let deleted = vector.delete_by_document(doc_a).await.unwrap();
    assert_eq!(deleted, 2);
    assert!(vector
        .get_by_id(&format!("{doc_a}-chunk-0"))
        .await
        .unwrap()
        .is_none());
    assert!(vector
        .get_by_id(&format!("{doc_b}-chunk-0"))
        .await
        .unwrap()
        .is_some());
    assert_eq!(vector.count().await.unwrap(), 1);
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn postgres_delete_by_document_and_denorm_columns() {
    // Soft-skip when no DB (do not use require_or_skip — CI may set REQUIRE=1 without URL).
    let Some(config) = postgres_test_config::contract_postgres_config("spec074") else {
        eprintln!("SKIP spec074 postgres: DATABASE_URL / POSTGRES_PASSWORD not set");
        return;
    };

    use edgequake_storage::PgVectorStorage;

    let storage = PgVectorStorage::with_dimension(config.clone(), TEST_DIM);
    if let Err(e) = storage.initialize().await {
        eprintln!("SKIP spec074 postgres: initialize failed ({e})");
        return;
    }

    let doc_a = "doc-074-pg-a";
    let doc_b = "doc-074-pg-b";
    let ws = "11111111-1111-1111-1111-111111111111";

    storage
        .upsert(&[
            (
                format!("{doc_a}-chunk-0"),
                emb(1.0),
                json!({
                    "type": "chunk",
                    "document_id": doc_a,
                    "workspace_id": ws,
                    "tenant_id": "tenant-074"
                }),
            ),
            (
                format!("{doc_b}-chunk-0"),
                emb(2.0),
                json!({
                    "type": "chunk",
                    "document_id": doc_b,
                    "workspace_id": ws,
                    "tenant_id": "tenant-074"
                }),
            ),
        ])
        .await
        .expect("upsert");

    // Denorm guard: materialized columns must mirror metadata (Wave-2 implication).
    let pool = postgres_test_config::contract_pg_pool(&config).await;
    let prefix = config.table_prefix();
    let table = format!("public.eq_{prefix}_vectors");
    let row: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(&format!(
        "SELECT document_id, workspace_id, tenant_id FROM {table} WHERE id = $1"
    ))
    .bind(format!("{doc_a}-chunk-0"))
    .fetch_one(&pool)
    .await
    .expect("denorm select");
    assert_eq!(row.0.as_deref(), Some(doc_a));
    assert_eq!(row.1.as_deref(), Some(ws));
    assert_eq!(row.2.as_deref(), Some("tenant-074"));

    let deleted = storage.delete_by_document(doc_a).await.expect("delete");
    assert_eq!(deleted, 1);
    assert!(storage
        .get_by_id(&format!("{doc_a}-chunk-0"))
        .await
        .unwrap()
        .is_none());
    assert!(storage
        .get_by_id(&format!("{doc_b}-chunk-0"))
        .await
        .unwrap()
        .is_some());

    let leftover: (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*)::bigint FROM {table} WHERE document_id = $1 OR metadata->>'document_id' = $1"
    ))
    .bind(doc_a)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(leftover.0, 0, "no ghost vectors for deleted document");
}
