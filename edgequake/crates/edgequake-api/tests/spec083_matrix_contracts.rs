//! SPEC-083 e2e matrix contracts (Cluster 00–06) — source / ops assertions.
//!
//! Named to match `docs/083-improvements/04-e2e-test-matrix.md`.
//! Runtime Postgres e2e live in `e2e_postgres_rls.rs` / `e2e_websocket.rs`.

#![cfg(feature = "postgres")]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn contract_makefile_has_postgres_start_alias() {
    let makefile = std::fs::read_to_string(repo_root().join("Makefile")).unwrap();
    assert!(
        makefile.contains("postgres-start:") && makefile.contains("db-start"),
        "Makefile must alias postgres-start → db-start (D-47)"
    );
}

#[test]
fn contract_no_sed_i_empty_string() {
    let makefile = std::fs::read_to_string(repo_root().join("Makefile")).unwrap();
    // Allow the comment that mentions the anti-pattern; ban live invocations.
    let live = makefile
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .any(|l| l.contains("sed -i ''"));
    assert!(
        !live,
        "Makefile must not use BSD-only sed -i '' (D-49); use SED_INPLACE"
    );
}

#[test]
fn contract_env_example_vision_not_openai_by_default() {
    let env = std::fs::read_to_string(repo_root().join(".env.example")).unwrap();
    // Active (uncommented) assignment must not hardcode openai.
    let active_openai = env.lines().any(|l| {
        let t = l.trim();
        !t.starts_with('#') && t.starts_with("EDGEQUAKE_VISION_PROVIDER=") && t.contains("openai")
    });
    assert!(
        !active_openai,
        ".env.example must not default VISION_PROVIDER=openai (D-50)"
    );
    assert!(
        env.contains("EDGEQUAKE_EQ_ID_FALLBACK"),
        ".env.example should document EDGEQUAKE_EQ_ID_FALLBACK"
    );
}

#[test]
fn contract_no_eval_in_bench047() {
    let bench = repo_root().join("tools/bench047/bench047/mmlongbench_eval_score.py");
    let src = std::fs::read_to_string(&bench).expect("bench047 score module");
    assert!(
        src.contains("ast.literal_eval"),
        "bench047 must use ast.literal_eval (S-13)"
    );
    let bare_eval = src.lines().any(|l| {
        let t = l.trim_start();
        if t.contains("literal_eval") {
            return false;
        }
        t.starts_with("eval(") || t.contains("= eval(") || t.contains("(eval(")
    });
    assert!(!bare_eval, "no bare eval( in bench047");
}

#[test]
fn contract_gleaning_uses_completion_options() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-pipeline/src/extractor/gleaning.rs");
    let src = std::fs::read_to_string(path).unwrap();
    assert!(
        src.contains("extraction_completion_options")
            && src.contains(".chat(")
            && src.contains("with_provider_prompt_cache"),
        "gleaning must share extraction CompletionOptions via chat (C-17 / SPEC-126)"
    );
}

#[test]
fn contract_extract_chat_uses_shared_prompt_cache_options() {
    let extract = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-pipeline/src/extractor/llm.rs");
    let options = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-pipeline/src/extractor/completion_options.rs");
    let extract_src = std::fs::read_to_string(extract).unwrap();
    let options_src = std::fs::read_to_string(options).unwrap();
    assert!(
        extract_src.contains(".chat(") && extract_src.contains("extraction_completion_options"),
        "extract llm.rs must chat with shared extraction CompletionOptions (SPEC-126)"
    );
    assert!(
        options_src.contains("with_provider_prompt_cache(\"extract\""),
        "extraction CompletionOptions must attach eq:extract prompt_cache_key (SPEC-126)"
    );
}

#[test]
fn contract_batch_fetch_uses_get_by_ids() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-storage/src/chunk_content.rs");
    let src = std::fs::read_to_string(path).unwrap();
    assert!(
        src.contains("get_by_ids") || src.contains("get_by_ids_ordered"),
        "chunk_content must batch via get_by_ids (C-21)"
    );
}

#[test]
fn contract_no_substring_retry_matching_embeddings() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-pipeline/src/pipeline/helpers/embeddings.rs");
    let src = std::fs::read_to_string(path).unwrap();
    assert!(
        src.contains("retry_strategy()"),
        "embeddings must use typed retry_strategy"
    );
    assert!(
        !src.contains("contains(\"429\")") && !src.contains("contains(\"timeout\")"),
        "embeddings must not substring-match 429/timeout (X-07)"
    );
}

#[test]
fn contract_upload_uses_sanitize_filename() {
    let upload = include_str!("../src/handlers/pdf_upload/upload.rs");
    assert!(
        upload.contains("sanitize_filename"),
        "pdf upload must sanitize filenames (S-12)"
    );
    let fv = include_str!("../src/file_validation.rs");
    assert!(fv.contains("fn sanitize_filename"));
    assert!(fv.contains("fn contract_filename_strips_path"));
    assert!(fv.contains("sniff_magic_mime") || fv.contains("validate_magic"));
}

#[test]
fn contract_ws_deletion_match_named() {
    let ws = include_str!("../src/handlers/websocket.rs");
    assert!(
        ws.contains("fn contract_matches_track_id_deletion_variants"),
        "websocket must expose matrix-named Deletion* match test (C-24)"
    );
}

#[test]
fn contract_eq_id_schema_health_field() {
    let health_types = include_str!("../src/handlers/health_types.rs");
    assert!(
        health_types.contains("eq_id_schema"),
        "ComponentHealth must expose eq_id_schema"
    );
    let bootstrap = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(bootstrap.contains("eq_id_schema"));
}

#[test]
fn contract_with_rls_transaction_ssot() {
    let rls = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/rls.rs"),
    )
    .unwrap();
    assert!(rls.contains("pub async fn with_rls_transaction"));
    let conv = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/conversation.rs"),
    )
    .unwrap();
    assert!(
        !conv.contains("acquire_rls_connection("),
        "conversation.rs must not call acquire_rls_connection (S-03)"
    );
    assert!(conv.contains("with_rls_transaction"));
}

