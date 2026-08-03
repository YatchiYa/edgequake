//! Shared typed-embedding write gates (SPEC-091 dim SSOT).
//!
//! DRY for `PgChunkEmbeddingIndex` / `PgFleetEmbeddingIndex`: reject mixed
//! or out-of-range dimensions before SQL so pgvector typmod errors never
//! surface as a transient "Database unavailable" failure.

use crate::adapters::postgres::capabilities::HNSW_MAX_DIM_HALFVEC;
use crate::error::StorageError;

/// Reject dimensions outside the halfvec HNSW ceiling (defense-in-depth).
pub fn validate_ann_dimensions(dimensions: i32) -> Result<i32, StorageError> {
    if dimensions <= 0 || dimensions as usize > HNSW_MAX_DIM_HALFVEC {
        return Err(StorageError::InvalidInput(format!(
            "embedding dimension {dimensions} out of range (expected 1..={HNSW_MAX_DIM_HALFVEC})"
        )));
    }
    Ok(dimensions)
}

/// Validate a homogeneous embedding batch before typed upsert.
///
/// Returns the shared dimension on success.
pub fn validate_typed_embedding_batch_dims(
    dims: impl IntoIterator<Item = i32>,
    embedding_lens: impl IntoIterator<Item = usize>,
) -> Result<i32, StorageError> {
    let dims: Vec<i32> = dims.into_iter().collect();
    let lens: Vec<usize> = embedding_lens.into_iter().collect();
    if dims.is_empty() {
        return Err(StorageError::InvalidInput(
            "empty embedding batch".to_string(),
        ));
    }
    if dims.len() != lens.len() {
        return Err(StorageError::InvalidInput(
            "embedding dimension metadata length mismatch".to_string(),
        ));
    }
    let dimensions = validate_ann_dimensions(dims[0])?;
    if dims.iter().any(|d| *d != dimensions) {
        return Err(StorageError::InvalidInput(
            "mixed dimensions in one batch".to_string(),
        ));
    }
    for (i, len) in lens.iter().enumerate() {
        if *len as i32 != dimensions {
            return Err(StorageError::InvalidInput(format!(
                "embedding dimension mismatch: expected {dimensions}, not {len} (row {i})"
            )));
        }
    }
    Ok(dimensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_homogeneous_1024() {
        assert_eq!(
            validate_typed_embedding_batch_dims([1024, 1024], [1024, 1024]).unwrap(),
            1024
        );
    }

    #[test]
    fn rejects_mixed() {
        let err = validate_typed_embedding_batch_dims([1024, 1536], [1024, 1536]).unwrap_err();
        assert!(err.to_string().contains("mixed dimensions"));
    }

    #[test]
    fn rejects_len_mismatch() {
        let err = validate_typed_embedding_batch_dims([1024], [768]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("expected 1024"));
        assert!(msg.contains("not 768"));
    }

    #[test]
    fn rejects_over_ceiling() {
        let err = validate_typed_embedding_batch_dims([4001], [4001]).unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }
}
