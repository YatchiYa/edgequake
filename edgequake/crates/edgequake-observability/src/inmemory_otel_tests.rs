//! SPEC-124 LAW-124-15/16: CI-unfakable span attribute proofs (no Langfuse keys required).

use crate::langfuse_attrs::{
    is_forbidden_cost_attr, COST_ATTR_DENYLIST, GEN_AI_CONVERSATION_ID, GEN_AI_USAGE_INPUT_TOKENS,
    GEN_AI_USAGE_OUTPUT_TOKENS, LANGFUSE_OBSERVATION_INPUT, LANGFUSE_OBSERVATION_METADATA_PREFIX,
    LANGFUSE_OBSERVATION_OUTPUT, LANGFUSE_OBSERVATION_TYPE, LANGFUSE_SESSION_ID,
    OBSERVATION_TYPE_GENERATION, OBSERVATION_TYPE_RETRIEVER,
};
use crate::langfuse_context::bind_langfuse_identity;
use crate::rag_span::{
    record_embedding_io, record_gen_ai_usage, record_ingest_document_input,
    record_ingest_document_output, record_observation_io, record_rag_retrieval_io,
    with_ingest_document_span, with_llm_generation, with_rag_embedding_span,
    with_rag_generation_span, with_rag_retrieval_span, LlmGenerationRecord, RagRetrievalAttrs,
};
use opentelemetry::trace::{TraceContextExt, Tracer, TracerProvider as _};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::{InMemorySpanExporterBuilder, SdkTracerProvider};
use tracing_subscriber::layer::SubscriberExt;

fn attr_map(
    span: &opentelemetry_sdk::trace::SpanData,
) -> std::collections::HashMap<String, String> {
    span.attributes
        .iter()
        .map(|kv| (kv.key.as_str().to_string(), kv.value.to_string()))
        .collect()
}

fn assert_no_cost_keys(attrs: &std::collections::HashMap<String, String>) {
    for key in COST_ATTR_DENYLIST {
        assert!(
            !attrs.contains_key(*key),
            "forbidden cost attr present: {key}"
        );
    }
    for k in attrs.keys() {
        assert!(!is_forbidden_cost_attr(k), "forbidden cost key leaked: {k}");
    }
}