#[test]
fn contract_api_no_autocommit_rls_acquire() {
    // SPEC-083 S-03: production API auth/PDF paths must use with_optional_pg_rls.
    let roots = [
        "src/services/session_storage.rs",
        "src/services/identity_storage.rs",
        "src/services/pdf_lineage.rs",
        "src/services/tenant_isolation.rs",
    ];
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in roots {
        let src = std::fs::read_to_string(manifest.join(rel)).unwrap();
        assert!(
            !src.contains("acquire_rls_connection("),
            "{rel} must not call acquire_rls_connection"
        );
        assert!(
            !src.contains("acquire_optional_pg_connection("),
            "{rel} must not call acquire_optional_pg_connection"
        );
    }
    let session = include_str!("../src/services/session_storage.rs");
    let identity = include_str!("../src/services/identity_storage.rs");
    let pdf = include_str!("../src/services/pdf_lineage.rs");
    assert!(session.contains("with_optional_pg_rls"));
    assert!(identity.contains("with_optional_pg_rls"));
    assert!(pdf.contains("with_optional_pg_rls"));

    // Bare pool-level set_tenant_context is deprecated — production API must not call it.
    let api_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir_rs_files(&api_src) {
        let text = std::fs::read_to_string(&entry).unwrap_or_default();
        for (i, line) in text.lines().enumerate() {
            let t = line.trim();
            if t.starts_with("//") || t.starts_with("///") || t.starts_with("//!") {
                continue;
            }
            if t.contains("set_tenant_context(")
                && !t.contains("set_tenant_context_on_conn")
                && !entry.ends_with("tenant_isolation.rs")
            {
                offenders.push(format!("{}:{}", entry.display(), i + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "bare set_tenant_context outside helpers: {offenders:?}"
    );
}

fn walkdir_rs_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out
}

#[test]
fn contract_explain_trace_on_query_response() {
    let types = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-query/src/types.rs"),
    )
    .unwrap();
    assert!(types.contains("struct ExplainTrace"));
    assert!(types.contains("pub explain: Option<ExplainTrace>"));
    let api = include_str!("../src/handlers/query_types.rs");
    assert!(api.contains("ExplainTraceDto") || api.contains("explain:"));
}

// --- SPEC-083 full-pack audit: exact matrix names for FIXED evidence ---

#[test]
fn contract_eq_columns_present_after_reconcile() {
    let wired = include_str!("contract_spec069_delete_progress.rs");
    let mutate = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/graph/nodes_ops/mutate.rs"),
    )
    .unwrap_or_default();
    assert!(
        wired.contains("contract_m092_reconcile_wired") || mutate.contains("eq_node_id"),
        "X-03: reconcile / eq_* columns must be wired"
    );
}

#[test]
fn e2e_degrees_match_property_fallback() {
    let eq_sql = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/graph/helpers/eq_id_sql.rs"),
    )
    .unwrap();
    assert!(
        eq_sql.contains("COALESCE") && eq_sql.contains("eq_"),
        "X-03: degree SQL must COALESCE eq_* with properties"
    );
    assert!(
        eq_sql.contains("fn contract_degrees_coalesce_eq_and_props"),
        "matrix alias target contract_degrees_coalesce_eq_and_props must exist"
    );
}

#[test]
fn contract_native_upsert_eq_arbiter() {
    let mutate = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/graph/nodes_ops/mutate.rs"),
    )
    .unwrap();
    assert!(
        mutate.contains("ON CONFLICT") && mutate.contains("eq_"),
        "C-20: native upsert must target eq_* arbiter"
    );
}

#[test]
fn e2e_rate_limit_ignores_spoofed_header() {
    let mw = include_str!("../src/middleware.rs");
    assert!(
        mw.contains("fn authenticated_rate_limit_key_ignores_spoofed_tenant_header"),
        "S-11: spoofed x-tenant-id must be ignored in rate key"
    );
    assert!(
        mw.contains("fn e2e_rate_limit_ignores_spoofed_header"),
        "matrix-named alias must exist"
    );
}

#[test]
fn unit_weight_associative() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/merger/weight_policy.rs"),
    )
    .unwrap();
    assert!(
        src.contains("fn unit_weight_associative")
            || src.contains("fn unit_weight_max_associative"),
        "D-31: associative weight policy unit must exist"
    );
}

#[test]
fn unit_needs_llm_always_summarizes() {
    let merge = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/merger/description_merge.rs"),
    )
    .unwrap();
    let summarizer = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pipeline/src/summarizer.rs"),
    )
    .unwrap();
    assert!(
        merge.contains("1200") && summarizer.contains("1200"),
        "D-34: merger and summarizer gates must share 1200 token SSOT"
    );
    assert!(
        !merge.contains("4000") || merge.contains("DEFAULT_SUMMARY_MAX_TOKENS"),
        "D-34: no divergent 4000 summarizer gate"
    );
}

#[test]
fn contract_other_in_default_entity_types() {
    let prompts = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pipeline/src/prompts/mod.rs"),
    )
    .unwrap();
    assert!(
        prompts.contains("OTHER"),
        "X-15: OTHER must be in default entity types"
    );
    assert!(
        prompts.contains("fn contract_other_in_default_entity_types")
            || prompts.contains("fn test_default_entity_types"),
        "X-15 named contract must exist"
    );
}

#[test]
fn e2e_chunk_max_retries_zero_still_attempts_once_or_rejects() {
    let cfg = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/tests/e2e_timeout_config.rs"),
    )
    .unwrap_or_default();
    let extraction = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/pipeline/extraction.rs"),
    )
    .unwrap_or_default();
    assert!(
        cfg.contains("test_max_retries_zero_clamps_to_one_attempt")
            || extraction.contains(".max(1)"),
        "C-18: CHUNK_MAX_RETRIES=0 must still attempt once"
    );
}

