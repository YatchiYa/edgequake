//! SPEC-091 Migration Console — durable KV residue (fact #5).
//!
//! LAW-C3 (reuse the real guard, verbatim): the predicates here mirror
//! `migrations/125_spec091_kv_drop.sql:57-101` exactly, so the advisor's
//! drop-readiness verdict can never silently diverge from the guard that will
//! actually run. A contract test (`contract_spec091_advisor_matches_125_guard`)
//! asserts the two stay equal on fixture databases.
//!
//! A row counts as durable residue only when it is durable AND not yet
//! represented in its typed SSOT — so already-backfilled (redundant) rows pass,
//! and the count blocks only on genuine data-loss risk.

use sqlx::PgPool;

use super::types::ResidueReport;
use crate::error::StorageError;

/// Canonical-UUID regex — identical to migration 125 (`uuid_re`). Used to
/// extract the document id from legacy shell/lineage/multimodal keys and as the
/// chunk-key document prefix.
const UUID_RE: &str =
    "([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})";

/// Per-category durable-row counts over one KV table. `%TABLE%` / `%UUID_RE%`
/// are substituted with `.replace()` (NOT `format!`) so the regex braces and
/// the `LIKE '%…'` wildcards pass through untouched.
/// Anchored UUID for purge-aware predicates (mirrors migration 125 verified purge).
const UUID_ANCHORED: &str =
    "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";

const RESIDUE_SQL: &str = r#"
SELECT
  count(*) FILTER (WHERE (k.key ~ '^%UUID_RE%-chunk-[0-9]+$'
     AND COALESCE(k.value->>'content', '') <> ''
     AND NOT EXISTS (SELECT 1 FROM public.chunks c
                     WHERE c.document_id = left(k.key, 36)::uuid
                       AND c.chunk_index = substring(k.key from 44)::int))) AS chunk_text,
  count(*) FILTER (WHERE ((k.key LIKE '%-metadata' OR k.key LIKE '%-content')
     AND NOT EXISTS (SELECT 1 FROM public.documents d
                     WHERE d.id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))) AS doc_shells,
  count(*) FILTER (WHERE (k.key LIKE '%-lineage'
     AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a
                     WHERE a.kind = 'lineage'
                       AND a.document_id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))) AS lineage,
  count(*) FILTER (WHERE ((k.key LIKE '%-multimodal-manifest'
     AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a
                     WHERE a.kind = 'multimodal-manifest'
                       AND a.document_id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))
     OR (k.key LIKE '%-multimodal-chunks'
     AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a
                     WHERE a.kind = 'multimodal-chunks'
                       AND a.document_id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid)))) AS multimodal,
  -- LAW-KVH5: purge-aware — only keys that would survive migration 125 verified purge.
  count(*) FILTER (WHERE (k.key LIKE 'doc:hash:%'
     AND NOT (split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.ingestion_dedup d
                          WHERE d.workspace_id = split_part(k.key, ':', 3)::uuid
                            AND d.content_hash = split_part(k.key, ':', 4)
                            AND d.pipeline_version = 'v1')))) AS doc_hash,
  count(*) FILTER (WHERE (k.key LIKE 'staging:hash:%'
     AND NOT (split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.ingestion_dedup d
                          WHERE d.workspace_id = split_part(k.key, ':', 3)::uuid
                            AND d.content_hash = split_part(k.key, ':', 4)
                            AND d.pipeline_version = 'staging')))) AS staging_hash,
  count(*) FILTER (WHERE (k.key LIKE 'wsdoc:%'
     AND NOT (split_part(k.key, ':', 2) ~ '%UUID_ANCHORED%'
              AND split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.documents d
                          WHERE d.id = split_part(k.key, ':', 3)::uuid
                            AND d.workspace_id = split_part(k.key, ':', 2)::uuid)))) AS wsdoc,
  count(*) FILTER (WHERE (k.key LIKE 'injection::%'
     AND NOT (k.key LIKE '%-metadata'
              AND split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND replace(split_part(k.key, ':', 5), '-metadata', '') ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.documents d
                          WHERE d.id = replace(split_part(k.key, ':', 5), '-metadata', '')::uuid
                            AND d.metadata->>'source_type' = 'injection')))) AS injection