#[test]
fn inmemory_record_gen_ai_usage_ints_no_cost() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124");

    tracer.in_span("generate-answer", |cx| {
        cx.span().set_attribute(KeyValue::new(
            LANGFUSE_OBSERVATION_TYPE,
            OBSERVATION_TYPE_GENERATION,
        ));
        record_gen_ai_usage(Some(10), Some(20));
        record_observation_io(Some("hello"), Some("world"));
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert_eq!(spans.len(), 1);
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs.get(GEN_AI_USAGE_INPUT_TOKENS).map(String::as_str),
        Some("10")
    );
    assert_eq!(
        attrs.get(GEN_AI_USAGE_OUTPUT_TOKENS).map(String::as_str),
        Some("20")
    );
    assert_eq!(
        attrs.get(LANGFUSE_OBSERVATION_INPUT).map(String::as_str),
        Some("hello")
    );
    assert_eq!(
        attrs.get(LANGFUSE_OBSERVATION_OUTPUT).map(String::as_str),
        Some("world")
    );
    assert_eq!(
        attrs.get(LANGFUSE_OBSERVATION_TYPE).map(String::as_str),
        Some(OBSERVATION_TYPE_GENERATION)
    );
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_tracing_generation_retrieval_embedding_ingest_io() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-tracing");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry().with(otel_layer);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    tracing::subscriber::with_default(subscriber, || {
        rt.block_on(async {
            let _ = with_rag_generation_span("generate-answer", "mock", "mock", async {
                record_gen_ai_usage(Some(3), Some(7));
                record_observation_io(Some("q-in"), Some("a-out"));
                1
            })
            .await;
            let _ = with_rag_retrieval_span(
                RagRetrievalAttrs {
                    data_source_id: Some("edgequake"),
                    top_k: Some(5),
                    arm: Some("naive"),
                    mode: Some("mix"),
                    query_preview: Some("what is nsclc".into()),
                },
                async {
                    record_rag_retrieval_io(false, 2, 1, Some("chunk preview"));
                    2
                },
            )
            .await;
            let _ = with_rag_embedding_span("embed-chunks", "m", "p", async {
                record_embedding_io("chunks", 4, 4, Some(8));
                3
            })
            .await;
            let _ = with_ingest_document_span(async {
                record_ingest_document_input("doc-1", "document body text");
                record_ingest_document_output(2, 5, 1, 2, 0);
                4
            })
            .await;
        });
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert!(
        spans.len() >= 4,
        "expected generation+retrieval+embedding+ingest, got {}",
        spans.len()
    );

    let mut saw_generation_io = false;
    let mut saw_retriever_io = false;
    let mut saw_embedding_io = false;
    let mut saw_ingest_io = false;

    for span in &spans {
        let attrs = attr_map(span);
        assert_no_cost_keys(&attrs);
        let has_in = attrs
            .get(LANGFUSE_OBSERVATION_INPUT)
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        let has_out = attrs
            .get(LANGFUSE_OBSERVATION_OUTPUT)
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        if attrs.get(LANGFUSE_OBSERVATION_TYPE).map(String::as_str)
            == Some(OBSERVATION_TYPE_GENERATION)
            && has_in
            && has_out
        {
            saw_generation_io = true;
        }
        if attrs.get(LANGFUSE_OBSERVATION_TYPE).map(String::as_str)
            == Some(OBSERVATION_TYPE_RETRIEVER)
            && has_in
            && has_out
        {
            saw_retriever_io = true;
            assert!(
                attrs
                    .get(LANGFUSE_OBSERVATION_INPUT)
                    .is_some_and(|s| s.contains("nsclc")),
                "retriever input should carry query preview"
            );
        }
        if attrs.get(LANGFUSE_OBSERVATION_TYPE).map(String::as_str) == Some("embedding")
            && has_in
            && has_out
        {
            saw_embedding_io = true;
        }
        if attrs.get(LANGFUSE_OBSERVATION_TYPE).map(String::as_str) == Some("chain")
            && has_in
            && has_out
        {
            saw_ingest_io = true;
        }
    }

    assert!(saw_generation_io, "missing generation I/O: {spans:?}");
    assert!(saw_retriever_io, "missing retriever I/O: {spans:?}");
    assert!(saw_embedding_io, "missing embedding I/O: {spans:?}");
    assert!(saw_ingest_io, "missing ingest chain I/O: {spans:?}");

    let batch = crate::langfuse_ingestion::spans_to_batch(&spans);
    use crate::langfuse_ingestion::{
        LANGFUSE_V31_EMITTED_ENVELOPE_TYPES, LANGFUSE_V31_GENERATION_CREATE,
        LANGFUSE_V31_SPAN_CREATE,
    };
    for ev in &batch {
        let ty = ev["type"].as_str().unwrap_or("");
        assert!(
            LANGFUSE_V31_EMITTED_ENVELOPE_TYPES.contains(&ty),
            "wired spans produced illegal 3.1.1 envelope {ty}: {ev}"
        );
        assert!(
            !ty.contains("retriever") && !ty.contains("embedding") && !ty.contains("chain"),
            "LAW-124-13 type leaked into envelope: {ty}"
        );
    }
    assert!(batch
        .iter()
        .any(|e| e["type"] == LANGFUSE_V31_GENERATION_CREATE));
    assert!(batch.iter().any(|e| e["type"] == LANGFUSE_V31_SPAN_CREATE));
}

