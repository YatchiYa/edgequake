//! SPEC-116 — Workspace chunking metadata apply (shared by Postgres + in-memory).

use std::collections::HashMap;

/// Apply SPEC-116 workspace chunking policy to metadata.
///
/// - `chunking_mode` `None` → leave unchanged (unless size fields alone update Fixed)
/// - clear tokens (`inherit`/`none`/`""`) → remove chunking keys
/// - `adaptive` → store mode only (clear size keys)
/// - `fixed` → store mode + size/overlap (defaults 1200/100); validate overlap < size
pub fn apply_chunking_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    chunking_mode: Option<String>,
    chunk_token_size: Option<u32>,
    chunk_overlap_token_size: Option<u32>,
) -> Result<(), String> {
    let Some(raw_mode) = chunking_mode else {
        if chunk_token_size.is_none() && chunk_overlap_token_size.is_none() {
            return Ok(());
        }
        let current = metadata
            .get("chunking_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("inherit");
        let mode = edgequake_pipeline::ChunkingMode::parse(current)
            .ok_or_else(|| format!("Unsupported chunking_mode '{current}'"))?;
        if mode != edgequake_pipeline::ChunkingMode::Fixed {
            return Err(
                "chunk_token_size / chunk_overlap_token_size require chunking_mode=fixed".into(),
            );
        }
        let size = chunk_token_size
            .map(|n| n as usize)
            .or_else(|| {
                metadata
                    .get("chunk_token_size")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize)
            })
            .unwrap_or(edgequake_pipeline::DEFAULT_FIXED_CHUNK_TOKEN_SIZE);
        let overlap = chunk_overlap_token_size
            .map(|n| n as usize)
            .or_else(|| {
                metadata
                    .get("chunk_overlap_token_size")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize)
            })
            .unwrap_or(edgequake_pipeline::DEFAULT_FIXED_CHUNK_OVERLAP);
        edgequake_pipeline::validate_fixed_pair(size, overlap)?;
        metadata.insert("chunking_mode".into(), serde_json::json!("fixed"));
        metadata.insert("chunk_token_size".into(), serde_json::json!(size));
        metadata.insert(
            "chunk_overlap_token_size".into(),
            serde_json::json!(overlap),
        );
        return Ok(());
    };

    let mode = edgequake_pipeline::ChunkingMode::parse(&raw_mode).ok_or_else(|| {
        format!(
            "Unsupported chunking_mode '{}'. Allowed: inherit, adaptive, fixed",
            raw_mode.trim()
        )
    })?;

    match mode {
        edgequake_pipeline::ChunkingMode::Inherit => {
            metadata.remove("chunking_mode");
            metadata.remove("chunk_token_size");
            metadata.remove("chunk_overlap_token_size");
            Ok(())
        }
        edgequake_pipeline::ChunkingMode::Adaptive => {
            metadata.insert("chunking_mode".into(), serde_json::json!("adaptive"));
            metadata.remove("chunk_token_size");
            metadata.remove("chunk_overlap_token_size");
            Ok(())
        }
        edgequake_pipeline::ChunkingMode::Fixed => {
            let size = chunk_token_size
                .map(|n| n as usize)
                .filter(|&n| n > 0)
                .unwrap_or(edgequake_pipeline::DEFAULT_FIXED_CHUNK_TOKEN_SIZE);
            let overlap = chunk_overlap_token_size
                .map(|n| n as usize)
                .unwrap_or(edgequake_pipeline::DEFAULT_FIXED_CHUNK_OVERLAP);
            edgequake_pipeline::validate_fixed_pair(size, overlap)?;
            metadata.insert("chunking_mode".into(), serde_json::json!("fixed"));
            metadata.insert("chunk_token_size".into(), serde_json::json!(size));
            metadata.insert(
                "chunk_overlap_token_size".into(),
                serde_json::json!(overlap),
            );
            Ok(())
        }
    }
}

/// Resolve pipeline policy from workspace metadata (SPEC-116).
pub fn chunking_policy_from_metadata(
    metadata: &HashMap<String, serde_json::Value>,
) -> Option<edgequake_pipeline::ChunkingPolicy> {
    edgequake_pipeline::chunking_policy_from_metadata(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_defaults_and_clear() {
        let mut meta = HashMap::new();
        apply_chunking_metadata(&mut meta, Some("fixed".into()), None, None).unwrap();
        assert_eq!(
            meta.get("chunking_mode").and_then(|v| v.as_str()),
            Some("fixed")
        );
        assert_eq!(
            meta.get("chunk_token_size").and_then(|v| v.as_u64()),
            Some(1200)
        );
        assert_eq!(
            meta.get("chunk_overlap_token_size")
                .and_then(|v| v.as_u64()),
            Some(100)
        );
        apply_chunking_metadata(&mut meta, Some("inherit".into()), None, None).unwrap();
        assert!(!meta.contains_key("chunking_mode"));
    }

    #[test]
    fn rejects_overlap_ge_size() {
        let mut meta = HashMap::new();
        let err = apply_chunking_metadata(&mut meta, Some("fixed".into()), Some(100), Some(100))
            .unwrap_err();
        assert!(err.contains("must be <"));
    }
}
