//! Mirror live pipeline stage into relational `documents` (SPEC-120 A6).
//!
//! WHY: Document list keyset reads Postgres, not KV. Convert progress written only
//! to KV leaves polls projecting admission Queued while vision is converting.

use serde_json::json;

use crate::handlers::ingestion_types::IngestionProgressCounts;

use super::progress_counts::progress_counts_json;
use super::FenceEpoch;

/// Best-effort sync of status + stage fields into relational `documents`.
///
/// Merges JSONB metadata (`||`) so cost/token keys are preserved. The write is
/// accepted only for the current run and when it does not regress stage/progress.
#[cfg(feature = "postgres")]
#[allow(clippy::too_many_arguments)]
pub async fn mirror_document_stage_to_relational(
    pool: &sqlx::PgPool,
    document_id: &str,
    held_epoch: FenceEpoch,
    expected_track_id: &str,
    status: &str,
    current_stage: &str,
    stage_rank: u16,
    stage_message: Option<&str>,
    stage_progress: Option<f64>,
) -> bool {
    mirror_document_stage_to_relational_with_counts(
        pool,
        document_id,
        held_epoch,
        expected_track_id,
        status,
        current_stage,
        stage_rank,
        stage_message,
        stage_progress,
        None,
    )
    .await
}

/// Same as [`mirror_document_stage_to_relational`] with durable `progress_counts`.
#[cfg(feature = "postgres")]
#[allow(clippy::too_many_arguments)]
pub async fn mirror_document_stage_to_relational_with_counts(
    pool: &sqlx::PgPool,
    document_id: &str,
    held_epoch: FenceEpoch,
    expected_track_id: &str,
    status: &str,
    current_stage: &str,
    stage_rank: u16,
    stage_message: Option<&str>,
    stage_progress: Option<f64>,
    progress_counts: Option<&IngestionProgressCounts>,
) -> bool {
    let Ok(uuid) = uuid::Uuid::parse_str(document_id) else {
        tracing::debug!(
            document_id = %document_id,
            "skip relational stage mirror: non-UUID document id"
        );
        return false;
    };

    let pg_status = if status == "completed" {
        "indexed"
    } else {
        status
    };

    let mut meta = json!({
        "current_stage": current_stage,
        "stage_rank": stage_rank,
        "run_epoch": held_epoch.0,
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });
    if let Some(msg) = stage_message {
        meta["stage_message"] = json!(msg);
    }
    if let Some(progress) = stage_progress {
        meta["stage_progress"] = json!(progress.clamp(0.0, 1.0));
    }
    if let Some(counts) = progress_counts {
        meta["progress_counts"] = progress_counts_json(counts);
    }

    match sqlx::query(
        r#"
        UPDATE public.documents SET
            status = $2,
            metadata = COALESCE(metadata, '{}'::jsonb) || $3::jsonb,
            updated_at = NOW()
        WHERE id = $1
          AND fence_epoch = $4
          AND track_id = $5
          AND (
              $6::int > CASE
                  WHEN jsonb_typeof(metadata->'stage_rank') = 'number'
                      THEN (metadata->>'stage_rank')::int
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) IN
                      ('pending', 'queued', 'cleaning', 'uploading') THEN 10
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) = 'converting' THEN 20
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) IN
                      ('processing', 'preprocessing') THEN 30
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) = 'chunking' THEN 40
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) = 'extracting' THEN 50
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) = 'gleaning' THEN 60
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) = 'merging' THEN 70
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) = 'summarizing' THEN 80
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) IN
                      ('embedding', 're_embedding') THEN 90
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) IN
                      ('storing', 'indexing') THEN 100
                  WHEN lower(COALESCE(metadata->>'current_stage', '')) IN
                      ('completed', 'indexed', 'failed', 'partial_failure',
                       'partial_success', 'cancelled') THEN 110
                  ELSE 0
              END
              OR (
                  (
                      $6::int = CASE
                          WHEN jsonb_typeof(metadata->'stage_rank') = 'number'
                              THEN (metadata->>'stage_rank')::int
                          ELSE 0
                      END
                      OR lower(COALESCE(metadata->>'current_stage', '')) = lower($8)
                  )
                  AND $7::double precision >= COALESCE(
                      CASE WHEN jsonb_typeof(metadata->'stage_progress') = 'number'
                          THEN (metadata->>'stage_progress')::double precision
                      END,
                      0.0
                  )
              )
          )
        "#,
    )
    .bind(uuid)
    .bind(pg_status)
    .bind(meta)
    .bind(held_epoch.0)
    .bind(expected_track_id)
    .bind(i32::from(stage_rank))
    .bind(stage_progress.unwrap_or(0.0).clamp(0.0, 1.0))
    .bind(current_stage)
    .execute(pool)
    .await
    {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::debug!(
                document_id = %document_id,
                status = %pg_status,
                current_stage = %current_stage,
                "Mirrored document stage to relational documents"
            );
            true
        }
        Ok(_) => {
            tracing::debug!(
                document_id = %document_id,
                held_epoch = held_epoch.0,
                expected_track_id,
                "relational stage mirror rejected (stale run or regressive write)"
            );
            edgequake_observability::record_fence_rejected_write("stage_mirror_cas");
            false
        }
        Err(e) => {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "Failed to mirror document stage to relational documents (non-fatal)"
            );
            false
        }
    }
}