#[test]
fn inmemory_bind_session_attrs_on_active_span() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-session");

    tracer.in_span("query.root", |_cx| {
        let _g = bind_langfuse_identity(Some("conv-abc"), Some("user-1"), None, None);
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert_eq!(spans.len(), 1);
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs.get(LANGFUSE_SESSION_ID).map(String::as_str),
        Some("conv-abc")
    );
    assert_eq!(
        attrs.get(GEN_AI_CONVERSATION_ID).map(String::as_str),
        Some("conv-abc")
    );
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_blank_session_emits_neither_session_attr() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-blank-session");

    tracer.in_span("query.root", |_cx| {
        let _g = bind_langfuse_identity(Some("  "), Some("user-1"), None, None);
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert_eq!(spans.len(), 1);
    let attrs = attr_map(&spans[0]);
    assert!(
        !attrs.contains_key(LANGFUSE_SESSION_ID),
        "blank session must not emit langfuse.session.id"
    );
    assert!(
        !attrs.contains_key(GEN_AI_CONVERSATION_ID),
        "blank session must not emit gen_ai.conversation.id"
    );
}

#[test]
fn inmemory_with_llm_generation_usage_and_io() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-llm-gen");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry().with(otel_layer);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    tracing::subscriber::with_default(subscriber, || {
        rt.block_on(async {
            let ok: Result<&str, &str> =
                with_llm_generation("extract-entities-glean", "m", "p", async {
                    Ok((
                        "done",
                        LlmGenerationRecord::from_response(Some("chunk"), "ents", 11, 22),
                    ))
                })
                .await;
            assert_eq!(ok, Ok("done"));
        });
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert!(!spans.is_empty());
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs.get(GEN_AI_USAGE_INPUT_TOKENS).map(String::as_str),
        Some("11")
    );
    assert_eq!(
        attrs.get(GEN_AI_USAGE_OUTPUT_TOKENS).map(String::as_str),
        Some("22")
    );
    assert_eq!(
        attrs.get(LANGFUSE_OBSERVATION_INPUT).map(String::as_str),
        Some("chunk")
    );
    assert_eq!(
        attrs.get(LANGFUSE_OBSERVATION_OUTPUT).map(String::as_str),
        Some("ents")
    );
    assert_eq!(
        attrs.get(LANGFUSE_OBSERVATION_TYPE).map(String::as_str),
        Some(OBSERVATION_TYPE_GENERATION)
    );
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_llm_generation_records_provider_cache_tokens() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec126-cache");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry().with(otel_layer);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    tracing::subscriber::with_default(subscriber, || {
        rt.block_on(async {
            let ok: Result<&str, &str> = with_llm_generation("extract-entities", "m", "p", async {
                Ok((
                    "done",
                    LlmGenerationRecord::from_response(Some("chunk"), "ents", 100, 20)
                        .with_provider_cache(Some(80), Some(20)),
                ))
            })
            .await;
            assert_eq!(ok, Ok("done"));
        });
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert!(!spans.is_empty());
    let attrs = attr_map(&spans[0]);
    let hit_key = format!("{LANGFUSE_OBSERVATION_METADATA_PREFIX}cache_hit_tokens");
    let write_key = format!("{LANGFUSE_OBSERVATION_METADATA_PREFIX}cache_write_tokens");
    assert_eq!(attrs.get(&hit_key).map(String::as_str), Some("80"));
    assert_eq!(attrs.get(&write_key).map(String::as_str), Some("20"));
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_slugs_additive_to_guids() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-slugs");

    tracer.in_span("query.root", |_cx| {
        let identity = crate::langfuse_attrs::LangfuseTraceIdentity::from_parts(
            Some("conv-1"),
            Some("user-1"),
            Some("11111111-1111-1111-1111-111111111111"),
            Some("22222222-2222-2222-2222-222222222222"),
        )
        .with_slugs(Some("acme"), Some("docs"));
        let _g = crate::langfuse_context::bind_langfuse_trace_identity(identity);
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    assert_eq!(spans.len(), 1);
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs
            .get(crate::langfuse_attrs::LANGFUSE_META_TENANT_ID)
            .map(String::as_str),
        Some("11111111-1111-1111-1111-111111111111")
    );
    assert_eq!(
        attrs
            .get(crate::langfuse_attrs::LANGFUSE_META_TENANT_SLUG)
            .map(String::as_str),
        Some("acme")
    );
    assert_eq!(
        attrs
            .get(crate::langfuse_attrs::LANGFUSE_META_WORKSPACE_ID)
            .map(String::as_str),
        Some("22222222-2222-2222-2222-222222222222")
    );
    assert_eq!(
        attrs
            .get(crate::langfuse_attrs::LANGFUSE_META_WORKSPACE_SLUG)
            .map(String::as_str),
        Some("docs")
    );
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_blank_slug_omitted_uuid_kept() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-blank-slug");

    tracer.in_span("query.root", |_cx| {
        let identity = crate::langfuse_attrs::LangfuseTraceIdentity::from_parts(
            None,
            None,
            Some("11111111-1111-1111-1111-111111111111"),
            Some("22222222-2222-2222-2222-222222222222"),
        )
        .with_slugs(Some("  "), Some(""));
        let _g = crate::langfuse_context::bind_langfuse_trace_identity(identity);
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs
            .get(crate::langfuse_attrs::LANGFUSE_META_TENANT_ID)
            .map(String::as_str),
        Some("11111111-1111-1111-1111-111111111111")
    );
    assert!(
        !attrs.contains_key(crate::langfuse_attrs::LANGFUSE_META_TENANT_SLUG),
        "blank tenant_slug must be omitted"
    );
    assert!(
        !attrs.contains_key(crate::langfuse_attrs::LANGFUSE_META_WORKSPACE_SLUG),
        "blank workspace_slug must be omitted"
    );
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_query_pipeline_meta_mode_no_cost() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-pipeline-meta");

    tracer.in_span("query.root", |_cx| {
        crate::record_query_pipeline_meta(crate::QueryPipelineMeta {
            mode: Some("mix".into()),
            query_intent: Some("factual".into()),
            fusion: Some("rrf".into()),
            keyword_cache_hit: Some(false),
            answer_cache_hit: Some(true),
            citation_count: Some(3),
            context_empty: Some(false),
            ..Default::default()
        });
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs
            .get("langfuse.trace.metadata.mode")
            .map(String::as_str),
        Some("mix")
    );
    assert_eq!(
        attrs
            .get("langfuse.trace.metadata.answer_cache_hit")
            .map(String::as_str),
        Some("true")
    );
    assert!(!attrs.keys().any(|k| k.contains("cost")));
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_ingest_parse_meta_on_span() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec124-parse-meta");

    tracer.in_span("ingest.converting", |_cx| {
        crate::record_ingest_parse_meta(crate::IngestParseMeta {
            parser: Some("vision".into()),
            pass: Some("a".into()),
            page_count: Some(12),
            fallback: Some(false),
            ..Default::default()
        });
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    let attrs = attr_map(&spans[0]);
    assert_eq!(
        attrs
            .get("langfuse.observation.metadata.parser")
            .map(String::as_str),
        Some("vision")
    );
    assert_eq!(
        attrs
            .get("langfuse.observation.metadata.pass")
            .map(String::as_str),
        Some("a")
    );
    assert_eq!(
        attrs
            .get("langfuse.observation.metadata.page_count")
            .map(String::as_str),
        Some("12")
    );
    assert_no_cost_keys(&attrs);
}

#[test]
fn inmemory_ingest_chunking_token_distribution() {
    let exporter = InMemorySpanExporterBuilder::new().build();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("edgequake-spec125-chunk-dist");

    let chunks = 2usize;
    let token_min = 5usize;
    let token_p50 = 40usize;
    let token_max = 80usize;
    let orphans = 0usize;
    let output = format!(
        "{{\"chunks\":{chunks},\"token_min\":{token_min},\"token_p50\":{token_p50},\"token_max\":{token_max},\"orphan_heading_chunks\":{orphans}}}"
    );

    tracer.in_span("ingest.chunking", |_cx| {
        record_observation_io(Some("{\"chars\":100}"), Some(&output));
        crate::record_ingest_kg_meta(crate::IngestKgMeta {
            chunk_strategy: Some("markdown".into()),
            chunk_size: Some(1200),
            overlap: Some(100),
            token_min: Some(token_min),
            token_p50: Some(token_p50),
            token_max: Some(token_max),
            orphan_heading_chunks: Some(orphans),
            ..Default::default()
        });
    });

    provider.force_flush().expect("flush");
    let spans = exporter.get_finished_spans().expect("spans");
    let attrs = attr_map(&spans[0]);
    let out = attrs
        .get(LANGFUSE_OBSERVATION_OUTPUT)
        .map(String::as_str)
        .unwrap_or("");
    assert_eq!(
        out, output,
        "output must be the recorded JSON, not a rewrite"
    );
    let min_s = token_min.to_string();
    assert_eq!(
        attrs
            .get("langfuse.observation.metadata.token_min")
            .map(String::as_str),
        Some(min_s.as_str())
    );
    assert_eq!(
        attrs
            .get("langfuse.observation.metadata.orphan_heading_chunks")
            .map(String::as_str),
        Some("0")
    );
    assert!(!out.contains("## "), "must not dump chunk text");
    assert_no_cost_keys(&attrs);
}
