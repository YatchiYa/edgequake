//! SPEC-091 W1 — relational chunk writer (single writer site for `chunks` rows).

use edgequake_storage::traits::domain::{
    Chunk, ChunkId, ChunkRepository, DocumentId, TenantId, UnitOfWork, WorkspaceId,
};
use edgequake_storage::StorageError;
use uuid::Uuid;

use crate::pipeline::ProcessingResult;

use super::IngestionPersistContext;

/// Build domain chunks from a processing result (legacy string ids preserved in metadata).
pub fn build_relational_chunks(
    ctx: &IngestionPersistContext,
    result: &ProcessingResult,
) -> Result<Vec<Chunk>, StorageError> {
    let document_id = parse_document_id(&ctx.document_id)?;
    let tenant_id = ctx
        .tenant_id
        .as_deref()
        .map(parse_uuid)
        .transpose()?
        .map(TenantId);
    let workspace_id = ctx
        .workspace_id
        .as_deref()
        .map(parse_uuid)
        .transpose()?
        .map(WorkspaceId);

    Ok(result
        .chunks
        .iter()
        .map(|chunk| {
            // Metadata mirrors `chunk_storage::chunk_kv_value` so relational
            // reads can reconstruct the full legacy chunk shape (SSOT cutover).
            let mut metadata = serde_json::json!({
                "legacy_chunk_key": chunk.id,
                "start_line": chunk.start_line,
                "end_line": chunk.end_line,
            });
            if let Some(file) = ctx.source_file_path.as_deref() {
                metadata["source_file"] = serde_json::json!(file);
            }
            if let Some(section) = &chunk.section {
                metadata["section"] = serde_json::json!({
                    "heading_path": section.heading_path,
                    "heading_level": section.heading_level,
                });
            }
            if let Some(page) = chunk.page_start {
                metadata["page_start"] = serde_json::json!(page);
                metadata["page_end"] = serde_json::json!(chunk.page_end.unwrap_or(page));
            }
            if let Some(modality) = chunk.modality.clone().or_else(|| {
                crate::multimodal::resolve_retrieval_modality_from_content(&chunk.content)
                    .map(str::to_string)
            }) {
                metadata["modality"] = serde_json::json!(modality);
            }
            Chunk {
                id: ChunkId::new(Uuid::new_v4()),
                document_id,
                tenant_id,
                workspace_id,
                chunk_index: i32::try_from(chunk.index).unwrap_or(i32::MAX),
                content: chunk.content.clone(),
                start_offset: i32::try_from(chunk.start_offset).ok(),
                end_offset: i32::try_from(chunk.end_offset).ok(),
                token_count: i32::try_from(chunk.token_count).ok(),
                metadata,
            }
        })
        .collect())
}

/// Insert relational chunks when authority mode and repository are configured.
pub async fn persist_relational_chunks(
    repo: &dyn ChunkRepository,
    ctx: &IngestionPersistContext,
    result: &ProcessingResult,
) -> Result<(), StorageError> {
    let chunks = build_relational_chunks(ctx, result)?;
    if chunks.is_empty() {
        return Ok(());
    }
    repo.insert_batch(&mut UnitOfWork::default(), &chunks)
        .await?;
    Ok(())
}

fn parse_document_id(raw: &str) -> Result<DocumentId, StorageError> {
    parse_uuid(raw).map(DocumentId)
}

fn parse_uuid(raw: &str) -> Result<Uuid, StorageError> {
    Uuid::parse_str(raw)
        .map_err(|e| StorageError::InvalidData(format!("invalid uuid '{raw}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::TextChunk;
    use edgequake_storage::MemoryChunkRepository;
    use std::sync::Arc;

    fn sample_result(doc_uuid: &str) -> ProcessingResult {
        ProcessingResult {
            document_id: doc_uuid.into(),
            chunks: vec![TextChunk {
                id: format!("{doc_uuid}-chunk-0"),
                content: "relational text".into(),
                index: 0,
                start_offset: 0,
                end_offset: 15,
                start_line: 1,
                end_line: 1,
                token_count: 3,
                embedding: None,
                section: None,
                page_start: None,
                page_end: None,
                modality: None,
            }],
            extractions: vec![],
            stats: Default::default(),
            lineage: None,
        }
    }

    #[tokio::test]
    async fn contract_spec091_single_chunk_writer() {
        let doc_id = Uuid::new_v4();
        let ctx = IngestionPersistContext::new(doc_id.to_string(), None, None);
        let repo = Arc::new(MemoryChunkRepository::new());
        persist_relational_chunks(repo.as_ref(), &ctx, &sample_result(&doc_id.to_string()))
            .await
            .expect("relational insert");

        let page = repo.scan_from(None, 10).await.expect("scan");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].content, "relational text");
        assert_eq!(page.items[0].document_id.0, doc_id);
    }

    #[test]
    fn contract_spec091_build_relational_chunks_rejects_bad_document_id() {
        let ctx = IngestionPersistContext::new("not-a-uuid", None, None);
        let err = build_relational_chunks(&ctx, &sample_result("not-a-uuid")).unwrap_err();
        assert!(matches!(err, StorageError::InvalidData(_)));
    }
}