FROM public."%TABLE%" k
"#;

/// The 125 OR-predicate as a single count, for the parity contract test. This
/// is the exact `durable` value the migration-125 guard computes for one table.
const GUARD_TOTAL_SQL: &str = r#"
SELECT count(*) FROM public."%TABLE%" k WHERE
  (k.key ~ '^%UUID_RE%-chunk-[0-9]+$'
     AND COALESCE(k.value->>'content', '') <> ''
     AND NOT EXISTS (SELECT 1 FROM public.chunks c
                     WHERE c.document_id = left(k.key, 36)::uuid
                       AND c.chunk_index = substring(k.key from 44)::int))
  OR ((k.key LIKE '%-metadata' OR k.key LIKE '%-content')
     AND NOT EXISTS (SELECT 1 FROM public.documents d
                     WHERE d.id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))
  OR (k.key LIKE '%-lineage'
     AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a
                     WHERE a.kind = 'lineage'
                       AND a.document_id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))
  OR (k.key LIKE '%-multimodal-manifest'
     AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a
                     WHERE a.kind = 'multimodal-manifest'
                       AND a.document_id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))
  OR (k.key LIKE '%-multimodal-chunks'
     AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a
                     WHERE a.kind = 'multimodal-chunks'
                       AND a.document_id = NULLIF(substring(k.key from '%UUID_RE%'), '')::uuid))
  OR (k.key LIKE 'doc:hash:%'
     AND NOT (split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.ingestion_dedup d
                          WHERE d.workspace_id = split_part(k.key, ':', 3)::uuid
                            AND d.content_hash = split_part(k.key, ':', 4)
                            AND d.pipeline_version = 'v1')))
  OR (k.key LIKE 'staging:hash:%'
     AND NOT (split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.ingestion_dedup d
                          WHERE d.workspace_id = split_part(k.key, ':', 3)::uuid
                            AND d.content_hash = split_part(k.key, ':', 4)
                            AND d.pipeline_version = 'staging')))
  OR (k.key LIKE 'wsdoc:%'
     AND NOT (split_part(k.key, ':', 2) ~ '%UUID_ANCHORED%'
              AND split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.documents d
                          WHERE d.id = split_part(k.key, ':', 3)::uuid
                            AND d.workspace_id = split_part(k.key, ':', 2)::uuid)))
  OR (k.key LIKE 'injection::%'
     AND NOT (k.key LIKE '%-metadata'
              AND split_part(k.key, ':', 3) ~ '%UUID_ANCHORED%'
              AND replace(split_part(k.key, ':', 5), '-metadata', '') ~ '%UUID_ANCHORED%'
              AND EXISTS (SELECT 1 FROM public.documents d
                          WHERE d.id = replace(split_part(k.key, ':', 5), '-metadata', '')::uuid
                            AND d.metadata->>'source_type' = 'injection')))
"#;

fn substitute(template: &str, table: &str) -> String {
    template
        .replace("%UUID_RE%", UUID_RE)
        .replace("%UUID_ANCHORED%", UUID_ANCHORED)
        .replace("%TABLE%", table)
}

/// List the remaining generic KV base tables (`public.eq_%_kv`, excluding the
/// `_kv_stats` sidecars) — the same relation set migration 125 iterates.
pub async fn list_kv_tables(pool: &PgPool) -> Result<Vec<String>, StorageError> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname LIKE 'eq\_%\_kv' ESCAPE '\'
          AND c.relname NOT LIKE '%\_kv\_stats' ESCAPE '\'
        ORDER BY c.relname
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("advisor list_kv_tables failed: {e}")))
}

