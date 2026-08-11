//! SPEC-120 — DRY absorb upsert for typed fleet embeddings.
//!
//! Single conflict policy for entity / relationship / report (LAW-120-3):
//! 1. stamp-once UPDATE by PK when `legacy_vector_id` is NULL
//! 2. INSERT with targetless `ON CONFLICT DO NOTHING` (absorbs PK + legacy unique)
//! 3. count absorbed lid-bearing rows whose FK was not written
//!
//! Table / FK identifiers come only from [`EmbeddingFamily`] (closed set — SOLID OCP
//! for new families: extend the enum + metadata, not copy another upsert path).

use sqlx::PgPool;
use uuid::Uuid;

use crate::embedding_family::EmbeddingFamily;
use crate::error::StorageError;

/// Columns for one absorb upsert batch.
pub(super) struct AbsorbBatch<'a> {
    pub family: EmbeddingFamily,
    pub model_id: Uuid,
    pub fk_uuids: Option<&'a [Uuid]>,
    pub fk_texts: Option<&'a [String]>,
    pub workspace_ids: &'a [Uuid],
    pub vectors: &'a [String],
    pub dims: &'a [i32],
    pub legacy_ids: &'a [String],
}

/// Returns `(upserted, absorbed_legacy_collisions)`.
pub(super) async fn upsert_with_legacy_absorb(
    pool: &PgPool,
    batch: &AbsorbBatch<'_>,
) -> Result<(u64, u64), StorageError> {
    let stamped = stamp_legacy_once(pool, batch).await?;
    let inserted = insert_absorbing_conflicts(pool, batch).await?;
    let absorbed = count_absorbed_lid_misses(pool, batch).await?;

    if absorbed > 0 {
        tracing::warn!(
            family = ?batch.family,
            absorbed_legacy_collisions = absorbed,
            "SPEC-120: absorbed legacy_vector_id collisions (losing FK skipped)"
        );
    }

    Ok((stamped + inserted, absorbed))
}

async fn stamp_legacy_once(pool: &PgPool, batch: &AbsorbBatch<'_>) -> Result<u64, StorageError> {
    let table = batch.family.typed_table();
    let fk = batch.family.typed_fk_column();
    if batch.family.typed_fk_is_uuid() {
        let ids = require_uuid_fks(batch)?;
        let sql = format!(
            r#"
            UPDATE {table} AS ee
            SET legacy_vector_id = COALESCE(ee.legacy_vector_id, NULLIF(t.lid, ''))
            FROM unnest($2::uuid[], $3::text[]) AS t(e, lid)
            WHERE ee.model_id = $1
              AND ee.{fk} = t.e
              AND ee.legacy_vector_id IS NULL
              AND NULLIF(t.lid, '') IS NOT NULL
            "#
        );
        Ok(sqlx::query(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.legacy_ids)
            .execute(pool)
            .await
            .map_err(StorageError::from)?
            .rows_affected())
    } else {
        let ids = require_text_fks(batch)?;
        let sql = format!(
            r#"
            UPDATE {table} AS ee
            SET legacy_vector_id = COALESCE(ee.legacy_vector_id, NULLIF(t.lid, ''))
            FROM unnest($2::text[], $3::text[]) AS t(e, lid)
            WHERE ee.model_id = $1
              AND ee.{fk} = t.e
              AND ee.legacy_vector_id IS NULL
              AND NULLIF(t.lid, '') IS NOT NULL
            "#
        );
        Ok(sqlx::query(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.legacy_ids)
            .execute(pool)
            .await
            .map_err(StorageError::from)?
            .rows_affected())
    }
}

async fn insert_absorbing_conflicts(
    pool: &PgPool,
    batch: &AbsorbBatch<'_>,
) -> Result<u64, StorageError> {
    let table = batch.family.typed_table();
    let fk = batch.family.typed_fk_column();
    if batch.family.typed_fk_is_uuid() {
        let ids = require_uuid_fks(batch)?;
        let sql = format!(
            r#"
            INSERT INTO {table}
              (model_id, {fk}, workspace_id, embedding, dimensions, legacy_vector_id)
            SELECT $1, e, w, v::halfvec, d, NULLIF(lid, '')
            FROM unnest($2::uuid[], $3::uuid[], $4::text[], $5::int[], $6::text[])
              AS t(e, w, v, d, lid)
            ON CONFLICT DO NOTHING
            "#
        );
        Ok(sqlx::query(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.workspace_ids)
            .bind(batch.vectors)
            .bind(batch.dims)
            .bind(batch.legacy_ids)
            .execute(pool)
            .await
            .map_err(StorageError::from)?
            .rows_affected())
    } else {
        let ids = require_text_fks(batch)?;
        let sql = format!(
            r#"
            INSERT INTO {table}
              (model_id, {fk}, workspace_id, embedding, dimensions, legacy_vector_id)
            SELECT $1, e, w, v::halfvec, d, NULLIF(lid, '')
            FROM unnest($2::text[], $3::uuid[], $4::text[], $5::int[], $6::text[])
              AS t(e, w, v, d, lid)
            ON CONFLICT DO NOTHING
            "#
        );
        Ok(sqlx::query(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.workspace_ids)
            .bind(batch.vectors)
            .bind(batch.dims)
            .bind(batch.legacy_ids)
            .execute(pool)
            .await
            .map_err(StorageError::from)?
            .rows_affected())
    }
}

async fn count_absorbed_lid_misses(
    pool: &PgPool,
    batch: &AbsorbBatch<'_>,
) -> Result<u64, StorageError> {
    let table = batch.family.typed_table();
    let fk = batch.family.typed_fk_column();
    if batch.family.typed_fk_is_uuid() {
        let ids = require_uuid_fks(batch)?;
        let sql = format!(
            r#"
            SELECT COUNT(*)::bigint
            FROM unnest($2::uuid[], $3::text[]) AS t(e, lid)
            WHERE NULLIF(t.lid, '') IS NOT NULL
              AND NOT EXISTS (
                SELECT 1 FROM {table} ee
                WHERE ee.model_id = $1 AND ee.{fk} = t.e
              )
            "#
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.legacy_ids)
            .fetch_one(pool)
            .await
            .map_err(StorageError::from)? as u64)
    } else {
        let ids = require_text_fks(batch)?;
        let sql = format!(
            r#"
            SELECT COUNT(*)::bigint
            FROM unnest($2::text[], $3::text[]) AS t(e, lid)
            WHERE NULLIF(t.lid, '') IS NOT NULL
              AND NOT EXISTS (
                SELECT 1 FROM {table} ee
                WHERE ee.model_id = $1 AND ee.{fk} = t.e
              )
            "#
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.legacy_ids)
            .fetch_one(pool)
            .await
            .map_err(StorageError::from)? as u64)
    }
}

fn require_uuid_fks<'a>(batch: &AbsorbBatch<'a>) -> Result<&'a [Uuid], StorageError> {
    batch.fk_uuids.ok_or_else(|| {
        StorageError::InvalidInput("SPEC-120: uuid FK batch missing for absorb upsert".into())
    })
}

fn require_text_fks<'a>(batch: &AbsorbBatch<'a>) -> Result<&'a [String], StorageError> {
    batch.fk_texts.ok_or_else(|| {
        StorageError::InvalidInput("SPEC-120: text FK batch missing for absorb upsert".into())
    })
}