#[test]
fn e2e_dedup_matches_completed_and_indexed() {
    let src = include_str!("../src/services/document_reingest.rs");
    assert!(
        src.contains("\"completed\"") && src.contains("\"indexed\""),
        "C-23: dedup must match completed and indexed"
    );
}

#[test]
fn unit_retry_has_jitter() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/pipeline/helpers/embeddings.rs"),
    )
    .unwrap();
    assert!(
        src.contains("Full jitter") || src.contains("% (base_ms"),
        "X-06: embed retry must use jittered backoff"
    );
    assert!(
        src.contains("retry_strategy()"),
        "X-06/X-07: typed retry_strategy required"
    );
    let err = include_str!("../src/error.rs");
    assert!(
        err.contains("CircuitBreakerOpen") || err.contains("circuit_breaker"),
        "X-06: CircuitBreakerOpen must be wired in API error mapping"
    );
}

#[test]
fn e2e_ollama_cosine_after_l2() {
    let emb = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/pipeline/helpers/embeddings.rs"),
    )
    .unwrap();
    assert!(
        emb.contains("l2_normalize") || emb.contains("L2"),
        "X-10: pipeline must L2-normalize embeddings"
    );
    let core = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-core/src/types/embedding.rs"),
    )
    .unwrap();
    assert!(
        core.contains("fn normalize") || core.contains("l2"),
        "X-10: Embedding::normalize SSOT required"
    );
}

#[test]
fn contract_fusion_mode_names() {
    let fusion = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-query/src/fusion.rs"),
    )
    .unwrap_or_else(|_| {
        std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../edgequake-query/src/retrieval/fusion.rs"),
        )
        .unwrap_or_default()
    });
    let rrf = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-query/tests/contract_rrf_fusion.rs"),
    )
    .unwrap_or_default();
    assert!(
        rrf.contains("contract_mix_fusion_env_modes")
            || fusion.contains("MaxAfterMinMax")
            || fusion.contains("max_after_minmax"),
        "D-35/D-36: fusion mode names must be contracted"
    );
    // D-35: operator label must not claim weighted sum for max-after-minmax path.
    assert!(
        fusion.contains("max_after_minmax")
            && fusion.contains("MaxAfterMinMax")
            && fusion.contains("not a weighted sum"),
        "D-35: Mix fusion docs/labels must describe max-after-minmax honesty"
    );
    assert!(
        fusion.contains("\"weighted\"") || fusion.contains("| \"weighted\""),
        "D-35: legacy weighted env alias must remain"
    );
}

#[test]
fn unit_score_scale_no_cross_compare() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-query/src/score_scale.rs"),
    )
    .expect("score_scale.rs");
    assert!(
        src.contains("fn unit_score_scale_no_cross_compare")
            && src.contains("enum ScoreScale")
            && src.contains("partial_cmp_compatible"),
        "D-37: ScoreScale must prevent cross-scale compare"
    );
}

#[test]
fn contract_stream_stats_superset() {
    let mapper = include_str!("../src/services/query_stats_mapper.rs");
    assert!(
        mapper.contains("fn contract_stream_stats_superset")
            || mapper.contains("fn stream_stats_from_context"),
        "D-40: stream stats SSOT helper required"
    );
    let types = include_str!("../src/handlers/query_types.rs");
    assert!(
        types.contains("fn from_query_stats") && types.contains("arms_run"),
        "D-40: QueryStreamStats must mirror QueryStats arm fields"
    );
}

#[test]
fn contract_vector_metric_cosine_only() {
    let caps = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/capabilities.rs"),
    )
    .unwrap();
    assert!(
        caps.contains("SUPPORTED_VECTOR_METRIC")
            && caps.contains("fn contract_vector_metric_cosine_only")
            && caps.contains("cosine-only"),
        "X-04: vector capabilities must document cosine-only"
    );
}

#[test]
fn e2e_fts_language_config() {
    let fts = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-storage/src/adapters/postgres/vector/fts.rs"),
    )
    .unwrap();
    assert!(
        fts.contains("fn e2e_fts_language_config")
            && fts.contains("EDGEQUAKE_FTS_LANGUAGE")
            && fts.contains("ts_rank_cd"),
        "X-05: FTS language config + ts_rank_cd honesty required"
    );
}

#[test]
fn contract_x_22() {
    let query_stream = include_str!("../src/handlers/query/query_stream.rs");
    let chat_stream = include_str!("../src/handlers/chat/streaming.rs");
    assert!(
        query_stream.contains("QueryStreamEvent::Thinking")
            && chat_stream.contains("ChatStreamEvent::Thinking"),
        "X-22: Thinking SSE must be emitted on query and chat stream paths"
    );
    let types = include_str!("../src/handlers/query_types.rs");
    assert!(
        types.contains("Thinking { content: String }")
            && types.contains("fn contract_x_22_thinking_stream_event"),
        "X-22: Thinking variant + unit contract required"
    );
}

#[test]
fn unit_progress_weighted() {
    let src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pipeline/src/progress/mod.rs"),
    )
    .unwrap();
    assert!(
        src.contains("fn unit_progress_weighted")
            || src.contains("fn test_progress_weighted_phases_d41"),
        "D-41: weighted progress unit must exist"
    );
}

