//! Accurate PDF page counting via pdfium (SSOT).
//!
//! ## WHY
//!
//! Byte-scanning for `/Count N` fails on PDFs that use compressed object
//! streams (common for arXiv). The vision outer timeout then under-budgets
//! (unknown → assume 50 pages → 520s on cloud) and permanently fails
//! legitimate long conversions.
//!
//! pdfium parses the real page tree, including object streams.

use std::io::Write;
use std::time::Duration;

use tracing::{debug, warn};

/// Soft ceiling for pdfium inspect during page-count heal (must not hang upload).
<<<<<<< HEAD
=======
///
/// SPEC-095: this timeout does **not** cancel `spawn_blocking` extract work
/// inside `edgequake-pdf2md` / `pdfium-auto`. Cache poison is fixed upstream
/// via atomic publish + size integrity; this remains a soft ceiling only.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
const COUNT_PAGES_TIMEOUT: Duration = Duration::from_secs(5);

/// Count pages in a PDF using pdfium via `edgequake-pdf2md::inspect`.
///
/// Returns `None` when the PDF cannot be opened, times out, or reports zero pages.
/// Callers should fall back to a byte-scan heuristic only as last resort.
pub async fn count_pdf_pages(pdf_bytes: &[u8]) -> Option<usize> {
    if pdf_bytes.len() < 5 || !pdf_bytes.starts_with(b"%PDF-") {
        return None;
    }

    let mut tmp = match tempfile::NamedTempFile::new() {
        Ok(t) => t,
        Err(e) => {
            warn!(error = %e, "count_pdf_pages: failed to create tempfile");
            return None;
        }
    };

    if let Err(e) = tmp.write_all(pdf_bytes) {
        warn!(error = %e, "count_pdf_pages: failed to write tempfile");
        return None;
    }
    if let Err(e) = tmp.flush() {
        warn!(error = %e, "count_pdf_pages: failed to flush tempfile");
        return None;
    }

    let path = tmp.path().to_string_lossy().to_string();
    let inspect = edgequake_pdf2md::inspect(&path);
    match tokio::time::timeout(COUNT_PAGES_TIMEOUT, inspect).await {
        Ok(Ok(meta)) if meta.page_count > 0 => {
            debug!(page_count = meta.page_count, "count_pdf_pages: pdfium ok");
            Some(meta.page_count)
        }
        Ok(Ok(meta)) => {
            warn!(
                page_count = meta.page_count,
                "count_pdf_pages: pdfium returned zero pages"
            );
            None
        }
        Ok(Err(e)) => {
            warn!(error = %e, "count_pdf_pages: pdfium inspect failed");
            None
        }
        Err(_) => {
            warn!(
                timeout_secs = COUNT_PAGES_TIMEOUT.as_secs(),
                "count_pdf_pages: pdfium inspect timed out — falling back"
            );
            None
        }
    }
}

/// Resolve page count: accurate pdfium first, then optional heuristic fallback.
///
/// `heuristic` is typically the `/Count` byte-scan used historically.
pub async fn resolve_pdf_page_count(pdf_bytes: &[u8], heuristic: Option<i32>) -> Option<i32> {
    if let Some(n) = count_pdf_pages(pdf_bytes).await {
        return Some(n as i32);
    }
    heuristic.filter(|&n| n > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid PDF with one blank page (no compressed object streams).
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
    async fn count_pdf_pages_reads_minimal_pdf() {
        let pages = count_pdf_pages(&minimal_one_page_pdf()).await;
        assert_eq!(pages, Some(1), "pdfium must count the single page");
    }

    #[tokio::test]
    async fn count_pdf_pages_empty_returns_none() {
        assert!(count_pdf_pages(&[]).await.is_none());
    }

    #[tokio::test]
    async fn resolve_falls_back_to_heuristic_when_corrupt() {
        // No %PDF- magic → skip pdfium entirely, use heuristic.
        let corrupt = b"not a pdf at all";
        let resolved = resolve_pdf_page_count(corrupt, Some(42)).await;
        assert_eq!(resolved, Some(42));
    }

    #[tokio::test]
    async fn resolve_falls_back_when_pdfium_rejects() {
        // Looks like a PDF but is truncated — pdfium fails or times out fast.
        let truncated = b"%PDF-1.4\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n";
        let resolved = resolve_pdf_page_count(truncated, Some(7)).await;
        assert_eq!(resolved, Some(7));
    }

    #[tokio::test]
    async fn resolve_prefers_pdfium_over_heuristic() {
        let resolved = resolve_pdf_page_count(&minimal_one_page_pdf(), Some(99)).await;
        assert_eq!(resolved, Some(1));
    }
}
