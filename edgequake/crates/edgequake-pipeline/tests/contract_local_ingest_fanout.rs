//! Contract: local-ingest fan-out clamps (process_batch floor, PG hold across LLM).

#[test]
fn process_batch_has_no_floor_of_four() {
    let src = include_str!("../src/pipeline/processing.rs");
    assert!(
        !src.contains(".max(4)"),
        "process_batch must not re-introduce .max(4) fan-out floor for local extract"
    );
    assert!(
        src.contains("max_concurrent_extractions.max(1)"),
        "process_batch should use clamped extract concurrency with floor 1"
    );
}

/// Extraction owns only the concurrency semaphore across LLM awaits — not a PG conn.
#[test]
fn extraction_does_not_acquire_pg_across_extract() {
    let src = include_str!("../src/pipeline/extraction.rs");
    assert!(
        !src.contains("pool.acquire"),
        "extraction must not hold a PG pool connection across extractor.extract (Ollama) awaits"
    );
    assert!(
        src.contains("semaphore"),
        "extraction should gate fan-out with a semaphore only"
    );
}