#[test]
fn contract_cors_default_fail_closed_prod() {
    let startup = include_str!("../src/startup_security.rs");
    assert!(
        startup.contains("production_cors_missing_is_fatal")
            && startup.contains("fn contract_cors_default_fail_closed_prod"),
        "S-10: prod CORS missing must be fatal"
    );
    let server = include_str!("../src/server.rs");
    assert!(
        server.contains("cors_fail_closed")
            && server.contains("apply_cors_methods_headers")
            && server.contains("CORS_FAIL_CLOSED_METHODS"),
        "S-10: fail-closed CORS must use explicit method allow-list"
    );
    assert!(
        server.contains("AllowOrigin::Any is intentionally unreachable")
            || server.contains("AllowOrigin::Any is unreachable"),
        "S-10: AllowOrigin::Any must be unreachable when cors_fail_closed"
    );
    // Fail-closed arm of apply_cors_methods_headers must use explicit lists (not Any).
    let fail_closed_arm = server
        .split("fn apply_cors_methods_headers")
        .nth(1)
        .and_then(|s| s.split("if fail_closed {").nth(1))
        .and_then(|s| s.split("} else {").next())
        .unwrap_or("");
    let live_fail_closed: String = fail_closed_arm
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("///")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        live_fail_closed.contains("CORS_FAIL_CLOSED_METHODS")
            && live_fail_closed.contains("cors_fail_closed_headers()")
            && !live_fail_closed.contains("allow_methods(Any)")
            && !live_fail_closed.contains("allow_headers(Any)")
            && !live_fail_closed.contains("allow_origin(Any)"),
        "S-10: fail-closed CORS arm must not use Any for methods/headers"
    );
    // Runtime: empty allow-list + fail-closed builds without panic.
    let security = edgequake_api::state::ApiSecurityConfig {
        cors_origins: None,
        cors_fail_closed: true,
        ..Default::default()
    };
    let _layer = edgequake_api::build_cors_layer(&security);

    let e2e = include_str!("e2e_issue277_cors_production.rs");
    assert!(
        e2e.contains("fn e2e_ws_missing_origin_rejected_prod"),
        "S-10: e2e_ws_missing_origin_rejected_prod must exist"
    );
}

#[test]
fn contract_upload_limit_ssot_50mib() {
    // Single SSOT constant.
    assert_eq!(
        edgequake_core::MAX_UPLOAD_BYTES,
        50 * 1024 * 1024,
        "D-44: MAX_UPLOAD_BYTES must be 50 MiB"
    );
    assert_eq!(
        edgequake_pipeline::ValidationConfig::default().max_size_bytes,
        edgequake_core::MAX_UPLOAD_BYTES,
        "D-44: ValidationConfig default must match MAX_UPLOAD_BYTES"
    );

    let budget = include_str!("../../edgequake-core/src/resource/budget.rs");
    assert!(
        budget.contains("pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024"),
        "D-44: SSOT definition must live in budget.rs"
    );

    let validation = include_str!("../../edgequake-pipeline/src/validation.rs");
    assert!(
        validation.contains("50 * 1024 * 1024"),
        "D-44: ValidationConfig must use 50 MiB"
    );
    assert!(
        !validation
            .lines()
            .any(|l| l.contains("max_size_bytes:") && l.contains("100 * 1024 * 1024")),
        "D-44: ValidationConfig must not default to 100 MiB"
    );

    let injection = include_str!("../src/handlers/injection/injection_file.rs");
    assert!(
        injection.contains("edgequake_core::MAX_UPLOAD_BYTES"),
        "D-44: injection_file must use MAX_UPLOAD_BYTES"
    );
    assert!(
        injection.contains("50 MiB") || injection.contains("50MB") || injection.contains("50 MB"),
        "D-44: injection error message must mention 50"
    );
    assert!(
        !injection.contains("10 MB"),
        "D-44: injection_file must not claim 10 MB"
    );

    let pdf = include_str!("../../edgequake-storage/src/pdf_storage.rs");
    assert!(
        pdf.contains("50 * 1024 * 1024") && pdf.contains("50 MiB"),
        "D-44: pdf_storage validate_pdf_data must use 50 MiB SSOT"
    );
    assert!(
        !pdf.contains("104_857_600") && !pdf.contains("100MB limit"),
        "D-44: dead 100 MiB pdf limit must be removed"
    );
}

#[test]
fn contract_x_25_build_checks_routes_inventory() {
    let build = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build.rs"))
        .unwrap();
    assert!(
        build.contains("axum_route_paths") && build.contains("X-25"),
        "build.rs must inventory routes.rs against utoipa paths (X-25)"
    );
    assert!(
        build.contains("cargo:rerun-if-changed=src/routes.rs"),
        "build.rs must rerun when routes.rs changes"
    );
}

#[test]
fn contract_ci_no_continue_on_error_critical() {
    let pg =
        std::fs::read_to_string(repo_root().join(".github/workflows/postgres-integration.yml"))
            .unwrap();
    let e2e = std::fs::read_to_string(repo_root().join(".github/workflows/e2e-quality-gates.yml"))
        .unwrap();

    // RLS step must not be soft-failed.
    let rls_idx = pg
        .find("Run PostgreSQL RLS E2E Tests")
        .expect("RLS job step present");
    let rls_window = &pg[rls_idx..rls_idx + 500.min(pg.len() - rls_idx)];
    assert!(
        !rls_window.contains("continue-on-error: true"),
        "RLS E2E step must not use continue-on-error: true (X-32)"
    );

    // Required AGE / migration proof steps must block.
    for needle in [
        "Run SPEC-006 migration bootstrap postgres e2e",
        "Run PostgreSQL AGE Graph Tests",
        "Run SPEC-017 storage backend contracts",
    ] {
        let idx = pg
            .find(needle)
            .unwrap_or_else(|| panic!("missing step: {needle}"));
        let window = &pg[idx..idx + 450.min(pg.len() - idx)];
        assert!(
            !window.contains("continue-on-error: true"),
            "{needle} must not continue-on-error: true"
        );
    }

    // Critical E2E path job must not soft-fail at job level.
    let crit = e2e.find("e2e-critical:").expect("e2e-critical job");
    let crit_window = &e2e[crit..crit + 350.min(e2e.len() - crit)];
    assert!(
        !crit_window.contains("continue-on-error: true"),
        "e2e-critical job must not set continue-on-error: true"
    );
    // Quarantine-only full suite may keep continue-on-error.
    assert!(
        e2e.contains("e2e-full:") && e2e.contains("quarantine"),
        "e2e-full must be documented as quarantine when using continue-on-error"
    );
}

