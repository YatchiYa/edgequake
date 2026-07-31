//! SPEC-095: process-wide PDFium prime (extract + bind) before PDF traffic.
//!
//! SSOT: delegates to [`edgequake_pdf2md::prime_pdfium`]. Callers must not
//! reach into `pdfium-auto` directly.

use thiserror::Error;

/// Failure to extract / bind the bundled PDFium library at startup.
#[derive(Debug, Error)]
#[error("PDFium prime failed: {0}")]
pub struct PdfPrimeError(pub String);

/// Extract (if needed) and bind the process-wide PDFium singleton.
///
/// Safe to call repeatedly; subsequent calls are cheap once the singleton
/// is initialised. Does **not** cancel in-flight `spawn_blocking` extract
/// work — that is owned by `pdfium-auto` atomic publish + lock (SPEC-095).
pub fn prime_pdfium() -> Result<(), PdfPrimeError> {
    edgequake_pdf2md::prime_pdfium().map_err(|e| PdfPrimeError(e.to_string()))
}