#[cfg(feature = "postgres")]
pub async fn mirror_converting_start_to_relational(
    pool: &sqlx::PgPool,
    document_id: &str,
    held_epoch: FenceEpoch,
    expected_track_id: &str,
    stage_message: &str,
) -> bool {
    let Ok(uuid) = uuid::Uuid::parse_str(document_id) else {
        tracing::debug!(
            document_id = %document_id,
            "skip converting-start mirror: non-UUID document id"
        );
        return false;
    };

    // Authoritative stage entry: same fence+track may reset progress (resume /
    // reclaim after a prior 100% converting write must not permanently reject
    // page-level progress). Anti-regression still applies to mid-stage updates
    // via [`mirror_document_stage_to_relational`].
    let rank = edgequake_pipeline::stage_bridge::stage_slug_rank("converting").unwrap_or(20);
    let meta = json!({
        "current_stage": "converting",
        "stage_rank": rank,
        "stage_message": stage_message,
        "stage_progress": 0.0,
        "run_epoch": held_epoch.0,
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });

    match sqlx::query(
        r#"
        UPDATE public.documents SET
            status = 'processing',
            metadata = COALESCE(metadata, '{}'::jsonb) || $2::jsonb,
            updated_at = NOW()
        WHERE id = $1
          AND fence_epoch = $3
          AND track_id = $4
        "#,
    )
    .bind(uuid)
    .bind(meta)
    .bind(held_epoch.0)
    .bind(expected_track_id)
    .execute(pool)
    .await
    {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::debug!(
                document_id = %document_id,
                "Mirrored converting-start to relational documents (progress reset allowed)"
            );
            true
        }
        Ok(_) => {
            tracing::debug!(
                document_id = %document_id,
                held_epoch = held_epoch.0,
                expected_track_id,
                "converting-start mirror rejected (stale run / fence mismatch)"
            );
            edgequake_observability::record_fence_rejected_write("converting_start_cas");
            false
        }
        Err(e) => {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "Failed to mirror converting-start to relational documents (non-fatal)"
            );
            false
        }
    }
}

/// Park a document at capacity-wait in relational metadata (fairness hold SSOT).
///
/// Called when a task is fairness-parked so keyset list polls show capacity wait
/// instead of a stale mid-pipeline stage from a prior attempt.
#[cfg(feature = "postgres")]
pub async fn mirror_capacity_wait_to_relational(
    pool: &sqlx::PgPool,
    document_id: &str,
    expected_track_id: &str,
) -> bool {
    mirror_capacity_wait_to_relational_with_message(
        pool,
        document_id,
        expected_track_id,
        None,
    )
    .await
}

/// Park a document at capacity-wait with an optional named wait message.
#[cfg(feature = "postgres")]
pub async fn mirror_capacity_wait_to_relational_with_message(
    pool: &sqlx::PgPool,
    document_id: &str,
    expected_track_id: &str,
    wait_message: Option<&str>,
) -> bool {
    let Ok(uuid) = uuid::Uuid::parse_str(document_id) else {
        return false;
    };
    let msg = wait_message
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Waiting for a processing slot — ingestion continues automatically");
    // Clear itemized counts while parked — Active Runs must not show stale N/M.
    let meta = json!({
        "current_stage": "queued",
        "stage_rank": 10,
        "stage_message": msg,
        "stage_progress": 0.0,
        "updated_at": chrono::Utc::now().to_rfc3339(),
    });
    match sqlx::query(
        r#"
        UPDATE public.documents SET
            metadata = (COALESCE(metadata, '{}'::jsonb) || $2::jsonb) - 'progress_counts',
            updated_at = NOW()
        WHERE id = $1
          AND track_id = $3
          AND status NOT IN ('indexed', 'completed', 'cancelled', 'failed', 'partial_failure', 'partial_success')
          AND lower(COALESCE(metadata->>'current_stage', '')) IN
              ('', 'queued', 'pending', 'uploading', 'waiting', 'cleaning')
        "#,
    )
    .bind(uuid)
    .bind(meta)
    .bind(expected_track_id)
    .execute(pool)
    .await
    {
        Ok(r) => r.rows_affected() > 0,
        Err(e) => {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "Failed to mirror capacity-wait to relational documents (non-fatal)"
            );
            false
        }
    }
}