/// Durable residue in one KV table (42P01-tolerant: a dropped table has none).
pub async fn kv_durable_residue(
    pool: &PgPool,
    kv_table: &str,
) -> Result<ResidueReport, StorageError> {
    // Identifier-injection guard: table names cannot be parameterized. Accept
    // only the identifier shape `list_kv_tables` can return.
    if !kv_table
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(StorageError::InvalidQuery(format!(
            "unsafe kv table identifier: {kv_table}"
        )));
    }
    let sql = substitute(RESIDUE_SQL, kv_table);
    let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64, i64)>(&sql)
        .fetch_one(pool)
        .await;
    match row {
        Ok((
            chunk_text,
            doc_shells,
            lineage,
            multimodal,
            doc_hash,
            staging_hash,
            wsdoc,
            injection,
        )) => Ok(ResidueReport {
            chunk_text,
            doc_shells,
            lineage,
            multimodal,
            doc_hash,
            staging_hash,
            wsdoc,
            injection,
        }),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => {
            // Only treat the *KV table itself* as gone. A missing typed SSOT
            // relation (chunks / documents / artifacts) must not report zero
            // residue — that produced a false GREEN drop-readiness on v0.22.0
            // DBs before migrations 106+ created those tables.
            let msg = db.message();
            if msg.contains(kv_table) {
                Ok(ResidueReport::default())
            } else {
                Err(StorageError::Database(format!(
                    "advisor kv_durable_residue failed: typed SSOT missing ({msg}). \
                     Run `edgequake migrate` first (SAFE SCHEMA), then retry guard."
                )))
            }
        }
        Err(e) => Err(StorageError::Database(format!(
            "advisor kv_durable_residue failed: {e}"
        ))),
    }
}

/// Total durable residue across EVERY remaining `eq_*_kv` table (the global
/// drop-readiness signal — migration 125 iterates all of them).
pub async fn kv_durable_residue_all(pool: &PgPool) -> Result<ResidueReport, StorageError> {
    let mut total = ResidueReport::default();
    for table in list_kv_tables(pool).await? {
        total.add(&kv_durable_residue(pool, &table).await?);
    }
    Ok(total)
}

/// The migration-125 guard's `durable` count for one table — exposed for the
/// parity contract test (advisor verdict must equal the guard's verdict).
#[cfg(any(test, feature = "postgres"))]
pub async fn guard_durable_total(pool: &PgPool, kv_table: &str) -> Result<i64, StorageError> {
    if !kv_table
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(StorageError::InvalidQuery(format!(
            "unsafe kv table identifier: {kv_table}"
        )));
    }
    let sql = substitute(GUARD_TOTAL_SQL, kv_table);
    sqlx::query_scalar::<_, i64>(&sql)
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Database(format!("advisor guard_durable_total failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_residue_mirrors_125_predicates() {
        // Every durable predicate in migration 125 must appear in the advisor's
        // residue SQL (drift guard). If 125 changes, this test forces a
        // conscious update here too.
        for needle in [
            "-chunk-[0-9]+$",
            "left(k.key, 36)::uuid",
            "substring(k.key from 44)::int",
            "'%-metadata'",
            "'%-content'",
            "'%-lineage'",
            "'%-multimodal-manifest'",
            "'%-multimodal-chunks'",
            "'doc:hash:%'",
            "'staging:hash:%'",
            "'wsdoc:%'",
            "'injection::%'",
            "public.chunks",
            "public.documents",
            "public.document_artifacts",
            "public.ingestion_dedup",
            "pipeline_version = 'v1'",
            "pipeline_version = 'staging'",
        ] {
            assert!(RESIDUE_SQL.contains(needle), "residue SQL missing {needle}");
            assert!(
                GUARD_TOTAL_SQL.contains(needle),
                "guard SQL missing {needle}"
            );
        }
    }

    #[test]
    fn contract_spec091_substitute_preserves_regex_and_wildcards() {
        let sql = substitute(RESIDUE_SQL, "eq_default_kv");
        assert!(sql.contains("public.\"eq_default_kv\""));
        // The UUID regex braces must survive intact (no format! mangling).
        assert!(sql.contains("{8}-[0-9a-fA-F]{4}"));
        // LIKE wildcards preserved.
        assert!(sql.contains("'%-metadata'"));
        assert!(!sql.contains("%TABLE%"));
        assert!(!sql.contains("%UUID_RE%"));
    }

    #[test]
    fn contract_spec091_identifier_guard() {
        // The substitute path must reject anything but a plain identifier.
        assert!(!"eq_default_kv; DROP TABLE"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'));
        assert!("eq_default_kv"
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'));
    }
}
