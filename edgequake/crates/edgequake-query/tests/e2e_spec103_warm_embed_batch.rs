//! Warm Mix embed law (SPEC-103 + LightRAG #2728): keyword cache hit → skip
//! speculative `embed_one`, one unique `embed([q, hl, ll])` RTT.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use edgequake_llm::traits::EmbeddingProvider;
use edgequake_llm::MockProvider;
use edgequake_query::{QueryEngine, QueryEngineConfig, QueryMode, QueryRequest};
use edgequake_storage::traits::{GraphStorage, VectorStorage};
use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct CountingEmbed {
    inner: MockProvider,
    embed_calls: Arc<AtomicUsize>,
    embed_one_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl EmbeddingProvider for CountingEmbed {
    fn name(&self) -> &str {
        EmbeddingProvider::name(&self.inner)
    }
    fn model(&self) -> &str {
        EmbeddingProvider::model(&self.inner)
    }
    fn dimension(&self) -> usize {
        EmbeddingProvider::dimension(&self.inner)
    }
    fn max_tokens(&self) -> usize {
        EmbeddingProvider::max_tokens(&self.inner)
    }
    async fn embed(&self, texts: &[String]) -> edgequake_llm::Result<Vec<Vec<f32>>> {
        self.embed_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed(texts).await
    }
    async fn embed_one(&self, text: &str) -> edgequake_llm::Result<Vec<f32>> {
        self.embed_one_calls.fetch_add(1, Ordering::SeqCst);
        self.inner.embed_one(text).await
    }
}

async fn seeded_engine(embed: Arc<CountingEmbed>, llm: Arc<MockProvider>) -> QueryEngine {
    let dim = EmbeddingProvider::dimension(embed.as_ref());
    let vector = Arc::new(MemoryVectorStorage::new("warm_embed", dim));
    let graph = Arc::new(MemoryGraphStorage::new("warm_embed"));
    vector.initialize().await.unwrap();
    graph.initialize().await.unwrap();
    let seed = vec![0.1_f32; dim];
    vector
        .upsert(&[(
            "chunk_warm".to_string(),
            seed,
            serde_json::json!({"type": "chunk", "content": "BRCA1 germline testing"}),
        )])
        .await
        .unwrap();

    QueryEngine::new(
        QueryEngineConfig {
            use_keyword_extraction: true,
            default_mode: QueryMode::Mix,
            ..Default::default()
        },
        vector,
        graph,
        embed,
        llm,
    )
    .with_answer_cache()
}

#[tokio::test]
async fn e2e_warm_keyword_hit_skips_embed_one_uses_one_batch() {
    {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("EDGEQUAKE_LLM_CACHE", "1");
        std::env::remove_var("EDGEQUAKE_KEYWORD_CACHE");
        std::env::remove_var("EDGEQUAKE_KEYWORD_MODE");
    }

    let embed_calls = Arc::new(AtomicUsize::new(0));
    let embed_one_calls = Arc::new(AtomicUsize::new(0));
    let embed = Arc::new(CountingEmbed {
        inner: MockProvider::default(),
        embed_calls: embed_calls.clone(),
        embed_one_calls: embed_one_calls.clone(),
    });
    let llm = Arc::new(MockProvider::default());
    let engine = seeded_engine(embed, llm).await;
    let request = QueryRequest::new("What is BRCA1 germline testing?").with_mode(QueryMode::Mix);

    engine.query(request.clone()).await.expect("cold fill");
    embed_calls.store(0, Ordering::SeqCst);
    embed_one_calls.store(0, Ordering::SeqCst);

    engine.query(request).await.expect("warm query");
    assert_eq!(
        embed_one_calls.load(Ordering::SeqCst),
        0,
        "warm keyword peek must not speculative embed_one"
    );
    assert_eq!(
        embed_calls.load(Ordering::SeqCst),
        1,
        "warm Mix must issue one unique embed batch"
    );
}

/// Acc Mix injects a fresh workspace embedder (same model, new Arc). Without
/// coerce, the engine LRU is bypassed and every query pays mistral-embed RTT.
#[tokio::test]
async fn e2e_workspace_role_llms_reuses_engine_embed_cache() {
    {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("EDGEQUAKE_LLM_CACHE", "1");
        std::env::remove_var("EDGEQUAKE_KEYWORD_CACHE");
        std::env::remove_var("EDGEQUAKE_KEYWORD_MODE");
    }

    let engine_embed_calls = Arc::new(AtomicUsize::new(0));
    let engine_embed_one = Arc::new(AtomicUsize::new(0));
    let engine_embed = Arc::new(CountingEmbed {
        inner: MockProvider::default(),
        embed_calls: engine_embed_calls.clone(),
        embed_one_calls: engine_embed_one.clone(),
    });
    let llm = Arc::new(MockProvider::default());
    let engine = seeded_engine(engine_embed, llm)
        .await
        .with_embedding_cache();

    let injected_calls = Arc::new(AtomicUsize::new(0));
    let injected: Arc<dyn EmbeddingProvider> = Arc::new(CountingEmbed {
        inner: MockProvider::default(),
        embed_calls: injected_calls.clone(),
        embed_one_calls: Arc::new(AtomicUsize::new(0)),
    });
    let vector = engine.default_vector_storage();
    let request = QueryRequest::new("What is BRCA1 germline testing?").with_mode(QueryMode::Mix);

    engine
        .query_with_role_llms(
            request.clone(),
            Arc::clone(&injected),
            Arc::clone(&vector),
            None,
            None,
        )
        .await
        .expect("cold fill via workspace inject");
    assert_eq!(
        injected_calls.load(Ordering::SeqCst),
        0,
        "same-identity workspace inject must not bypass the engine embedder"
    );
    assert!(
        engine_embed_calls.load(Ordering::SeqCst) > 0,
        "first Mix must miss the engine embed LRU"
    );

    engine_embed_calls.store(0, Ordering::SeqCst);
    engine_embed_one.store(0, Ordering::SeqCst);

    engine
        .query_with_role_llms(request, injected, vector, None, None)
        .await
        .expect("warm Mix via workspace inject");
    assert_eq!(
        engine_embed_one.load(Ordering::SeqCst),
        0,
        "warm Mix must not speculative embed_one"
    );
    assert_eq!(
        engine_embed_calls.load(Ordering::SeqCst),
        0,
        "warm Mix same texts must hit engine embed LRU (inner RTT = 0)"
    );
    assert_eq!(injected_calls.load(Ordering::SeqCst), 0);
}
