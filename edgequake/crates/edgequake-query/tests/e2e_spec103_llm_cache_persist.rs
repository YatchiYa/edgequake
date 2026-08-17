//! SPEC-103 postgres e2e — durable hit across engine rebuild (optional).
//!
//! ```bash
//! DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!   cargo test -p edgequake-query --features postgres --test e2e_spec103_llm_cache_persist -- --ignored --nocapture
//! ```

#![cfg(feature = "postgres")]

use std::sync::Arc;

use edgequake_llm::MockProvider;
use edgequake_query::engine::QueryRequest;
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode};
use edgequake_storage::traits::{GraphStorage, KVStorage, VectorStorage};
use edgequake_storage::{
    MemoryGraphStorage, MemoryVectorStorage, PostgresConfig, PostgresKVStorage,
};

/// Parse `postgres://user:pass@host:port/db` (best-effort for local Acc DB).
fn config_from_database_url(url: &str, namespace: &str) -> Option<PostgresConfig> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))?;
    let (creds, hostpart) = rest.split_once('@')?;
    let (user, password) = creds.split_once(':')?;
    let (hostport, database) = hostpart.split_once('/')?;
    let database = database.split('?').next().unwrap_or(database);
    let (host, port_s) = match hostport.split_once(':') {
        Some((h, p)) => (h, p),
        None => (hostport, "5432"),
    };
    let port: u16 = port_s.parse().ok()?;
    Some(
        PostgresConfig::new(host, port, database, user, password)
            .with_namespace(namespace)
            .with_max_connections(4),
    )
}

#[tokio::test]
#[ignore = "requires DATABASE_URL + running Postgres with llm_cache"]
async fn e2e_spec103_persist_across_engine_rebuild_postgres() {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) if !u.trim().is_empty() => u,
        _ => {
            eprintln!("skip: DATABASE_URL unset");
            return;
        }
    };
    let ns = format!(
        "spec103_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let Some(cfg) = config_from_database_url(url.trim(), &ns) else {
        eprintln!("skip: could not parse DATABASE_URL");
        return;
    };

    let kv = Arc::new(PostgresKVStorage::new(cfg));
    if let Err(e) = kv.initialize().await {
        eprintln!("skip: postgres kv init failed: {e}");
        return;
    }

    let dim = 1536;
    let vector = Arc::new(MemoryVectorStorage::new(&ns, dim));
    let graph = Arc::new(MemoryGraphStorage::new(&ns));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();
    vector
        .upsert(&[(
            "chunk_pg".to_string(),
            vec![1.0_f32; dim],
            serde_json::json!({
                "type": "chunk",
                "content": "postgres durable cache content",
                "document_id": "doc-pg",
            }),
        )])
        .await
        .unwrap();

    let mock = Arc::new(MockProvider::default());
    mock.add_response("PG_CACHED_ANSWER").await;
    mock.add_response("PG_SHOULD_NOT").await;

    std::env::set_var("EDGEQUAKE_LLM_CACHE", "1");
    std::env::remove_var("EDGEQUAKE_QUERY_ANSWER_CACHE");

    let engine1 = QueryEngine::with_mock_keywords(
        QueryEngineConfig {
            mix_local_weight: 0.0,
            mix_global_weight: 0.0,
            mix_naive_weight: 1.0,
            ..Default::default()
        },
        Arc::clone(&vector) as Arc<dyn VectorStorage>,
        Arc::clone(&graph) as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    )
    .with_answer_cache()
    .with_kv_storage(Arc::clone(&kv) as Arc<dyn KVStorage>);

    let mut req = QueryRequest::new("postgres persist");
    req.mode = Some(QueryMode::Naive);
    let r1 = engine1.query(req.clone()).await.expect("engine1");
    assert_eq!(r1.answer, "PG_CACHED_ANSWER");
    assert!(!r1.stats.answer_cache_hit);

    let engine2 = QueryEngine::with_mock_keywords(
        QueryEngineConfig {
            mix_local_weight: 0.0,
            mix_global_weight: 0.0,
            mix_naive_weight: 1.0,
            ..Default::default()
        },
        vector as Arc<dyn VectorStorage>,
        graph as Arc<dyn GraphStorage>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::EmbeddingProvider>,
        Arc::clone(&mock) as Arc<dyn edgequake_llm::traits::LLMProvider>,
    )
    .with_answer_cache()
    .with_kv_storage(kv as Arc<dyn KVStorage>);

    let r2 = engine2.query(req).await.expect("engine2");
    assert!(r2.stats.answer_cache_hit);
    assert_eq!(r2.answer, "PG_CACHED_ANSWER");

    std::env::remove_var("EDGEQUAKE_LLM_CACHE");
}
