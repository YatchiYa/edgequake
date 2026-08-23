//! SPEC-135 E2E-135-01 — Pdf-chunked span fixture persists page_start/page_end.
//!
//! ```bash
//! cargo test -p edgequake-api --features postgres --test e2e_spec135_pdf_pack
//! ```

#![cfg(feature = "postgres")]

mod common;

use std::sync::Arc;

use edgequake_pipeline::pipeline::ProcessingResult;
use edgequake_pipeline::{
    persist_relational_chunks, resolve_chunker, ChunkStrategy, ChunkerConfig,
    IngestionPersistContext,
};
use edgequake_storage::PostgresChunkRepository;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use common::spec013_postgres::try_database_url;
use serial_test::serial;

const DEFAULT_TENANT: &str = "00000000-0000-0000-0000-000000000002";
const DEFAULT_WORKSPACE: &str = "00000000-0000-0000-0000-000000000003";
const SHA_SPAN: &str = "6c35a71bf672ce91f26b2bbfb04ba46958555b7cc6d7885be445cdd1605d1f44";

fn acc_fair_pdf() -> ChunkerConfig {
    ChunkerConfig {
        chunk_size: 1200,
        chunk_overlap: 100,
        min_chunk_size: 100,
        ..Default::default()
    }
}

fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../specs/135-chunking/fixtures")
}

#[tokio::test]
#[serial]
async fn e2e_135_01_span_pages_persist_not_null() {
    std::env::remove_var("EDGEQUAKE_PDF_PACK");
    std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");

    let Some(url) = try_database_url() else {
        eprintln!("SKIP e2e_135_01_span_pages_persist_not_null: no DATABASE_URL");
        return;
    };

    let path = fixtures_dir().join("span_tiny.md");
    let bytes = std::fs::read(&path).expect("span_tiny.md");
    let got = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(got, SHA_SPAN, "fixture span_tiny.md SHA-256 mismatch");
    let text = String::from_utf8(bytes).expect("utf-8");

    let chunker = resolve_chunker(ChunkStrategy::Pdf, acc_fair_pdf());
    let chunks = chunker
        .chunk_async(&text, "eq135-e2e")
        .await
        .expect("pdf chunk");
    assert_eq!(chunks.len(), 1, "P2 must emit one span chunk");
    assert_eq!(chunks[0].page_start, Some(1));
    assert_eq!(chunks[0].page_end, Some(2));

    let pool = PgPool::connect(&url).await.expect("connect");
    let tenant = Uuid::parse_str(DEFAULT_TENANT).unwrap();
    let workspace = Uuid::parse_str(DEFAULT_WORKSPACE).unwrap();
    let doc_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO public.documents (id, tenant_id, workspace_id, title, content, status, metadata)
        VALUES ($1, $2, $3, 'SPEC-135 span', $4, 'processing',
                jsonb_build_object('source_type', 'pdf'))
        ON CONFLICT (id) DO UPDATE SET updated_at = now()
        "#,
    )
    .bind(doc_id)
    .bind(tenant)
    .bind(workspace)
    .bind(&text)
    .execute(&pool)
    .await
    .expect("ensure document parent");

    let ctx = IngestionPersistContext::new(
        doc_id.to_string(),
        Some(tenant.to_string()),
        Some(workspace.to_string()),
    );
    let result = ProcessingResult {
        document_id: doc_id.to_string(),
        chunks,
        extractions: vec![],
        stats: Default::default(),
        lineage: None,
    };

    let repo = Arc::new(PostgresChunkRepository::new(pool.clone()));
    persist_relational_chunks(repo.as_ref(), &ctx, &result)
        .await
        .expect("persist relational chunks");

    let null_starts: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM public.chunks WHERE document_id = $1 AND page_start IS NULL",
    )
    .bind(doc_id)
    .fetch_one(&pool)
    .await
    .expect("count null page_start");
    assert_eq!(null_starts, 0, "E2E-135-01: zero NULL page_start");

    let rows: Vec<(Option<i32>, Option<i32>, i64)> = sqlx::query_as(
        r#"
        SELECT page_start, page_end, count(*)
        FROM public.chunks
        WHERE document_id = $1
        GROUP BY 1, 2
        "#,
    )
    .bind(doc_id)
    .fetch_all(&pool)
    .await
    .expect("group by span");

    assert_eq!(rows.len(), 1, "expected one span group, got {rows:?}");
    assert_eq!(rows[0].0, Some(1));
    assert_eq!(rows[0].1, Some(2));
    assert_eq!(rows[0].2, 1);
}