#[test]
fn contract_frontend_test_must_run() {
    let makefile = std::fs::read_to_string(repo_root().join("Makefile")).unwrap();
    let idx = makefile
        .find("frontend-test:")
        .expect("frontend-test target");
    // Only the frontend-test recipe body (until next target), ignoring comments.
    let rest = &makefile[idx..];
    let body_end = rest
        .lines()
        .skip(1)
        .take_while(|l| {
            l.starts_with('\t') || l.trim().is_empty() || l.trim_start().starts_with('#')
        })
        .map(|l| l.len() + 1)
        .sum::<usize>();
    let window: String = rest[..body_end.min(rest.len())]
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !window.contains("|| echo"),
        "frontend-test must not use `|| echo` false-green (X-32)"
    );
    assert!(
        window.contains("pnpm test") || window.contains("bun test"),
        "frontend-test must invoke pnpm/bun test"
    );
}

#[test]
fn contract_embed_batch_ssot() {
    // SPEC-083 X-08: single env SSOT; no silent secondary default clamp when unset.
    let safety = include_str!("../src/safety_limits.rs");
    assert!(
        safety.contains("EDGEQUAKE_EMBEDDING_BATCH_SIZE"),
        "safety_limits must read EDGEQUAKE_EMBEDDING_BATCH_SIZE"
    );
    assert!(
        safety.contains("EMBED_BATCH_NO_OVERRIDE") || safety.contains("usize::MAX"),
        "unset env must not force a second default clamp over provider.max_batch_size"
    );
    let env = std::fs::read_to_string(repo_root().join(".env.example")).unwrap();
    let assignments: Vec<_> = env
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.contains("EDGEQUAKE_EMBEDDING_BATCH_SIZE=") && !t.starts_with('#')
        })
        .collect();
    assert!(
        assignments.is_empty(),
        "active .env.example must not hardcode conflicting batch sizes; got {assignments:?}"
    );
    let commented = env
        .lines()
        .filter(|l| l.contains("EDGEQUAKE_EMBEDDING_BATCH_SIZE="))
        .count();
    assert_eq!(
        commented, 1,
        "X-08: document exactly one EDGEQUAKE_EMBEDDING_BATCH_SIZE line in .env.example"
    );
}

#[test]
fn contract_x_13() {
    let pipeline_marker = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/chunker/page_marker.rs"),
    )
    .unwrap();
    let pdf_marker = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pdf/src/page_marker.rs"),
    )
    .unwrap();
    assert!(pipeline_marker.contains("struct PageMarkerWriter"));
    assert!(pdf_marker.contains("struct PageMarkerWriter"));
    assert!(pipeline_marker.contains("<!-- edgequake-page:"));
    assert!(pdf_marker.contains("<!-- edgequake-page:"));
    assert!(pipeline_marker.contains("strip_before_restamp"));
    let edgeparse = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pdf/src/backend/edgeparse.rs"),
    )
    .unwrap();
    assert!(
        !edgeparse.contains("const PAGE_MARKER_PREFIX"),
        "edgeparse must use PageMarkerWriter SSOT, not local constants"
    );
    let vision = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pdf/src/vision_markdown.rs"),
    )
    .unwrap();
    assert!(
        !vision.contains("const PAGE_MARKER_PREFIX"),
        "vision_markdown must use PageMarkerWriter SSOT, not local constants"
    );
}

#[test]
fn contract_x_14() {
    let types = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/chunker/types.rs"),
    )
    .unwrap();
    assert!(
        types.contains("default_recursive_separators()"),
        "ChunkerConfig::default must use LightRAG cascade (X-14)"
    );
    let recursive = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/chunker/recursive.rs"),
    )
    .unwrap();
    assert!(
        recursive.contains("String::new()") || recursive.contains("\"\".to_string()"),
        "default_recursive_separators must include final empty separator"
    );
    assert!(
        recursive.contains("contract_x_14_default_separators_are_lightrag_cascade")
            || recursive.contains("default_separators_match_lightrag"),
        "unit contract for LightRAG separators must exist"
    );
}

#[test]
fn contract_cache_set_or_module_removed() {
    // SPEC-083 D-52: prefer delete of never-set CachedExtractor.
    let cache_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pipeline/src/cache.rs");
    if cache_path.exists() {
        let src = std::fs::read_to_string(&cache_path).unwrap();
        assert!(
            src.contains(".set(") && !src.contains("write skipped"),
            "if cache.rs remains, set() must be wired and called"
        );
    } else {
        let lib = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-pipeline/src/lib.rs"),
        )
        .unwrap();
        assert!(
            !lib.contains("pub mod cache") && !lib.contains("CachedExtractor"),
            "cache module removed — lib.rs must not re-export CachedExtractor"
        );
    }
}

#[test]
fn contract_single_source_id_cap() {
    // SPEC-083 C-26: core MAX_SOURCE_IDS == pipeline merge_limits DEFAULT (200).
    assert_eq!(edgequake_core::GraphEntity::MAX_SOURCE_IDS, 200);
    assert_eq!(edgequake_core::GraphRelationship::MAX_SOURCE_IDS, 200);
    assert_eq!(
        edgequake_core::GraphEntity::MAX_SOURCE_IDS,
        edgequake_pipeline::DEFAULT_MAX_SOURCE_IDS
    );
    let entity_src = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-core/src/types/entity.rs"),
    )
    .unwrap();
    assert!(
        !entity_src.contains("MAX_SOURCE_IDS: usize = 300"),
        "dead MAX_SOURCE_IDS=300 must be gone"
    );
}

