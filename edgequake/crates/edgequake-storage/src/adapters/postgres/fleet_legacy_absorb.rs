//! SPEC-120 / SPEC-136 — DRY absorb upsert for typed fleet embeddings.
//!
//! Single conflict policy for entity / relationship / report (LAW-120-3, LAW-136-1):
//! 1. stamp-once UPDATE by PK when `legacy_vector_id` is NULL **and** the lid is
//!    not already owned in the workspace (UPDATE has no `ON CONFLICT`; 23505 is
//!    absorbed — durable #377 / NULL-lid loser PK)
//! 2. INSERT with targetless `ON CONFLICT DO NOTHING` (absorbs PK + legacy unique)
//! 3. count lid-bearing FKs that do not own the lid while another row does
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

/// Stamp NULL lids only when this workspace does not already own the lid.
///
/// Postgres UPDATE cannot `ON CONFLICT`. A losing FK with a pre-existing
/// NULL-lid PK would otherwise 23505 against `idx_*_legacy_vector_id` (#377).
/// `NOT EXISTS` is the happy path; 23505 is absorbed for the concurrent race.
fn stamp_owned_lid_predicate(table: &str) -> String {
    format!(
        r#"
              AND NOT EXISTS (
                SELECT 1 FROM {table} owned
                WHERE owned.workspace_id = t.w
                  AND owned.legacy_vector_id = NULLIF(t.lid, '')
              )
        "#
    )
}

fn is_legacy_unique_violation(err: &sqlx::Error) -> bool {
    let sqlx::Error::Database(db) = err else {
        return false;
    };
    if db.code().as_deref() != Some("23505") {
        return false;
    }
    let constraint = db.constraint().unwrap_or("");
    constraint.contains("legacy_vector_id") || db.message().contains("legacy_vector_id")
}

fn map_stamp_result(
    result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
) -> Result<u64, StorageError> {
    match result {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) if is_legacy_unique_violation(&e) => {
            tracing::warn!(
                error = %e,
                "SPEC-136: absorbed stamp-once legacy_vector_id unique violation"
            );
            Ok(0)
        }
        Err(e) => Err(StorageError::from(e)),
    }
}

async fn stamp_legacy_once(pool: &PgPool, batch: &AbsorbBatch<'_>) -> Result<u64, StorageError> {
    let table = batch.family.typed_table();
    let fk = batch.family.typed_fk_column();
    let skip_owned = stamp_owned_lid_predicate(table);
    if batch.family.typed_fk_is_uuid() {
        let ids = require_uuid_fks(batch)?;
        let sql = format!(
            r#"
            UPDATE {table} AS ee
            SET legacy_vector_id = COALESCE(ee.legacy_vector_id, NULLIF(t.lid, ''))
            FROM unnest($2::uuid[], $3::uuid[], $4::text[]) AS t(e, w, lid)
            WHERE ee.model_id = $1
              AND ee.{fk} = t.e
              AND ee.workspace_id = t.w
              AND ee.legacy_vector_id IS NULL
              AND NULLIF(t.lid, '') IS NOT NULL
              {skip_owned}
            "#
        );
        map_stamp_result(
            sqlx::query(&sql)
                .bind(batch.model_id)
                .bind(ids)
                .bind(batch.workspace_ids)
                .bind(batch.legacy_ids)
                .execute(pool)
                .await,
        )
    } else {
        let ids = require_text_fks(batch)?;
        let sql = format!(
            r#"
            UPDATE {table} AS ee
            SET legacy_vector_id = COALESCE(ee.legacy_vector_id, NULLIF(t.lid, ''))
            FROM unnest($2::text[], $3::uuid[], $4::text[]) AS t(e, w, lid)
            WHERE ee.model_id = $1
              AND ee.{fk} = t.e
              AND ee.workspace_id = t.w
              AND ee.legacy_vector_id IS NULL
              AND NULLIF(t.lid, '') IS NOT NULL
              {skip_owned}
            "#
        );
        map_stamp_result(
            sqlx::query(&sql)
                .bind(batch.model_id)
                .bind(ids)
                .bind(batch.workspace_ids)
                .bind(batch.legacy_ids)
                .execute(pool)
                .await,
        )
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

/// Lid-bearing FKs that do not own the lid while another workspace row does.
///
/// Covers INSERT-skip (no PK) **and** stamp-skip (NULL-lid PK left in place).
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
            FROM unnest($2::uuid[], $3::uuid[], $4::text[]) AS t(e, w, lid)
            WHERE NULLIF(t.lid, '') IS NOT NULL
              AND NOT EXISTS (
                SELECT 1 FROM {table} ee
                WHERE ee.model_id = $1
                  AND ee.{fk} = t.e
                  AND ee.legacy_vector_id = NULLIF(t.lid, '')
              )
              AND EXISTS (
                SELECT 1 FROM {table} owned
                WHERE owned.workspace_id = t.w
                  AND owned.legacy_vector_id = NULLIF(t.lid, '')
              )
            "#
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.workspace_ids)
            .bind(batch.legacy_ids)
            .fetch_one(pool)
            .await
            .map_err(StorageError::from)? as u64)
    } else {
        let ids = require_text_fks(batch)?;
        let sql = format!(
            r#"
            SELECT COUNT(*)::bigint
            FROM unnest($2::text[], $3::uuid[], $4::text[]) AS t(e, w, lid)
            WHERE NULLIF(t.lid, '') IS NOT NULL
              AND NOT EXISTS (
                SELECT 1 FROM {table} ee
                WHERE ee.model_id = $1
                  AND ee.{fk} = t.e
                  AND ee.legacy_vector_id = NULLIF(t.lid, '')
              )
              AND EXISTS (
                SELECT 1 FROM {table} owned
                WHERE owned.workspace_id = t.w
                  AND owned.legacy_vector_id = NULLIF(t.lid, '')
              )
            "#
        );
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .bind(batch.model_id)
            .bind(ids)
            .bind(batch.workspace_ids)
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
