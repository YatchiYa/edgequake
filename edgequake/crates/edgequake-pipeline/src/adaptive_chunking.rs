//! Adaptive chunk sizing for large documents (LightRAG best practice).
//!
//! SSOT for library (`EdgeQuake::insert`) and HTTP worker ingestion paths
//! (SPEC-025 5.2, SPEC-116 workspace policy).
//!
//! Env knobs (SPEC-001 fair Acc / ops) — used when policy is [`ChunkingPolicy::Inherit`]:
//! - `EDGEQUAKE_ADAPTIVE_CHUNKING` — default **on** (`1`/`true`); set `0`/`false` for fixed size
//! - `EDGEQUAKE_CHUNK_SIZE` — fixed token size when adaptive is off (default **1200**)
//! - `EDGEQUAKE_CHUNK_OVERLAP` — fixed overlap when adaptive is off (default **100**)
//!
//! Precedence (LAW-116-2): document `ChunkOptions` > workspace [`ChunkingPolicy`] > fleet env.

use serde::{Deserialize, Serialize};

/// Acc-fair / LightRAG paper defaults (SPEC-001 / SPEC-116).
pub const DEFAULT_FIXED_CHUNK_TOKEN_SIZE: usize = 1200;
pub const DEFAULT_FIXED_CHUNK_OVERLAP: usize = 100;

/// Workspace (or explicit) chunking policy (SPEC-116).
///
/// `Inherit` defers to fleet env. `Adaptive` forces adaptive thresholds even when
/// env adaptive is off. `Fixed` forces exact size/overlap (Acc-fair pin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingPolicy {
    /// Use fleet `EDGEQUAKE_ADAPTIVE_CHUNKING` + fixed env sizes.
    #[default]
    Inherit,
    /// Force adaptive ON (1200/800/600 by document bytes).
    Adaptive,
    /// Force adaptive OFF with explicit sizes.
    Fixed { size: usize, overlap: usize },
}

impl ChunkingPolicy {
    /// Acc-fair / LightRAG pin: Fixed 1200/100.
    pub fn acc_fair() -> Self {
        Self::Fixed {
            size: DEFAULT_FIXED_CHUNK_TOKEN_SIZE,
            overlap: DEFAULT_FIXED_CHUNK_OVERLAP,
        }
    }

    /// Parse workspace metadata mode string (`inherit`/`adaptive`/`fixed`).
    pub fn parse_mode(raw: &str) -> Option<ChunkingMode> {
        ChunkingMode::parse(raw)
    }

    /// Build Fixed with defaults when size/overlap omitted.
    pub fn fixed_or_default(size: Option<usize>, overlap: Option<usize>) -> Result<Self, String> {
        let size = size
            .filter(|&n| n > 0)
            .unwrap_or(DEFAULT_FIXED_CHUNK_TOKEN_SIZE);
        let overlap = overlap.unwrap_or(DEFAULT_FIXED_CHUNK_OVERLAP);
        validate_fixed_pair(size, overlap)?;
        Ok(Self::Fixed { size, overlap })
    }
}

/// Wire mode without sizes (API / metadata).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingMode {
    Inherit,
    Adaptive,
    Fixed,
}

impl ChunkingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Adaptive => "adaptive",
            Self::Fixed => "fixed",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "inherit" | "none" | "default" | "auto" => Some(Self::Inherit),
            "adaptive" | "on" => Some(Self::Adaptive),
            "fixed" | "off" | "fair" | "lightrag" | "acc" => Some(Self::Fixed),
            _ => None,
        }
    }

    pub fn is_clear(raw: &str) -> bool {
        matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "" | "inherit" | "none" | "default" | "auto"
        )
    }
}

/// Reject overlap >= size (LightRAG invariant).
pub fn validate_fixed_pair(size: usize, overlap: usize) -> Result<(), String> {
    if size == 0 {
        return Err("chunk_token_size must be >= 1".into());
    }
    if overlap >= size {
        return Err(format!(
            "chunk_overlap_token_size ({overlap}) must be < chunk_token_size ({size})"
        ));
    }
    Ok(())
}

/// Resolve [`ChunkingPolicy`] from workspace metadata (SPEC-116).
///
/// Missing / inherit / clear → `None` (caller treats as Inherit / fleet env).
pub fn chunking_policy_from_metadata(
    metadata: &std::collections::HashMap<String, serde_json::Value>,
) -> Option<ChunkingPolicy> {
    let mode_raw = metadata.get("chunking_mode").and_then(|v| v.as_str())?;
    let mode = ChunkingMode::parse(mode_raw)?;
    match mode {
        ChunkingMode::Inherit => None,
        ChunkingMode::Adaptive => Some(ChunkingPolicy::Adaptive),
        ChunkingMode::Fixed => {
            let size = metadata
                .get("chunk_token_size")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .filter(|&n| n > 0)
                .unwrap_or(DEFAULT_FIXED_CHUNK_TOKEN_SIZE);
            let overlap = metadata
                .get("chunk_overlap_token_size")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize)
                .unwrap_or(DEFAULT_FIXED_CHUNK_OVERLAP);
            if validate_fixed_pair(size, overlap).is_err() {
                return Some(ChunkingPolicy::acc_fair());
            }
            Some(ChunkingPolicy::Fixed { size, overlap })
        }
    }
}

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