#[test]
fn e2e_shutdown_drains_or_cancels_within_budget() {
    let shutdown = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-tasks/src/shutdown.rs"),
    )
    .unwrap();
    assert!(
        shutdown.contains("EDGEQUAKE_SHUTDOWN_DRAIN_SECS")
            && shutdown.contains("DEFAULT_SHUTDOWN_DRAIN_SECS"),
        "X-31: shutdown drain env/default must exist"
    );
    let worker = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-tasks/src/worker.rs"),
    )
    .unwrap();
    assert!(
        worker.contains("e2e_shutdown_drains_or_cancels_within_budget")
            && worker.contains("shutdown_drain_budget")
            && worker.contains("cancel_all_active"),
        "X-31: worker shutdown must use drain budget + cancel"
    );
    let server = include_str!("../src/server.rs");
    assert!(
        server.contains("with_graceful_shutdown")
            && server.contains("e2e_shutdown_drains_or_cancels_within_budget"),
        "X-31: server must graceful-shutdown with drain budget test"
    );
}

#[test]
fn e2e_batch_file_cap() {
    assert_eq!(edgequake_core::MAX_BATCH_UPLOAD_FILES, 20);
    let upload = include_str!("../src/handlers/pdf_upload/upload.rs");
    assert!(
        upload.contains("ensure_batch_file_cap"),
        "D-51: pdf batch must enforce file cap"
    );
    let batch = include_str!("../src/handlers/documents/upload/batch_upload.rs");
    assert!(
        batch.contains("ensure_batch_file_cap"),
        "D-51: document batch must enforce file cap"
    );
    let mp = include_str!("../src/multipart_upload.rs");
    assert!(
        mp.contains("fn e2e_batch_file_cap"),
        "D-51: unit e2e_batch_file_cap must exist"
    );
}

#[test]
fn e2e_upload_streams_to_temp() {
    let mp = include_str!("../src/multipart_upload.rs");
    assert!(
        mp.contains("stream_field_to_tempfile")
            && mp.contains("NamedTempFile")
            && mp.contains(".chunk()")
            && mp.contains("fn e2e_upload_streams_to_temp_contract"),
        "D-51: multipart must stream chunks to NamedTempFile"
    );
    let pdf = include_str!("../src/handlers/pdf_upload/upload.rs");
    let file = include_str!("../src/handlers/documents/upload/file_upload.rs");
    assert!(
        pdf.contains("stream_field_to_tempfile") && file.contains("stream_field_to_tempfile"),
        "D-51: upload handlers must use stream_field_to_tempfile"
    );
    for (label, src) in [("pdf", pdf), ("file", file)] {
        let live = src
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("///")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !live.contains(".bytes()"),
            "D-51: {label} upload must not buffer via Field::bytes()"
        );
    }
}

#[test]
fn unit_anthropic_url_image_source() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-pipeline/src/anthropic_images.rs");
    let src = std::fs::read_to_string(&path).unwrap();
    assert!(
        src.contains("fn unit_anthropic_url_image_source")
            && src.contains("anthropic_image_source_json")
            && src.contains("\"type\": \"url\""),
        "C-25: pipeline shim must expose unit_anthropic_url_image_source"
    );
    // Blocker documentation: crates.io edgequake-llm still forces base64.
    assert!(
        src.contains("crates.io") || src.contains("edgequake-llm"),
        "C-25: must document external llm blocker"
    );
}

#[test]
fn contract_admission_rejects_over_budget_wired() {
    let admission = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-tasks/src/admission.rs"),
    )
    .unwrap();
    assert!(
        admission.contains("fn contract_admission_rejects_over_budget"),
        "X-19: admission unit contract must exist"
    );
    let worker = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-tasks/src/worker.rs"),
    )
    .unwrap();
    assert!(
        worker.contains("try_admit") && worker.contains("estimate_task_bytes"),
        "X-19: worker claim path must call admission try_admit"
    );
}

// --- SPEC-083 Wave C ---

#[test]
fn contract_tasks_pk_documented() {
    let readme =
        std::fs::read_to_string(repo_root().join("edgequake/migrations/README.md")).unwrap();
    assert!(
        readme.contains("X-01")
            && readme.contains("001_init_database.sql")
            && readme.contains("002_add_tasks_table.sql")
            && readme.contains("Dead / no-op"),
        "X-01: migrations README must document dead 002"
    );
    let note = repo_root().join("docs/083-improvements/X-01-dead-migration-002.md");
    assert!(note.exists(), "X-01 operator note must exist");
}

#[test]
fn contract_checksum_drift_fails_loud() {
    let m071 = include_str!("../src/state/migration_bootstrap/reconcile/m071.rs");
    let m078 = include_str!("../src/state/migration_bootstrap/reconcile/m078.rs");
    let m118 = include_str!("../src/state/migration_bootstrap/reconcile/m118.rs");
    let m121 = include_str!("../src/state/migration_bootstrap/reconcile/m121.rs");
    for (label, src) in [
        ("m071", m071),
        ("m078", m078),
        ("m118", m118),
        ("m121", m121),
    ] {
        assert!(
            src.contains("allow_checksum_repair")
                && src.contains("EDGEQUAKE_DEV_MODE")
                && src.contains("Refusing silent repair"),
            "X-02: {label} must fail loud without DEV_MODE"
        );
    }
}

#[test]
fn contract_single_edgequake_llm_version() {
    let lock = std::fs::read_to_string(repo_root().join("edgequake/Cargo.lock")).unwrap();
    let mut versions = Vec::new();
    let mut lines = lock.lines().peekable();
    while let Some(line) = lines.next() {
        if line == "name = \"edgequake-llm\"" {
            if let Some(v) = lines.next() {
                if let Some(ver) = v.strip_prefix("version = \"") {
                    versions.push(ver.trim_end_matches('"').to_string());
                }
            }
        }
    }
    versions.sort();
    versions.dedup();
    // Workspace pin 0.10.x + documented pdf2md transitive 0.6.x until pdf2md upgrades.
    assert!(
        !versions.is_empty() && versions.len() <= 2,
        "X-09: at most two edgequake-llm versions (workspace + pdf2md); got {versions:?}"
    );
    assert!(
        versions.iter().any(|v| v.starts_with("0.10.")),
        "X-09: workspace must pin edgequake-llm 0.10.x; got {versions:?}"
    );
    let cargo = std::fs::read_to_string(repo_root().join("edgequake/Cargo.toml")).unwrap();
    assert!(
        cargo.contains("X-09") && cargo.contains("edgequake-llm = \"0.10."),
        "X-09: Cargo.toml must document diamond + pin 0.10.x"
    );
}

