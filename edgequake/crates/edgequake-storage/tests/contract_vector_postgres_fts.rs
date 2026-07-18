//! SPEC-023 I10 — Postgres native FTS on vector chunk content.

#[test]
fn contract_postgres_vector_fts_joins_shared_kv_for_chunk_text() {
    let fts = include_str!("../src/adapters/postgres/vector/fts.rs");
    assert!(fts.contains("ts_rank_cd"));
    assert!(fts.contains("websearch_to_tsquery"));
    assert!(fts.contains("chunk_kv_table_name"));
    assert!(fts.contains("k.value->>'content'"));
    assert!(fts.contains("LEFT JOIN"));
    assert!(fts.contains("content_tsv"));
    // SPEC-058: empty generated tsv must not block KV fallthrough.
    assert!(fts.contains("NULLIF(v.content_tsv"));
    assert!(fts.contains("content_ref"));
}

#[test]
fn contract_spec058_upsert_populates_content_tsv() {
    let impl_src = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    let ddl = include_str!("../src/adapters/postgres/vector/ddl.rs");
    let migration = include_str!("../../../migrations/091_vector_content_tsv_writable.sql");
    assert!(
        impl_src.contains("content_tsv = EXCLUDED.content_tsv"),
        "upsert must write content_tsv"
    );
    assert!(
        !ddl.contains("GENERATED ALWAYS AS"),
        "ddl must not recreate generated content_tsv"
    );
    assert!(
        migration.contains("ADD COLUMN content_tsv TSVECTOR"),
        "migration 091 must add writable content_tsv"
    );
}

#[test]
fn contract_workspace_vector_uses_shared_chunk_kv_table() {
    let ws = include_str!("../src/adapters/postgres/workspace_vector.rs");
    assert!(
        ws.contains("qualified_kv_table"),
        "workspace vectors must join the shared default KV for FTS"
    );
}

#[test]
fn contract_postgres_vector_fts_filters_modality_metadata() {
    let fts = include_str!("../src/adapters/postgres/vector/fts.rs");
    let storage_impl = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    assert!(fts.contains("Failed to bind modalities"));
    assert!(storage_impl.contains("Failed to bind modalities"));
    assert!(fts.contains("build_sql_with_alias"));
}

#[test]
fn contract_vector_ddl_adds_content_tsv() {
    let ddl = include_str!("../src/adapters/postgres/vector/ddl.rs");
    assert!(ddl.contains("content_tsv"));
    assert!(ddl.contains("ensure_content_fts"));
}