/// Whether adaptive sizing is enabled in fleet env (default **true**).
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
        .unwrap_or(DEFAULT_FIXED_CHUNK_TOKEN_SIZE)
}

/// Fixed overlap from env when adaptive is off (default 100).
pub fn env_fixed_chunk_overlap() -> usize {
    std::env::var("EDGEQUAKE_CHUNK_OVERLAP")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_FIXED_CHUNK_OVERLAP)
}

/// Resolve base chunk size + overlap from fleet env only (Inherit path).
pub fn resolve_base_chunk_size_overlap(document_size_bytes: usize) -> (usize, usize) {
    resolve_base_chunk_size_overlap_with_policy(document_size_bytes, None)
}

/// Resolve base size/overlap with optional workspace [`ChunkingPolicy`] (SPEC-116).
///
/// Document `ChunkOptions` are applied later by [`crate::build_chunker_config`].
pub fn resolve_base_chunk_size_overlap_with_policy(
    document_size_bytes: usize,
    policy: Option<&ChunkingPolicy>,
) -> (usize, usize) {
    let policy = policy.copied().unwrap_or(ChunkingPolicy::Inherit);
    match policy {
        ChunkingPolicy::Inherit => {
            if adaptive_chunking_enabled() {
                let size = calculate_adaptive_chunk_size(document_size_bytes);
                (size, adaptive_chunk_overlap(size))
            } else {
                (env_fixed_chunk_size(), env_fixed_chunk_overlap())
            }
        }
        ChunkingPolicy::Adaptive => {
            let size = calculate_adaptive_chunk_size(document_size_bytes);
            (size, adaptive_chunk_overlap(size))
        }
        ChunkingPolicy::Fixed { size, overlap } => (size, overlap),
    }
}

/// Whether the effective policy uses adaptive thresholds (for small-doc floor).
pub fn policy_uses_adaptive(policy: Option<&ChunkingPolicy>) -> bool {
    match policy.copied().unwrap_or(ChunkingPolicy::Inherit) {
        ChunkingPolicy::Inherit => adaptive_chunking_enabled(),
        ChunkingPolicy::Adaptive => true,
        ChunkingPolicy::Fixed { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize env-mutating tests (process-global env is racy otherwise).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        assert!(adaptive_chunk_overlap(1200) > 0);
    }

    #[test]
    fn fixed_env_path_ignores_document_size() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev_adaptive = std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING").ok();
        let prev_size = std::env::var("EDGEQUAKE_CHUNK_SIZE").ok();
        let prev_overlap = std::env::var("EDGEQUAKE_CHUNK_OVERLAP").ok();
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

    #[test]
    fn workspace_adaptive_wins_over_env_off() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING").ok();
        unsafe {
            std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "0");
        }
        let (size, _) =
            resolve_base_chunk_size_overlap_with_policy(200_000, Some(&ChunkingPolicy::Adaptive));
        assert_eq!(size, 600);
        unsafe {
            match prev {
                Some(v) => std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", v),
                None => std::env::remove_var("EDGEQUAKE_ADAPTIVE_CHUNKING"),
            }
        }
    }

    #[test]
    fn workspace_fixed_ignores_env_and_doc_size() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var("EDGEQUAKE_ADAPTIVE_CHUNKING").ok();
        unsafe {
            std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", "1");
        }
        let (size, ov) =
            resolve_base_chunk_size_overlap_with_policy(200_000, Some(&ChunkingPolicy::acc_fair()));
        assert_eq!((size, ov), (1200, 100));
        unsafe {
            match prev {
                Some(v) => std::env::set_var("EDGEQUAKE_ADAPTIVE_CHUNKING", v),
                None => std::env::remove_var("EDGEQUAKE_ADAPTIVE_CHUNKING"),
            }
        }
    }

    #[test]
    fn validate_fixed_pair_rejects_overlap_ge_size() {
        assert!(validate_fixed_pair(1200, 100).is_ok());
        assert!(validate_fixed_pair(100, 100).is_err());
        assert!(validate_fixed_pair(0, 0).is_err());
    }

    #[test]
    fn chunking_mode_parse() {
        assert_eq!(ChunkingMode::parse("fixed"), Some(ChunkingMode::Fixed));
        assert_eq!(ChunkingMode::parse("ACC"), Some(ChunkingMode::Fixed));
        assert_eq!(ChunkingMode::parse("inherit"), Some(ChunkingMode::Inherit));
        assert_eq!(ChunkingMode::parse("nope"), None);
    }
}
