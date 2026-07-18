//! SPEC-077 — Opt-in binary_quantize + exact rerank (B2 bake-off helpers).
//!
//! Default OFF. Pure SQL builders for harness / future opt-in — not boot default.

use crate::filter_column_policy::env_flag_true;

/// Default Hamming candidate pool before exact reorder (≫ typical top_k).
pub const DEFAULT_BINARY_CANDIDATE_K: usize = 200;

/// Runtime policy for binary quantize study path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryQuantizePolicy {
    pub enabled: bool,
    pub candidate_k: usize,
}

impl Default for BinaryQuantizePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            candidate_k: DEFAULT_BINARY_CANDIDATE_K,
        }
    }
}

impl BinaryQuantizePolicy {
    /// Load from env — default OFF (no silent flip).
    pub fn from_env() -> Self {
        let enabled = env_flag_true("EDGEQUAKE_BINARY_QUANTIZE");
        let candidate_k = std::env::var("EDGEQUAKE_BINARY_CANDIDATE_K")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_BINARY_CANDIDATE_K)
            .clamp(1, 50_000);
        Self {
            enabled,
            candidate_k,
        }
    }

    pub fn effective_candidate_k(&self, top_k: usize) -> usize {
        if !self.enabled {
            return top_k;
        }
        self.candidate_k.max(top_k).clamp(1, 50_000)
    }
}

/// DDL for expression HNSW on `binary_quantize(embedding)::bit(dim)`.
///
/// Does **not** drop the primary halfvec/vector HNSW — additive study index.
pub fn build_binary_hnsw_index_sql(
    table: &str,
    index_name: &str,
    dim: usize,
    m: u32,
    ef_construction: u32,
) -> String {
    let dim = dim.clamp(1, 64_000);
    let m = m.clamp(2, 64);
    let ef = ef_construction.clamp(4, 1000);
    format!(
        "CREATE INDEX IF NOT EXISTS {index_name} ON {table} \
         USING hnsw ((binary_quantize(embedding)::bit({dim})) bit_hamming_ops) \
         WITH (m = {m}, ef_construction = {ef})"
    )
}

/// Two-stage SELECT: Hamming ANN candidates → exact distance reorder.
///
/// `emb_type` is `vector` or `halfvec`. `where_clause` may be empty or include `WHERE …`.
/// Outer LIMIT bind index is `limit_param` (same arity pattern as exact-reorder helpers).
pub fn build_binary_rerank_select_sql(
    table: &str,
    emb_type: &str,
    dim: usize,
    where_clause: &str,
    limit_param: u32,
    top_k: usize,
    policy: &BinaryQuantizePolicy,
) -> String {
    let dim = dim.clamp(1, 64_000);
    let candidate_k = policy.effective_candidate_k(top_k);
    let where_sql = if where_clause.trim().is_empty() {
        String::new()
    } else if where_clause
        .trim_start()
        .to_ascii_uppercase()
        .starts_with("WHERE")
    {
        format!("\n            {}", where_clause.trim())
    } else {
        format!("\n            WHERE {}", where_clause.trim())
    };

    // pgvector README: binary candidates then ORDER BY original embedding.
    format!(
        r#"
            WITH candidates AS MATERIALIZED (
                SELECT id, metadata, embedding
                FROM {table}{where_sql}
                ORDER BY binary_quantize(embedding)::bit({dim})
                      <~> binary_quantize($1::{emb_type})::bit({dim})
                LIMIT {candidate_k}
            )
            SELECT id, metadata, 1 - (embedding <=> $1::{emb_type}) AS score
            FROM candidates
            ORDER BY embedding <=> $1::{emb_type}
            LIMIT ${limit_param}
            "#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn default_off() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("EDGEQUAKE_BINARY_QUANTIZE");
        assert!(!BinaryQuantizePolicy::from_env().enabled);
    }

    #[test]
    fn index_sql_uses_bit_hamming() {
        let sql = build_binary_hnsw_index_sql("eq_v", "eq_v_bq_idx", 1536, 16, 64);
        assert!(sql.contains("binary_quantize(embedding)::bit(1536)"));
        assert!(sql.contains("bit_hamming_ops"));
        assert!(sql.contains("USING hnsw"));
    }

    #[test]
    fn query_sql_two_stage() {
        let p = BinaryQuantizePolicy {
            enabled: true,
            candidate_k: 200,
        };
        let sql = build_binary_rerank_select_sql(
            "eq_v",
            "halfvec",
            64,
            "WHERE workspace_id = $2",
            3,
            20,
            &p,
        );
        assert!(sql.contains("<~>"));
        assert!(sql.contains("LIMIT 200"));
        assert!(sql.contains("ORDER BY embedding <=> $1::halfvec"));
        assert!(sql.contains("workspace_id = $2"));
    }
}