#[test]
fn e2e_reindex_embedding_model_change() {
    // X-11 pragmatic: Scan/Reindex job types return 501 (not creatable).
    let registry = include_str!("../src/services/job_registry.rs");
    assert!(
        registry.contains("NOT_IMPLEMENTED_V2_JOB_TYPES")
            && registry.contains("\"scan\"")
            && registry.contains("\"reindex\""),
        "X-11: Scan/Reindex must be marked not-implemented"
    );
    assert!(
        !registry
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
            .contains("CREATABLE_V2_JOB_TYPES: &[\"upload\", \"insert\", \"scan\""),
        "X-11: scan must not be in creatable list"
    );
    let submission = include_str!("../src/handlers/v2/jobs/submission.rs");
    assert!(
        submission.contains("NOT_IMPLEMENTED_V2_JOB_TYPES")
            && submission.contains("NotImplemented"),
        "X-11: submission must return 501 for Scan/Reindex"
    );
}

#[test]
fn contract_pdf_concurrency_schedule() {
    let src = include_str!("../src/processor/pdf_processing.rs");
    assert!(
        src.contains("CLOUD_PDF_PAGE_CONCURRENCY") && src.contains("X-12"),
        "X-12: decorative match replaced by documented constant"
    );
    let live: String = src
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("///")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !live.contains("0..=49 => 2"),
        "X-12: decorative page-count match arms must be gone"
    );
}

#[test]
fn contract_x_23() {
    let ws = include_str!("../src/handlers/websocket.rs");
    assert!(
        ws.contains("RecvError::Lagged")
            && ws.contains("Client lagged")
            && (ws.contains("events dropped") || ws.contains("Reconnect")),
        "X-23: Lagged must notify client (not silent continue)"
    );
}

#[test]
fn contract_x_26() {
    let pkg = std::fs::read_to_string(repo_root().join("edgequake_webui/package.json")).unwrap();
    assert!(
        pkg.contains("codegen:api") && pkg.contains("openapi/schema.d.ts") && pkg.contains("X-26"),
        "X-26: package.json must document schema.d.ts generation"
    );
    let wire = repo_root().join("edgequake_webui/src/types/openapi-schema.ts");
    assert!(
        wire.exists(),
        "X-26: webui must import/re-export OpenAPI schema types"
    );
}

#[test]
fn contract_x_27() {
    let mw = std::fs::read_to_string(repo_root().join("edgequake_webui/middleware.ts")).unwrap();
    assert!(
        mw.contains("edgequake_access_token")
            && mw.contains("NextResponse.redirect")
            && mw.contains("/login"),
        "X-27: middleware.ts must redirect unauthenticated protected routes"
    );
    let ctx =
        std::fs::read_to_string(repo_root().join("edgequake_webui/src/lib/api/client-context.ts"))
            .unwrap();
    assert!(
        ctx.contains("edgequake_access_token") && ctx.contains("syncAuthCookie"),
        "X-27: client must mirror token to cookie for middleware"
    );
}

#[test]
fn contract_sdk_major_matches_server_policy() {
    let policy = std::fs::read_to_string(repo_root().join("docs/sdks/VERSION-POLICY.md")).unwrap();
    assert!(
        policy.contains("X-33") && policy.contains("0.4") && policy.contains("0.20"),
        "X-33: VERSION-POLICY.md must document client vs product versions"
    );
    let rust = std::fs::read_to_string(repo_root().join("sdks/rust/Cargo.toml")).unwrap();
    let py = std::fs::read_to_string(repo_root().join("sdks/python/pyproject.toml")).unwrap();
    let ts = std::fs::read_to_string(repo_root().join("sdks/typescript/package.json")).unwrap();
    assert!(rust.contains("version = \"0.4."));
    assert!(py.contains("version = \"0.4."));
    assert!(ts.contains("\"version\": \"0.4."));
}

#[test]
fn contract_config_precedence() {
    let orch = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-core/src/orchestrator/mod.rs"),
    )
    .unwrap();
    assert!(
        orch.contains("fn resolve(")
            && orch.contains("EdgeQuakeConfigOverrides")
            && orch.contains("fn contract_config_precedence")
            && orch.contains("X-36"),
        "X-36: EdgeQuakeConfig::resolve + contract_config_precedence required"
    );
}

#[test]
fn e2e_progress_survives_restart() {
    let progress = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-tasks/src/progress.rs"),
    )
    .unwrap();
    assert!(
        progress.contains("fn e2e_progress_survives_restart"),
        "D-42: unit e2e_progress_survives_restart must exist"
    );
    // Attrs immediately above the field definition must not skip serde.
    let field_idx = progress
        .find("avg_item_time_ms: f64")
        .expect("avg_item_time_ms field");
    let before = &progress[..field_idx];
    let attr_window = before.lines().rev().take(6).collect::<Vec<_>>().join("\n");
    assert!(
        !attr_window.contains("serde(skip)"),
        "D-42: avg_item_time_ms must not use serde(skip); attrs: {attr_window}"
    );
    assert!(
        attr_window.contains("serde(default)") || progress.contains("#[serde(default)]"),
        "D-42: avg_item_time_ms should use serde(default) for backward-compat"
    );
}

