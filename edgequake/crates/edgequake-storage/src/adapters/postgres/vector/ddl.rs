//! DDL and schema maintenance for [`super::PgVectorStorage`].

use super::super::capabilities::{AnnIndexPolicy, HNSW_MAX_DIM_HALFVEC};
use super::super::config::VectorIndexType;
use super::super::row_count_stats::{self, RowCountStatsConfig};
use super::super::schema;
use super::PgVectorStorage;
use crate::error::{Result, StorageError};

impl PgVectorStorage {
    /// Create the vectors table and indexes.
    pub(crate) async fn create_table(&self) -> Result<()> {
        let pool = self.pool.get().await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await
            .map_err(|e| {
                StorageError::Database(format!("Failed to create vector extension: {}", e))
            })?;

        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        let emb_type = self.embedding_pg_type();
        let opclass = self.embedding_opclass();
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                embedding {}({}) NOT NULL,
                metadata JSONB DEFAULT '{{}}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            self.table_name, emb_type, self.dimension
        );

        sqlx::query(&sql).execute(&pool).await.map_err(|e| {
            StorageError::Database(format!("Failed to create vectors table: {}", e))
        })?;

        let index_sql = match self.index_type {
            VectorIndexType::IVFFlat if policy.hnsw_viable => format!(
                "CREATE INDEX IF NOT EXISTS eq_{}_vectors_embedding_idx ON {} USING ivfflat (embedding {}) WITH (lists = {})",
                self.prefix, self.table_name, opclass, self.ivfflat_lists
            ),
            VectorIndexType::HNSW if policy.hnsw_viable => format!(
                "CREATE INDEX IF NOT EXISTS eq_{}_vectors_embedding_idx ON {} USING hnsw (embedding {}) WITH (m = {}, ef_construction = {})",
                self.prefix, self.table_name, opclass, self.hnsw_m, self.hnsw_ef_construction
            ),
            VectorIndexType::IVFFlat | VectorIndexType::HNSW => {
                tracing::warn!(
                    table = %self.table_name,
                    dimension = self.dimension,
                    max_hnsw_dim = HNSW_MAX_DIM_HALFVEC,
                    "Skipping ANN index — embedding dimension exceeds pgvector HNSW limits (issue #275)"
                );
                String::new()
            }
            VectorIndexType::None => String::new(),
        };

        if !index_sql.is_empty() {
            // SPEC-046 OPS-P0.3: fail-closed — never swallow ANN index DDL errors.
            // Missing HNSW silently degrades to seq-scan (latency/recall cliff).
            sqlx::query(&index_sql).execute(&pool).await.map_err(|e| {
                StorageError::Database(format!(
                    "Failed to create ANN index on {}: {}",
                    self.table_name, e
                ))
            })?;
        }

        // SPEC-034 IMP-08: Vector metadata GIN index removed.
        // WHY: 0 query scans — all metadata lookups use metadata->>'key' = value
        // (equality on extracted text), served by doc_id_idx / tenant_ws_idx btrees.
        // This was 13 MB per workspace with zero benefit.
        // To restore: uncomment the line below.
        // sqlx::query(&format!(
        //     "CREATE INDEX IF NOT EXISTS eq_{}_vectors_metadata_idx ON {} USING GIN (metadata jsonb_path_ops)",
        //     self.prefix, self.table_name
        // )).execute(&pool).await.ok();

