//! SPEC-135 — unfakable PDF pack-to-budget contracts.
//!
//! Loads committed fixture bytes from `specs/135-chunking/fixtures/` and
//! asserts SHA-256 before gold checks. Uses real `ChunkStrategy::Pdf`.

use std::path::PathBuf;

use edgequake_pipeline::{
    count_tokens, ingest_chunking_observation_full, is_html_comment_only, resolve_chunker,
    ChunkStrategy, ChunkerConfig, PDF_PACK_ENV,
};
use serde_json::Value;
use serial_test::serial;
use sha2::{Digest, Sha256};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../specs/135-chunking/fixtures")
}

fn load_fixture(name: &str) -> (Vec<u8>, String) {
    let path = fixtures_dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let text = String::from_utf8(bytes.clone()).expect("utf-8 fixture");
    (bytes, text)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn assert_sha256(name: &str, expected: &str) -> String {
    let (bytes, text) = load_fixture(name);
    let got = sha256_hex(&bytes);
    assert_eq!(
        got, expected,
        "fixture {name} SHA-256 mismatch — do not silently swap bytes"
    );
    text
}

fn gold() -> Value {
    let raw = std::fs::read_to_string(fixtures_dir().join("freetoken_like.gold.json"))
        .expect("gold json");
    serde_json::from_str(&raw).expect("gold parse")
}

fn acc_fair_pdf() -> ChunkerConfig {
    ChunkerConfig {
        chunk_size: 1200,
        chunk_overlap: 100,
        min_chunk_size: 100,
        ..Default::default()
    }
}

fn median_tokens(tokens: &[usize]) -> usize {
    if tokens.is_empty() {
        return 0;
    }
    let mut t = tokens.to_vec();
    t.sort_unstable();
    t[t.len() / 2]
}

const SHA_FREETOKEN: &str = "0f3b59fffe97a005c5d063075845699e1c42eda8d92fa7cab78efcd580c33be5";
const SHA_SPAN: &str = "6c35a71bf672ce91f26b2bbfb04ba46958555b7cc6d7885be445cdd1605d1f44";
const SHA_OVERSIZE: &str = "0e840925e3134fb10e2149bb7ee976f9b920e2e2f7dfad602258559d06ba1c72";
const SHA_H1: &str = "90ac09bc62c76c5f7e7e4a6e83d5077fc00c2f231fcc97fe9cbcde8c2f907a8f";

/// U-135-FILL + U-135-PROBE + U-135-TIKTOKEN + U-135-NO-COMMENT + OTEL keys
#[tokio::test]
#[serial]
async fn u135_fill_probe_tiktoken_no_comment() {
    std::env::remove_var(PDF_PACK_ENV);
    std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");

    let gold = gold();
    assert_eq!(gold["sha256"].as_str().unwrap(), SHA_FREETOKEN);
    let text = assert_sha256("freetoken_like.md", SHA_FREETOKEN);
    let with_comment = format!("{text}\n\n<!-- multimodal-chunks -->\n");

    let chunker = resolve_chunker(ChunkStrategy::Pdf, acc_fair_pdf());
    let chunks = chunker
        .chunk_async(&with_comment, "eq135-fill")
        .await
        .expect("pdf pack");

    let n_min = gold["n_min"].as_u64().unwrap() as usize;
    let n_max = gold["n_max"].as_u64().unwrap() as usize;
    let fill_min = gold["fill_p50_min"].as_f64().unwrap();
    let n = chunks.len();
    let tokens: Vec<usize> = chunks.iter().map(|c| c.token_count).collect();
    let p50 = median_tokens(&tokens);
    let fill = p50 as f64 / 1200.0;
    assert!(
        n >= n_min && n <= n_max,
        "U-135-FILL N={n} not in gold [{n_min},{n_max}]"
    );
    assert!(
        fill >= fill_min,
        "U-135-FILL fill_p50={fill:.3} < {fill_min} (p50={p50})"
    );

    let fig = gold["probe_fig"].as_str().unwrap();
    let prose = gold["probe_prose"].as_str().unwrap();
    assert!(
        chunks
            .iter()
            .any(|c| c.content.contains(fig) && c.content.contains(prose)),
        "U-135-PROBE {fig} and {prose} must share a chunk"
    );

    for c in &chunks {
        assert_eq!(
            c.token_count,
            count_tokens(&c.content),
            "U-135-TIKTOKEN mismatch on chunk {}",
            c.index
        );
        let trimmed = c.content.trim();
        assert!(
            !is_html_comment_only(trimmed),
            "U-135-NO-COMMENT leaked {trimmed:?}"
        );
    }

    let (_, output, dist) = ingest_chunking_observation_full(
        with_comment.len(),
        chunks.iter().map(|c| (c.token_count, c.content.as_str())),
        Some(1200),
        false,
    );
    assert!(output.contains("\"fill_p50\""));
    assert!(output.contains("\"mm_sidecar_appended\""));
    assert!(!output.contains("PROBE_FIG_A"), "must not emit chunk body");
    assert_eq!(dist.token_p50, p50);
}

/// U-135-SPAN
#[tokio::test]
#[serial]
async fn u135_span_tiny_pages_merge() {
    std::env::remove_var(PDF_PACK_ENV);
    std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");
    let text = assert_sha256("span_tiny.md", SHA_SPAN);
    let chunker = resolve_chunker(ChunkStrategy::Pdf, acc_fair_pdf());
    let chunks = chunker.chunk_async(&text, "eq135-span").await.unwrap();
    assert_eq!(
        chunks.len(),
        1,
        "U-135-SPAN expected 1 chunk, got {}",
        chunks.len()
    );
    assert_eq!(chunks[0].page_start, Some(1));
    assert_eq!(chunks[0].page_end, Some(2));
}

/// E1: new H1 blocks cross-page pack
#[tokio::test]
#[serial]
async fn u135_h1_blocks_span() {
    std::env::remove_var(PDF_PACK_ENV);
    std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");
    let text = assert_sha256("h1_block_span.md", SHA_H1);
    let chunker = resolve_chunker(ChunkStrategy::Pdf, acc_fair_pdf());
    let chunks = chunker.chunk_async(&text, "eq135-h1").await.unwrap();
    assert_eq!(
        chunks.len(),
        2,
        "E1 expected 2 chunks (no merge across new H1), got {}",
        chunks.len()
    );
    assert!(chunks.iter().any(|c| c.content.contains("EQ135_H1_P1")));
    assert!(chunks.iter().any(|c| c.content.contains("EQ135_H1_P2")));
}

/// U-135-NO-SPAN-OVERSIZE
#[tokio::test]
#[serial]
async fn u135_oversize_page_still_splits() {
    std::env::remove_var(PDF_PACK_ENV);
    std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");
    let text = assert_sha256("oversize_page.md", SHA_OVERSIZE);
    let chunker = resolve_chunker(ChunkStrategy::Pdf, acc_fair_pdf());
    let chunks = chunker.chunk_async(&text, "eq135-oversize").await.unwrap();
    assert!(
        chunks.len() >= 2,
        "U-135-NO-SPAN-OVERSIZE must split, got {}",
        chunks.len()
    );
    let joined = chunks
        .iter()
        .map(|c| c.content.as_str())
        .collect::<String>();
    for i in [0usize, 50, 100, 150, 219] {
        let needle = format!("EQ135_P1_S{i}");
        assert!(joined.contains(&needle), "silent drop of {needle}");
    }
    for c in &chunks {
        if let (Some(s), Some(e)) = (c.page_start, c.page_end) {
            assert_eq!(s, e, "single-page oversize must not invent a span");
        }
    }
}

/// U-135-KILL
#[tokio::test]
#[serial]
async fn u135_kill_restores_legacy_n() {
    let gold = gold();
    let text = assert_sha256("freetoken_like.md", SHA_FREETOKEN);
    let cfg = acc_fair_pdf();

    std::env::set_var(PDF_PACK_ENV, "0");
    std::env::set_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK", "0");
    assert!(
        !edgequake_pipeline::pdf_pack_enabled(),
        "kill switch must disable pdf pack"
    );
    let chunker = resolve_chunker(ChunkStrategy::Pdf, cfg);
    let legacy = chunker.chunk_async(&text, "eq135-kill").await.unwrap();
    std::env::remove_var(PDF_PACK_ENV);
    std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");

    let n = legacy.len();
    let n_legacy = gold["n_legacy"].as_u64().unwrap() as usize;
    let packed_n = gold["n"].as_u64().unwrap() as usize;
    assert_eq!(
        n, n_legacy,
        "U-135-KILL frozen Recursive N={n} expected {n_legacy}"
    );
    assert_ne!(n, packed_n, "U-135-KILL must not match packed N={packed_n}");
    assert!(
        legacy
            .iter()
            .any(|c| c.token_count != count_tokens(&c.content)),
        "U-135-KILL Recursive inner must stamp word-count tokens, not tiktoken"
    );
}

/// Registry Pdf uses page_aware name.
#[test]
fn u135_registry_pdf_is_page_aware() {
    let chunker = resolve_chunker(ChunkStrategy::Pdf, acc_fair_pdf());
    assert_eq!(chunker.strategy_name(), "page_aware");
}