#[test]
fn contract_single_audit_definition() {
    let readme =
        std::fs::read_to_string(repo_root().join("edgequake/migrations/README.md")).unwrap();
    assert!(
        readme.contains("D-45") && readme.contains("audit_logs") && readme.contains("SSOT"),
        "D-45: migrations README must declare audit_logs SSOT"
    );
    let m001 =
        std::fs::read_to_string(repo_root().join("edgequake/migrations/001_init_database.sql"))
            .unwrap();
    assert!(
        m001.contains("create_next_audit_log_partition"),
        "D-45: 001 must define create_next_audit_log_partition"
    );
}

#[test]
fn e2e_audit_insert_next_month_partition() {
    let bootstrap = include_str!("../src/state/migration_bootstrap/mod.rs");
    assert!(
        bootstrap.contains("create_next_audit_log_partition")
            && bootstrap.contains("audit_next_month_partition"),
        "D-45: bootstrap must call create_next_audit_log_partition"
    );
    let fn_sql =
        std::fs::read_to_string(repo_root().join("edgequake/migrations/001_init_database.sql"))
            .unwrap();
    assert!(
        fn_sql.contains("CREATE OR REPLACE FUNCTION create_next_audit_log_partition"),
        "D-45: partition create function must exist in 001"
    );
}

#[test]
fn contract_otel_respects_rust_log() {
    let sub = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-observability/src/subscriber.rs"),
    )
    .unwrap();
    assert!(
        sub.contains("fn contract_otel_respects_rust_log")
            && sub.contains("otel.with_filter(otel_filter)")
            && sub.contains("D-46"),
        "D-46: EnvFilter must wrap OTEL via with_filter"
    );
}

#[test]
fn contract_no_nested_github_workflows_or_root_dispatch() {
    let root_wf = repo_root().join(".github/workflows/sdk-ci.yml");
    assert!(root_wf.exists(), "D-48: root sdk-ci.yml must exist");
    let yml = std::fs::read_to_string(&root_wf).unwrap();
    assert!(
        yml.contains("D-48")
            && yml.contains("sdks/python")
            && yml.contains("sdks/typescript")
            && yml.contains("sdks/rust"),
        "D-48: root workflow must cover Tier-1 SDKs"
    );
    let note = repo_root().join("sdks/README-CI.md");
    assert!(
        note.exists(),
        "D-48: sdks/README-CI.md must explain nested ignore"
    );
}

// --- SPEC-083 Wave D: accuracy / identity / Louvain ---

#[test]
fn e2e_entity_type_conflict_logged_and_resolved() {
    let entity = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/merger/entity.rs"),
    )
    .unwrap();
    assert!(
        entity.contains("fn e2e_entity_type_conflict_logged_and_resolved"),
        "D-32: e2e_entity_type_conflict_logged_and_resolved must exist"
    );
    let vote = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/merger/entity_type_vote.rs"),
    )
    .unwrap();
    assert!(
        vote.contains("resolve_majority_type") && vote.contains("entity_type_conflict"),
        "D-32: majority/confidence vote module required"
    );
}

#[test]
fn contract_x_17() {
    let fuzzy = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-storage/src/entity_fuzzy.rs"),
    )
    .unwrap();
    assert!(
        fuzzy.contains("EDGEQUAKE_ENTITY_FUZZY")
            && fuzzy.contains("fn find_best_fuzzy_match")
            && fuzzy.contains("fn contract_x_17_fuzzy_off_by_default"),
        "X-17: entity_fuzzy module + contract required"
    );
    let env = std::fs::read_to_string(repo_root().join(".env.example")).unwrap();
    assert!(
        env.contains("EDGEQUAKE_ENTITY_FUZZY"),
        "X-17: .env.example must document EDGEQUAKE_ENTITY_FUZZY"
    );
}

#[test]
fn e2e_x_17() {
    let entity = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../edgequake-pipeline/src/merger/entity.rs"),
    )
    .unwrap();
    assert!(
        entity.contains("fn contract_x_17_fuzzy_collapses_near_duplicate_when_enabled")
            || entity.contains("entity_fuzzy_enabled"),
        "X-17: merger must wire fuzzy resolution"
    );
    let fuzzy = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-storage/src/entity_fuzzy.rs"),
    )
    .unwrap();
    assert!(
        fuzzy.contains("fn e2e_x_17_blocking_rejects_different_blocks"),
        "X-17: e2e_x_17 blocking test must exist"
    );
}

#[test]
fn unit_louvain_hierarchy_levels() {
    let community = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-storage/src/community.rs"),
    )
    .unwrap();
    assert!(
        community.contains("fn unit_louvain_hierarchy_levels")
            && community.contains("EDGEQUAKE_LOUVAIN_HIERARCHY")
            && community.contains("aggregate_communities"),
        "D-54: Louvain phase-2 hierarchy + unit_louvain_hierarchy_levels required"
    );
    let env = std::fs::read_to_string(repo_root().join(".env.example")).unwrap();
    assert!(
        env.contains("EDGEQUAKE_LOUVAIN_HIERARCHY"),
        "D-54: .env.example must document EDGEQUAKE_LOUVAIN_HIERARCHY"
    );
}

#[test]
fn nightly_golden_acc_gate() {
    let golden = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-query/src/eval/golden_acc.rs"),
    )
    .unwrap();
    assert!(
        golden.contains("fn nightly_golden_acc_gate")
            && golden.contains("score_golden_set_deterministic")
            && golden.contains("fn nightly_golden_acc_gate_live_llm"),
        "X-34: nightly_golden_acc_gate must score fixtures (not count-only)"
    );
}

#[test]
fn bench_acc_at_n_regression_gate() {
    let acc = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../edgequake-query/src/eval/acc_at_n.rs"),
    )
    .unwrap();
    assert!(
        acc.contains("fn bench_acc_at_n_regression_gate")
            && acc.contains("regression_floor_acc_at_40"),
        "X-35: bench_acc_at_n_regression_gate required"
    );
    let floors = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../edgequake-query/tests/fixtures/acc_at_n_floors.json");
    assert!(
        floors.exists(),
        "X-35: acc_at_n_floors.json fixture required"
    );
}
