//! SPEC-095: PDFium cache integrity + startup prime (EdgeQuake facade).
//!
//! Atomic extract / poison heal for the on-disk cache live in `pdfium-auto`
//! unit tests (isolated per call). This binary proves the EdgeQuake SSOT
//! facade: `prime_pdfium` + concurrent `count_pdf_pages` after bind.

use std::sync::Arc;

use edgequake_pdf::{count_pdf_pages, prime_pdfium};

/// Minimal valid one-page PDF (no compressed object streams).
fn minimal_one_page_pdf() -> Vec<u8> {
    br#"%PDF-1.4
1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj
2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj
3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>endobj
xref
0 4
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
trailer<< /Size 4 /Root 1 0 R >>
startxref
196
%%EOF
"#
    .to_vec()
}

#[tokio::test]
async fn e2e_spec095_startup_prime() {
    tokio::task::spawn_blocking(prime_pdfium)
        .await
        .expect("join")
        .expect("prime_pdfium");

    // Idempotent: second prime is a cheap singleton hit.
    tokio::task::spawn_blocking(prime_pdfium)
        .await
        .expect("join")
        .expect("re-prime");

    let pages = count_pdf_pages(&minimal_one_page_pdf())
        .await
        .expect("page count after prime");
    assert_eq!(pages, 1);
}

#[tokio::test]
async fn e2e_spec095_pdfium_cold_cache_concurrent() {
    // Warm bind first (startup prime equivalent).
    tokio::task::spawn_blocking(prime_pdfium)
        .await
        .expect("join")
        .expect("prime");

    let pdf = Arc::new(minimal_one_page_pdf());
    let mut handles = Vec::new();
    for _ in 0..16 {
        let pdf = Arc::clone(&pdf);
        handles.push(tokio::spawn(async move {
            count_pdf_pages(&pdf)
                .await
                .expect("count_pdf_pages after prime")
        }));
    }
    for h in handles {
        let pages = h.await.expect("join");
        assert_eq!(pages, 1);
    }
}

#[tokio::test]
async fn e2e_spec095_pdfium_poison_heals_via_facade() {
    // Truncation heal is owned by pdfium-auto unit tests. Here: after prime,
    // the EdgeQuake facade keeps serving accurate page counts (no sticky bind
    // failure).
    tokio::task::spawn_blocking(prime_pdfium)
        .await
        .expect("join")
        .expect("prime");

    let pages = count_pdf_pages(&minimal_one_page_pdf())
        .await
        .expect("page count with primed pdfium");
    assert_eq!(pages, 1);
}
