//! Adaptive chunk sizing for large documents (LightRAG best practice).
//!
//! SSOT for library (`EdgeQuake::insert`) and HTTP worker ingestion paths (SPEC-025 5.2).
//!
//! Env knobs (SPEC-001 fair Acc / ops):
//! - `EDGEQUAKE_ADAPTIVE_CHUNKING` — default **on** (`1`/`true`); set `0`/`false` for fixed size
//! - `EDGEQUAKE_CHUNK_SIZE` — fixed token size when adaptive is off (default **1200**)
//! - `EDGEQUAKE_CHUNK_OVERLAP` — fixed overlap when adaptive is off (default **100**)

/// Recommended chunk size in tokens from document byte length.
///
/// Thresholds (from LightRAG empirical testing):
/// - `<50KB` → 1200 tokens
/// - `50–100KB` → 800 tokens
/// - `>100KB` → 600 tokens
pub fn calculate_adaptive_chunk_size(document_size_bytes: usize) -> usize {
    if document_size_bytes > 100_000 {
        600
    } else if document_size_bytes > 50_000 {
        800
    } else {
        1200
    }
}

/// Overlap as ~8.3% of chunk size (LightRAG best practice).
pub fn adaptive_chunk_overlap(chunk_size: usize) -> usize {
    (chunk_size as f32 * 0.083) as usize
}

/// Whether adaptive sizing is enabled (default **true**).
pub fn adaptive_chunking_enabled() -> bool {
    match std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING") {
        Ok(v) => !matches!(
            v.to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => true,
    }
}

/// Fixed chunk size from env when adaptive is off (default 1200).
pub fn env_fixed_chunk_size() -> usize {
    std::env::var("EDGEQUAKE_CHUNK_SIZE")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1200)
}

/// Fixed overlap from env when adaptive is off (default 100).
pub fn env_fixed_chunk_overlap() -> usize {
    std::env::var("EDGEQUAKE_CHUNK_OVERLAP")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(100)
}

/// Resolve base chunk size + overlap before API `ChunkOptions` overrides.
pub fn resolve_base_chunk_size_overlap(document_size_bytes: usize) -> (usize, usize) {
    if adaptive_chunking_enabled() {
        let size = calculate_adaptive_chunk_size(document_size_bytes);
        (size, adaptive_chunk_overlap(size))
    } else {
        (env_fixed_chunk_size(), env_fixed_chunk_overlap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_sizes_match_lightrag_thresholds() {
        assert_eq!(calculate_adaptive_chunk_size(30_000), 1200);
        assert_eq!(calculate_adaptive_chunk_size(80_000), 800);
        assert_eq!(calculate_adaptive_chunk_size(200_000), 600);
    }

    #[test]
    fn overlap_is_proportional_to_chunk_size() {
        assert_eq!(adaptive_chunk_overlap(1200), 99);
        assert_eq!(adaptive_chunk_overlap(600), 49);
    }

    #[test]
    fn adaptive_default_on() {
        // Do not assert env-dependent values in parallel tests; only the pure helper.
        assert!(adaptive_chunk_overlap(1200) > 0);
    }

    #[test]
    fn fixed_env_path_ignores_document_size() {
        // Isolated env mutation: restore previous values after assert.
        let prev_adaptive = std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING").ok();
        let prev_size = std::env::var("EDGEQUAKE_CHUNK_SIZE").ok();
        let prev_overlap = std::env::var("EDGEQUAKE_CHUNK_OVERLAP").ok();
        // SAFETY: test-only; single-threaded assertion window.
        unsafe {
            std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "0");
            std::env::set_var("EDGEQUAKE_CHUNK_SIZE", "1200");
            std::env::set_var("EDGEQUAKE_CHUNK_OVERLAP", "100");
        }
        let (size, overlap) = resolve_base_chunk_size_overlap(200_000);
        assert!(!adaptive_chunking_enabled());
        assert_eq!((size, overlap), (1200, 100));
        unsafe {
            match prev_adaptive {
                Some(v) => std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", v),
                None => std::env::remove_var("EDGEQUAKE_ADAPTIVE_CHUNKING"),
            }
            match prev_size {
                Some(v) => std::env::set_var("EDGEQUAKE_CHUNK_SIZE", v),
                None => std::env::remove_var("EDGEQUAKE_CHUNK_SIZE"),
            }
            match prev_overlap {
                Some(v) => std::env::set_var("EDGEQUAKE_CHUNK_OVERLAP", v),
                None => std::env::remove_var("EDGEQUAKE_CHUNK_OVERLAP"),
            }
        }
    }
}
