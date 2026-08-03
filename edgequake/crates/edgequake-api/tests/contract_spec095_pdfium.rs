//! SPEC-095 contract: startup primes PDFium; env SSOT documented.

use std::fs;
use std::path::PathBuf;

fn read_repo(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn spec095_main_primes_pdfium_before_listen() {
    let main = read_repo("../../src/main.rs");
    assert!(
        main.contains("prime_pdfium"),
        "main.rs must call prime_pdfium (SPEC-095)"
    );
    assert!(
        main.contains("EDGEQUAKE_SKIP_PDFIUM_PRIME"),
        "main.rs must honour EDGEQUAKE_SKIP_PDFIUM_PRIME opt-out"
    );
    assert!(
        main.contains("spawn_blocking(prime_pdfium)"),
        "prime must run on blocking pool before listen"
    );

    let prime_idx = main
        .find("spawn_blocking(prime_pdfium)")
        .expect("prime call");
    let server_new = main.find("Server::new").expect("Server::new");
    let server_run = main.find("server.run()").expect("server.run");
    assert!(
        prime_idx < server_new && server_new < server_run,
        "prime must appear before Server::new and server.run (fail-closed boot)"
    );
}

#[test]
fn spec095_skip_opt_out_documented() {
    let main = read_repo("../../src/main.rs");
    assert!(
        main.contains("EDGEQUAKE_SKIP_PDFIUM_PRIME"),
        "skip opt-out env must be documented in main"
    );
    let env = read_repo("../../../.env.example");
    assert!(
        env.contains("EDGEQUAKE_SKIP_PDFIUM_PRIME") || env.contains("PDFIUM_AUTO_CACHE_DIR"),
        ".env.example must document pdfium env SSOT"
    );
}

#[test]
fn spec095_pdf_facade_ssot() {
    let ready = read_repo("../edgequake-pdf/src/pdfium_ready.rs");
    assert!(ready.contains("edgequake_pdf2md::prime_pdfium"));
    assert!(ready.contains("PdfPrimeError"));

    let lib = read_repo("../edgequake-pdf/src/lib.rs");
    assert!(lib.contains("pdfium_ready"));
    assert!(lib.contains("prime_pdfium"));
}

#[test]
fn spec095_page_count_documents_non_cancel() {
    let page_count = read_repo("../edgequake-pdf/src/page_count.rs");
    assert!(
        page_count.contains("SPEC-095"),
        "page_count must document that timeout does not cancel extract"
    );
}
