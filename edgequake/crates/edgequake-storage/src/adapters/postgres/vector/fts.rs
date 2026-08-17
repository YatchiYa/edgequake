//! PostgreSQL native FTS for chunk sparse retrieval (SPEC-023 I10).
//!
//! Uses GIN-indexed `content_tsv` + `ts_rank_cd` (cover-density ranking — **not**
//! Okapi BM25; see SPEC-083 X-05) instead of re-scoring vector candidates in
<<<<<<< HEAD
//! application memory. Chunk text SSOT is the shared default KV table
//! (SPEC-024 2.5); workspace vector tables only hold embeddings.
=======
//! application memory.
//!
//! SPEC-105 / LAW-L3: KV LEFT JOIN is **era-aware** — only when
//! `chunk_kv_table_exists` (≤0.22 mid-upgrade). Post-125 census, content comes
//! from `content_tsv` / metadata only (typed chunks SSOT via serving path).
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
//!
//! Text search language is configurable via `EDGEQUAKE_FTS_LANGUAGE`
//! (default `english`).

use sqlx::Row;

use super::PgVectorStorage;
use crate::adapters::postgres::schema;
use crate::error::{Result, StorageError};
use crate::traits::{MetadataFilter, VectorSearchResult};

/// Env key for Postgres text-search configuration name (X-05).
pub const FTS_LANGUAGE_ENV: &str = "EDGEQUAKE_FTS_LANGUAGE";

/// Default Postgres text-search config when env is unset.
pub const DEFAULT_FTS_LANGUAGE: &str = "english";

/// Sanitize a Postgres `regconfig` name: lowercase ASCII letters only.
///
/// Rejects anything that could break out of a SQL string literal.
pub fn sanitize_fts_language(raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.is_empty()
        || !lower.chars().all(|c| c.is_ascii_lowercase() || c == '_')
        || lower.len() > 32
    {
        return DEFAULT_FTS_LANGUAGE.to_string();
    }
    lower
}

/// Resolve FTS language from `EDGEQUAKE_FTS_LANGUAGE` (default `english`).
pub fn fts_language_from_env() -> String {
    match std::env::var(FTS_LANGUAGE_ENV) {
        Ok(v) => sanitize_fts_language(&v),
        Err(_) => DEFAULT_FTS_LANGUAGE.to_string(),
    }
}

fn fts_content_expr(join_kv: bool, lang: &str) -> String {
    if join_kv {
        format!(
            "coalesce(NULLIF(v.content_tsv, ''::tsvector), to_tsvector('{lang}', coalesce(v.metadata->>'content', k.value->>'content', '')))"
        )
    } else {
        format!(
            "coalesce(NULLIF(v.content_tsv, ''::tsvector), to_tsvector('{lang}', coalesce(v.metadata->>'content', '')))"
        )
    }
}

impl PgVectorStorage {
    pub(crate) async fn chunk_kv_table_exists_cached(&self) -> Result<bool> {
        if let Some(exists) = self.chunk_kv_table_exists.get() {
            return Ok(*exists);
        }

        let pool = self.pool.get().await?;
        let exists = schema::relation_exists(&pool, &self.chunk_kv_table_name).await?;
        let _ = self.chunk_kv_table_exists.set(exists);
        Ok(exists)
    }

    /// Full-text search with `ts_rank_cd` over chunk content (cover-density rank).
    pub(crate) async fn postgres_text_search_filtered(
        &self,
        query_text: &str,
        top_k: usize,
        filter_ids: Option<&[String]>,
        metadata_filter: Option<&MetadataFilter>,
    ) -> Result<Vec<VectorSearchResult>> {
        if query_text.trim().is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }

        let lang = fts_language_from_env();
        let pool = self.pool.get().await?;
        let join_kv = self.chunk_kv_table_exists_cached().await?;
        let content_expr = fts_content_expr(join_kv, &lang);
        let mf = metadata_filter.cloned().unwrap_or_default();
        let has_id_filter = filter_ids.map(|ids| !ids.is_empty()).unwrap_or(false);
        let filter_sql = mf.build_sql_with_alias(has_id_filter, 2, Some("v"));

