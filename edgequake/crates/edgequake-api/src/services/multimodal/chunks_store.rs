//! Persist structured mm chunks for pipeline modality-relation injection.

use edgequake_storage::traits::KVStorage;

use super::chunks::MultimodalChunk;

pub fn mm_chunks_key(document_id: &str) -> String {
    format!("{document_id}-multimodal-chunks")
}

pub async fn persist_mm_chunks(
    kv: &dyn KVStorage,
    document_id: &str,
    chunks: &[MultimodalChunk],
) -> Result<(), String> {
    let key = mm_chunks_key(document_id);
    let value = serde_json::to_value(chunks).map_err(|e| e.to_string())?;
<<<<<<< HEAD
    kv.upsert(&[(key, value)]).await.map_err(|e| e.to_string())
}

pub async fn load_mm_chunks(kv: &dyn KVStorage, document_id: &str) -> Option<Vec<MultimodalChunk>> {
    let key = mm_chunks_key(document_id);
    let value = kv.get_by_id(&key).await.ok()??;
    serde_json::from_value(value).ok()
=======
    kv.upsert(&[(key, value.clone())])
        .await
        .map_err(|e| e.to_string())?;
    // SPEC-091 Wave B5: typed artifact dual-write (warn-only).
    crate::services::relational_sidecar_store::typed_artifact_put(
        document_id,
        crate::services::relational_sidecar_store::ARTIFACT_KIND_MM_CHUNKS,
        &value,
    )
    .await;
    Ok(())
}

pub async fn load_mm_chunks(kv: &dyn KVStorage, document_id: &str) -> Option<Vec<MultimodalChunk>> {
    let value = if crate::services::relational_sidecar_store::artifacts_prefer_relational() {
        match crate::services::relational_sidecar_store::typed_artifact_get(
            document_id,
            crate::services::relational_sidecar_store::ARTIFACT_KIND_MM_CHUNKS,
        )
        .await
        {
            Some(v) => Some(v),
            None => kv.get_by_id(&mm_chunks_key(document_id)).await.ok()?,
        }
    } else {
        kv.get_by_id(&mm_chunks_key(document_id)).await.ok()?
    };
    serde_json::from_value(value?).ok()
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}