        let add_cols = format!(
            r#"
            ALTER TABLE {} ADD COLUMN IF NOT EXISTS document_id TEXT;
            ALTER TABLE {} ADD COLUMN IF NOT EXISTS tenant_id TEXT;
            ALTER TABLE {} ADD COLUMN IF NOT EXISTS workspace_id TEXT
            "#,
            self.table_name, self.table_name, self.table_name
        );
        for stmt in add_cols.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(stmt.trim()).execute(&pool).await.ok();
        }

        let doc_idx = format!(
            "CREATE INDEX IF NOT EXISTS eq_{}_vectors_doc_id_idx ON {} (document_id) WHERE document_id IS NOT NULL",
            self.prefix, self.table_name
        );
        sqlx::query(&doc_idx).execute(&pool).await.ok();

        let tenant_idx = format!(
            "CREATE INDEX IF NOT EXISTS eq_{}_vectors_tenant_ws_idx ON {} (tenant_id, workspace_id) WHERE tenant_id IS NOT NULL",
            self.prefix, self.table_name
        );
        sqlx::query(&tenant_idx).execute(&pool).await.ok();

        self.ensure_content_fts(&pool).await?;

        self.ensure_row_count_stats(&pool).await?;

        Ok(())
    }

    pub(crate) async fn ensure_row_count_stats(&self, pool: &sqlx::PgPool) -> Result<()> {
        row_count_stats::ensure_row_count_stats(
            pool,
            &RowCountStatsConfig {
                prefix: &self.prefix,
                table_name: &self.table_name,
                stats_table_name: &self.stats_table_name,
                kind: "vectors",
            },
        )
        .await
    }

    /// Drop the vectors table if it exists.
    pub async fn drop_table(&self) -> Result<()> {
        let pool = self.pool.get().await?;

        let sql = format!("DROP TABLE IF EXISTS {} CASCADE", self.table_name);

        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("Failed to drop vectors table: {}", e)))?;

        tracing::info!(
            table = %self.table_name,
            "Dropped vector table for dimension migration"
        );

        Ok(())
    }

    /// Check if the table exists in the database.
    pub async fn table_exists(&self) -> Result<bool> {
        let pool = match self.pool.get().await {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };

        schema::relation_exists(&pool, &self.table_name).await
    }

    /// Name of the ANN embedding index for this table (SPEC-046 OPS-P0.3).
    pub fn ann_index_name(&self) -> String {
        format!("eq_{}_vectors_embedding_idx", self.prefix)
    }

    /// True when HNSW/IVFFlat index exists (fail-closed readiness probe).
    pub async fn ann_index_exists(&self) -> Result<bool> {
        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        if !policy.hnsw_viable || matches!(self.index_type, VectorIndexType::None) {
            return Ok(true); // ANN not expected — not a readiness failure
        }
        let pool = self.pool.get().await?;
        let index_name = self.ann_index_name();
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM pg_indexes
                WHERE schemaname = 'public' AND indexname = $1
            )",
        )
        .bind(&index_name)
        .fetch_one(&pool)
        .await
        .map_err(|e| StorageError::Database(format!("ann_index_exists probe failed: {e}")))?;
        Ok(exists)
    }

    /// Count vector tables that have no HNSW/IVFFlat index (bootstrap readiness).
    pub async fn count_vector_tables_missing_ann_index(pool: &sqlx::PgPool) -> Result<usize> {
        let missing: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM pg_tables t
            WHERE t.schemaname = 'public'
              AND t.tablename LIKE 'eq\_%\_vectors' ESCAPE '\'
              AND t.tablename NOT LIKE '%\_stats'
              AND NOT EXISTS (
                SELECT 1
                FROM pg_indexes i
                WHERE i.schemaname = 'public'
                  AND i.tablename = t.tablename
                  AND (
                    i.indexname = (t.tablename || '_embedding_idx')
                    OR i.indexdef ILIKE '% USING hnsw %'
                    OR i.indexdef ILIKE '% USING ivfflat %'
                  )
              )
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Database(format!("missing ANN index scan failed: {e}")))?;
        Ok(missing.max(0) as usize)
    }

    /// Add GIN-backed `content_tsv` for native Postgres FTS on chunk content (SPEC-023 I10).
    pub(crate) async fn ensure_content_fts(&self, pool: &sqlx::PgPool) -> Result<()> {
        let table_only = self
            .table_name
            .split('.')
            .next_back()
            .unwrap_or(&self.table_name);

        let add_col = format!(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = '{table_only}'
                      AND column_name = 'content_tsv'
                ) THEN
                    ALTER TABLE {table}
                    ADD COLUMN content_tsv TSVECTOR
                    GENERATED ALWAYS AS (
                        to_tsvector('english', coalesce(metadata->>'content', ''))
                    ) STORED;
                END IF;
            END $$;
            "#,
            table_only = table_only,
            table = self.table_name
        );

        sqlx::query(&add_col).execute(pool).await.ok();

        let fts_idx = format!(
            "CREATE INDEX IF NOT EXISTS eq_{}_vectors_content_tsv_idx ON {} USING GIN (content_tsv)",
            self.prefix, self.table_name
        );
        sqlx::query(&fts_idx).execute(pool).await.ok();

        Ok(())
    }
}
