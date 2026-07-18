//! Dimension migration helpers for [`super::PgVectorStorage`].

use super::PgVectorStorage;
use crate::error::{Result, StorageError};

impl PgVectorStorage {
    /// Get the dimension of the vector column in the database table.
    pub async fn get_stored_dimension(&self) -> Result<Option<usize>> {
        let pool = match self.pool.get().await {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };

        let (schema, table) = if self.table_name.contains('.') {
            let parts: Vec<&str> = self.table_name.split('.').collect();
            (parts[0], parts[1])
        } else {
            ("public", self.table_name.as_str())
        };

        let sql = r#"
            SELECT a.atttypmod
            FROM pg_attribute a
            JOIN pg_class c ON a.attrelid = c.oid
            JOIN pg_namespace n ON c.relnamespace = n.oid
            WHERE n.nspname = $1
              AND c.relname = $2
              AND a.attname = 'embedding'
              AND a.atttypmod > 0
        "#;

        let result: Option<(i32,)> = sqlx::query_as(sql)
            .bind(schema)
            .bind(table)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                StorageError::Database(format!("Failed to get column dimension: {}", e))
            })?;

        match result {
            Some((dim,)) if dim > 0 => {
                tracing::debug!(
                    table = %self.table_name,
                    dimension = dim,
                    "Got column dimension from pg_attribute.atttypmod"
                );
                Ok(Some(dim as usize))
            }
            _ => {
                let fallback_sql = format!(
                    "SELECT vector_dims(embedding) as dim FROM {} LIMIT 1",
                    self.table_name
                );

                let fallback_result: Option<(i32,)> = sqlx::query_as(&fallback_sql)
                    .fetch_optional(&pool)
                    .await
                    .ok()
                    .flatten();

                match fallback_result {
                    Some((dim,)) if dim > 0 => {
                        tracing::debug!(
                            table = %self.table_name,
                            dimension = dim,
                            "Got dimension from stored vector (fallback)"
                        );
                        Ok(Some(dim as usize))
                    }
                    _ => Ok(None),
                }
            }
        }
    }

    /// Ensure the table has the correct dimension.
    ///
    /// SPEC-058: dimension mismatch **fails closed** by default (no silent
    /// `DROP TABLE`). Set `EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD=1` to restore
    /// the destructive recreate path (operator-driven re-embed / new workspace).
    pub async fn ensure_dimension(&self, required_dimension: usize) -> Result<bool> {
        self.pool.initialize().await?;

        let stored_dim = self.get_stored_dimension().await?;

        match stored_dim {
            Some(dim) if dim == required_dimension => {
                tracing::debug!(
                    table = %self.table_name,
                    dimension = required_dimension,
                    "Vector table dimension matches, no recreation needed"
                );
                Ok(false)
            }
            Some(dim) => {
                if !allow_vector_table_rebuild() {
                    crate::compensation::record_vector_dim_mismatch_rejected();
                    return Err(StorageError::InvalidQuery(format!(
                        "Vector dimension mismatch on {}: stored={dim}, required={required_dimension}. \
                         Refusing DROP TABLE (SPEC-058). Re-embed into a new workspace, or set \
                         EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD=1 to wipe and recreate.",
                        self.table_name
                    )));
                }

                tracing::warn!(
                    table = %self.table_name,
                    old_dimension = dim,
                    new_dimension = required_dimension,
                    "Vector dimension mismatch — EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD=1, recreating table"
                );

                self.drop_table().await?;
                self.create_table().await?;

                tracing::info!(
                    table = %self.table_name,
                    dimension = required_dimension,
                    "Vector table recreated with new dimension"
                );

                Ok(true)
            }
            None => {
                tracing::debug!(
                    table = %self.table_name,
                    dimension = required_dimension,
                    "Vector table empty or not exists, will create on initialize"
                );
                Ok(false)
            }
        }
    }
}

/// Opt-in destructive recreate on embedding dimension mismatch (SPEC-058).
pub fn allow_vector_table_rebuild() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_ALLOW_VECTOR_TABLE_REBUILD")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}