        let mut conditions = vec![format!(
            "{content_expr} @@ websearch_to_tsquery('{lang}', $1)"
        )];
        conditions.extend(filter_sql.conditions);

        let where_clause = format!("WHERE {}", conditions.join(" AND "));

        let kv_join = if join_kv {
            format!(
                "LEFT JOIN {} k ON k.key = coalesce(v.metadata->>'content_ref', v.id)",
                self.chunk_kv_table_name
            )
        } else {
            String::new()
        };

        let sql = format!(
            r#"
            SELECT v.id, v.metadata,
                   ts_rank_cd(
                       {content_expr},
                       websearch_to_tsquery('{lang}', $1)
                   )::float4 AS score
            FROM {vectors} v
            {kv_join}
            {where_clause}
            ORDER BY score DESC
            LIMIT ${limit_param}
            "#,
            content_expr = content_expr,
            lang = lang,
            vectors = self.table_name,
            kv_join = kv_join,
            where_clause = where_clause,
            limit_param = filter_sql.next_param
        );

        use sqlx::postgres::PgArguments;
        use sqlx::Arguments;

        let mut args = PgArguments::default();
        args.add(query_text)
            .map_err(|e| StorageError::Database(format!("Failed to bind FTS query text: {}", e)))?;

        if let Some(ids) = filter_ids {
            if !ids.is_empty() {
                let id_vec: Vec<String> = ids.to_vec();
                args.add(&id_vec).map_err(|e| {
                    StorageError::Database(format!("Failed to bind filter_ids: {}", e))
                })?;
            }
        }

        if let Some(doc_ids) = &mf.document_ids {
            args.add(&doc_ids.clone()).map_err(|e| {
                StorageError::Database(format!("Failed to bind document_ids: {}", e))
            })?;
        }
        if let Some(tid) = &mf.tenant_id {
            args.add(tid)
                .map_err(|e| StorageError::Database(format!("Failed to bind tenant_id: {}", e)))?;
        }
        if let Some(wid) = &mf.workspace_id {
            args.add(wid).map_err(|e| {
                StorageError::Database(format!("Failed to bind workspace_id: {}", e))
            })?;
        }
        if let Some(vtype) = &mf.vector_type {
            args.add(vtype).map_err(|e| {
                StorageError::Database(format!("Failed to bind vector_type: {}", e))
            })?;
        }
        if let Some(modalities) = &mf.modalities {
            let mods: Vec<String> = modalities.clone();
            args.add(&mods)
                .map_err(|e| StorageError::Database(format!("Failed to bind modalities: {}", e)))?;
        }

        args.add(top_k as i32)
            .map_err(|e| StorageError::Database(format!("Failed to bind top_k: {}", e)))?;

        let rows = sqlx::query_with(&sql, args)
            .fetch_all(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("Postgres FTS query failed: {}", e)))?;

        Ok(rows
            .iter()
            .map(|row| VectorSearchResult {
                id: row.get("id"),
                score: row.get::<f32, _>("score"),
                metadata: row.get("metadata"),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e2e_fts_language_config() {
        assert_eq!(sanitize_fts_language("french"), "french");
        assert_eq!(sanitize_fts_language("SIMPLE"), "simple");
        assert_eq!(
            sanitize_fts_language("english'; drop table x"),
            DEFAULT_FTS_LANGUAGE
        );
        assert_eq!(sanitize_fts_language(""), DEFAULT_FTS_LANGUAGE);
        assert_eq!(sanitize_fts_language("fr-FR"), DEFAULT_FTS_LANGUAGE);

        std::env::remove_var(FTS_LANGUAGE_ENV);
        assert_eq!(fts_language_from_env(), DEFAULT_FTS_LANGUAGE);

        std::env::set_var(FTS_LANGUAGE_ENV, "french");
        assert_eq!(fts_language_from_env(), "french");
        let expr = fts_content_expr(false, &fts_language_from_env());
        assert!(
            expr.contains("to_tsvector('french'"),
            "FTS content expr must use configured language: {expr}"
        );
        assert!(
            !expr.contains("BM25"),
            "X-05: postgres FTS path must not claim BM25"
        );
        std::env::remove_var(FTS_LANGUAGE_ENV);
    }
}
