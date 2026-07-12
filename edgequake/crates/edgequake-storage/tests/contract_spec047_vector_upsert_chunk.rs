//! SPEC-047 P3 — tunable vector upsert chunk size.

#[test]
fn vector_upsert_chunk_size_ssot() {
    let vector = include_str!("../src/traits/vector.rs");
    assert!(
        vector.contains("fn vector_upsert_chunk_size"),
        "vector_upsert_chunk_size must be the SSOT"
    );
    assert!(
        vector.contains("EDGEQUAKE_VECTOR_UPSERT_CHUNK"),
        "env var required"
    );
    let storage = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    assert!(
        storage.contains("vector_upsert_chunk_size"),
        "PgVectorStorage upsert must use tunable chunk size"
    );
    assert!(
        !storage.contains("const CHUNK: usize = 1_000"),
        "hardcoded CHUNK=1000 must be removed"
    );
}
