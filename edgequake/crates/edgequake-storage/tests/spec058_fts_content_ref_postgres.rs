//! SPEC-058 Wave 3 — FTS hits when chunk body lives only in KV (content_ref).

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::adapters::postgres::PostgresPool;
use edgequake_storage::traits::{KVStorage, MetadataFilter, VectorStorage};
use edgequake_storage::{PgVectorStorage, PostgresKVStorage};

const DIM: usize = 4;

#[tokio::test]
async fn spec058_fts_hits_content_ref_only_chunks() {
    let Some(config) = postgres_test_config::contract_postgres_config("spec058_fts_content_ref")
    else {
        eprintln!("SKIP spec058_fts: DATABASE_URL or POSTGRES_PASSWORD not set");
        return;
    };

    let kv = PostgresKVStorage::new(config.clone());
    kv.initialize().await.expect("kv init");

    let pool = PostgresPool::new(config.clone());
    pool.initialize().await.expect("pool init");
    let vectors = PgVectorStorage::with_pool_and_dimension(pool, config.clone(), DIM)
        .with_chunk_kv_table(config.qualified_kv_table());
    vectors.initialize().await.expect("vector init");

    let chunk_id = "spec058-fts-chunk-0";
    let body = "quantum entanglement and photon polarization uniquephrase058";
    kv.upsert(&[(
        chunk_id.to_string(),
        serde_json::json!({"content": body, "type": "chunk"}),
    )])
    .await
    .expect("kv upsert");

    vectors
        .upsert(&[(
            chunk_id.to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
            serde_json::json!({
                "type": "chunk",
                "content_ref": chunk_id,
                "document_id": "spec058-doc"
            }),
        )])
        .await
        .expect("vector upsert");

    let filter = MetadataFilter {
        vector_type: Some("chunk".to_string()),
        ..Default::default()
    };
    let hits = vectors
        .text_search_filtered("uniquephrase058", 10, None, Some(&filter))
        .await
        .expect("fts");

    assert!(
        hits.iter().any(|h| h.id == chunk_id && h.score > 0.0),
        "FTS must rank content_ref chunk via populated content_tsv, got {hits:?}"
    );

    let _ = vectors.delete(&[chunk_id.to_string()]).await;
    let _ = kv.delete(&[chunk_id.to_string()]).await;
}
