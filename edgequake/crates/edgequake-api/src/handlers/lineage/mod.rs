//! Lineage tracking API handlers (Phase 5).
//!
//! Provides endpoints for querying document lineage, including
//! entity provenance and extraction history.
//!
//! ## Implements
//!
//! - **FEAT0540**: Chunk detail retrieval with source tracking
//! - **FEAT0541**: Entity provenance showing extraction origin
//! - **FEAT0542**: Document lineage with graph relationships
//! - **FEAT0543**: Extraction statistics per document
//!
//! ## Use Cases
//!
//! - **UC2140**: User views chunk detail with source document info
//! - **UC2141**: User traces entity back to source document and line
//! - **UC2142**: User explores document's contribution to knowledge graph
//! - **UC2143**: User reviews extraction quality metrics
//!
//! ## Enforces
//!
//! - **BR0540**: Chunk IDs must be valid UUIDs
//! - **BR0541**: Lineage queries must respect workspace isolation
//! - **BR0542**: Extraction metadata must include version info

use axum::{
    extract::{Path, State},
    Json,
};

use crate::error::{ApiError, ApiResult};
use crate::handlers::isolation::{properties_match_tenant_context, verify_document_access};
use crate::middleware::TenantContext;
use crate::state::AppState;

// Re-export DTOs for backward compatibility
pub use crate::handlers::lineage_types::{
    CharRange, ChunkDetailResponse, ChunkLineageResponse, ChunkSourceInfo,
    DescriptionVersionResponse, DocumentGraphLineageResponse, EntityLineageResponse,
    EntityProvenanceResponse, EntitySourceInfo, EntitySummaryResponse, ExtractedEntityInfo,
    ExtractedRelationshipInfo, ExtractionMetadataInfo, ExtractionStatsResponse, LineRangeInfo,
    RelatedEntityInfo, RelationshipSummaryResponse, SourceDocumentInfo,
};

// ============================================================================
// Lineage Response Cache (OODA-23)
// ============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// WHY: Lineage data rarely changes after document processing completes.
/// Caching avoids repeated KV lookups for the same document, providing
/// sub-millisecond response times for dashboard and UI polling scenarios.
/// TTL of 120s balances freshness vs. performance (T1: P95 < 200ms).
const LINEAGE_CACHE_TTL: Duration = Duration::from_secs(120);

/// Maximum entries before evicting oldest. Prevents unbounded memory growth.
const LINEAGE_CACHE_MAX_ENTRIES: usize = 500;

#[derive(Clone)]
struct CachedLineage {
    data: serde_json::Value,
    cached_at: Instant,
}

type LineageCache = Arc<RwLock<HashMap<String, CachedLineage>>>;

lazy_static::lazy_static! {
    static ref LINEAGE_KV_CACHE: LineageCache = Arc::new(RwLock::new(HashMap::new()));
}

/// Read from lineage cache or fetch from KV storage.
///
/// WHY: Lineage queries hit KV storage on every request. After a document is
/// processed, the lineage data is immutable until reprocessing. Caching the
/// result avoids redundant I/O and meets the T1 latency target (<200ms P95).
async fn cached_kv_get(
    kv: &dyn edgequake_storage::traits::KVStorage,
    key: &str,
) -> Result<Option<serde_json::Value>, ApiError> {
    // Check cache first
    {
        let cache = LINEAGE_KV_CACHE.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.cached_at.elapsed() < LINEAGE_CACHE_TTL {
                return Ok(Some(entry.data.clone()));
            }
        }
    }

    // Cache miss — fetch from storage
    let value = kv.get_by_id(key).await?;

    // Populate cache on hit
    if let Some(ref v) = value {
        let mut cache = LINEAGE_KV_CACHE.write().await;
        // WHY: Evict oldest entries when cache is full to prevent unbounded growth
        if cache.len() >= LINEAGE_CACHE_MAX_ENTRIES {
            // Simple eviction: remove entries older than TTL first
            cache.retain(|_, entry| entry.cached_at.elapsed() < LINEAGE_CACHE_TTL);
            // If still too full, clear half the cache
            if cache.len() >= LINEAGE_CACHE_MAX_ENTRIES {
                let keys_to_remove: Vec<String> =
                    cache.keys().take(cache.len() / 2).cloned().collect();
                for k in keys_to_remove {
                    cache.remove(&k);
                }
            }
        }
        cache.insert(
            key.to_string(),
            CachedLineage {
                data: v.clone(),
                cached_at: Instant::now(),
            },
        );
    }

    Ok(value)
}

/// Invalidate a lineage cache entry.
///
/// WHY: Called after document reprocessing to ensure fresh data is served.
/// Without invalidation, stale lineage data would persist until TTL expires.
#[allow(dead_code)]
pub async fn invalidate_lineage_cache(document_id: &str) {
    let mut cache = LINEAGE_KV_CACHE.write().await;
    let lineage_key = format!("{}-lineage", document_id);
    let metadata_key = format!("{}-metadata", document_id);
    cache.remove(&lineage_key);
    cache.remove(&metadata_key);
    tracing::debug!(
        document_id = %document_id,
        "Invalidated lineage cache entries"
    );
}

// ============================================================================
// Chunk Detail Endpoint (WebUI Spec WEBUI-006)
// ============================================================================

/// Get chunk detail.
#[utoipa::path(
    get,
    path = "/api/v1/chunks/{chunk_id}",
    tag = "Lineage",
    params(
        ("chunk_id" = String, Path, description = "Chunk ID to query")
    ),
    responses(
        (status = 200, description = "Chunk detail", body = ChunkDetailResponse),
        (status = 404, description = "Chunk not found")
    )
)]
pub async fn get_chunk_detail(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Path(chunk_id): Path<String>,
) -> ApiResult<Json<ChunkDetailResponse>> {
    // Look up chunk in KV storage
    let chunk_data = state
        .kv_storage
        .get_by_id(&chunk_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Chunk '{}' not found", chunk_id)))?;

    // Parse chunk data
    let content = chunk_data
        .get("content")
        .and_then(|v: &serde_json::Value| v.as_str())
        .unwrap_or("")
        .to_string();

    // OODA-07: Read index field (stored as "index" by OODA-05, fallback to "chunk_index" for legacy)
    let chunk_index = chunk_data
        .get("index")
        .or_else(|| chunk_data.get("chunk_index"))
        .and_then(|v: &serde_json::Value| v.as_u64())
        .unwrap_or(0) as usize;

    let token_count = chunk_data
        .get("token_count")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .unwrap_or(0) as usize;

    let start_offset = chunk_data
        .get("start_offset")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .unwrap_or(0) as usize;

    let end_offset = chunk_data
        .get("end_offset")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .unwrap_or(0) as usize;

    // OODA-07: Read line numbers from chunk KV data (stored by OODA-05)
    let start_line = chunk_data
        .get("start_line")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .map(|v| v as usize);

    let end_line = chunk_data
        .get("end_line")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .map(|v| v as usize);

    // WHY: Chunk IDs follow a deterministic format "{document_id}-chunk-{N}".
    // Extracting the document ID from this format avoids an extra KV lookup
    // and maintains the F8 bidirectional chain (Document ↔ Chunk).
    let document_id = if chunk_id.contains("-chunk-") {
        chunk_id
            .split("-chunk-")
            .next()
            .unwrap_or(&chunk_id)
            .to_string()
    } else {
        chunk_id.clone()
    };

    // SECURITY: Verify the parent document belongs to the requesting tenant/workspace.
    // Returns 404 (not 403) to avoid leaking cross-tenant document IDs.
    let doc_metadata =
        verify_document_access(state.kv_storage.as_ref(), &document_id, &tenant_ctx).await?;

    // Get document name from already-fetched metadata
    let doc_name = doc_metadata
        .get("title")
        .and_then(|v: &serde_json::Value| v.as_str())
        .map(|s| s.to_string());

    // Find entities extracted from this chunk
    let all_nodes = state.graph_storage.get_all_nodes().await?;
    let mut entities: Vec<ExtractedEntityInfo> = Vec::new();

    for node in &all_nodes {
        if let Some(source_id) = node.properties.get("source_id").and_then(|v| v.as_str()) {
            if source_id.contains(&chunk_id) {
                let entity_type = node
                    .properties
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let description = node
                    .properties
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                entities.push(ExtractedEntityInfo {
                    id: node.id.clone(),
                    name: node.id.clone(),
                    entity_type,
                    description,
                });
            }
        }
    }

    // Find relationships from this chunk
    let all_edges = state.graph_storage.get_all_edges().await?;
    let mut relationships: Vec<ExtractedRelationshipInfo> = Vec::new();

    for edge in all_edges {
        if let Some(source_id) = edge.properties.get("source_id").and_then(|v| v.as_str()) {
            if source_id.contains(&chunk_id) {
                let relation_type = edge
                    .properties
                    .get("keywords")
                    .and_then(|v| v.as_str())
                    .unwrap_or("related_to")
                    .to_string();
                let description = edge
                    .properties
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                relationships.push(ExtractedRelationshipInfo {
                    source_name: edge.source.clone(),
                    target_name: edge.target.clone(),
                    relation_type,
                    description,
                });
            }
        }
    }

    Ok(Json(ChunkDetailResponse {
        chunk_id,
        document_id,
        document_name: doc_name,
        content,
        index: chunk_index,
        char_range: CharRange {
            start: start_offset,
            end: end_offset,
        },
        start_line,
        end_line,
        token_count,
        entities,
        relationships,
        extraction_metadata: None, // Would need to be stored during extraction
    }))
}

/// Get entity provenance.
#[utoipa::path(
    get,
    path = "/api/v1/entities/{entity_id}/provenance",
    tag = "Lineage",
    params(
        ("entity_id" = String, Path, description = "Entity ID to query")
    ),
    responses(
        (status = 200, description = "Entity provenance", body = EntityProvenanceResponse),
        (status = 404, description = "Entity not found")
    )
)]
pub async fn get_entity_provenance(
    State(state): State<AppState>,
    tenant_ctx: TenantContext,
    Path(entity_id): Path<String>,
) -> ApiResult<Json<EntityProvenanceResponse>> {
    // WHY: Entity names are normalized to UPPERCASE_WITH_UNDERSCORES during
    // extraction (see entity_extraction.rs). We must apply the same normalization
    // here so lookups match stored graph nodes regardless of user input casing.
    let normalized_id = entity_id.to_uppercase().replace(' ', "_");

    // Look up entity
    let node = state
        .graph_storage
        .get_node(&normalized_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "Entity '{}' not found (normalized: '{}'). \
                 Entity names are stored as UPPERCASE_WITH_UNDERSCORES.",
                entity_id, normalized_id
            ))
        })?;

    // SECURITY: Verify the entity belongs to the requesting tenant/workspace.
    // Returns 404 (not 403) to avoid leaking cross-tenant entity names.
    if !properties_match_tenant_context(&node.properties, &tenant_ctx) {
        return Err(ApiError::NotFound(format!(
            "Entity '{}' not found",
            entity_id
        )));
    }

    let entity_type = node
        .properties
        .get("entity_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let description = node
        .properties
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse source_id to find all source documents
    let source_id = node
        .properties
        .get("source_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let sources: Vec<String> = source_id.split('|').map(|s| s.to_string()).collect();
    let sources_count = sources.len();
    let mut doc_map: std::collections::HashMap<String, Vec<ChunkSourceInfo>> =
        std::collections::HashMap::new();

    for source in &sources {
        if source.contains("-chunk-") {
            if let Some(pos) = source.find("-chunk-") {
                let doc_id = &source[..pos];
                doc_map
                    .entry(doc_id.to_string())
                    .or_default()
                    .push(ChunkSourceInfo {
                        chunk_id: source.clone(),
                        start_line: None,
                        end_line: None,
                        source_text: None,
                    });
            }
        }
    }

    // OODA-27: Resolve document names and chunk positions from cached KV storage
    // WHY: Without document names, the UI shows UUIDs which are not user-friendly.
    // Using cached_kv_get avoids repeated I/O for documents with many entities.
    let mut entity_sources: Vec<EntitySourceInfo> = Vec::with_capacity(doc_map.len());
    for (doc_id, mut chunks) in doc_map {
        // Resolve document name from metadata
        let metadata_key = format!("{}-metadata", doc_id);
        let doc_name =
            if let Ok(Some(meta)) = cached_kv_get(state.kv_storage.as_ref(), &metadata_key).await {
                meta.get("title")
                    .or_else(|| meta.get("file_name"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };

        // Resolve chunk line positions from KV storage
        for chunk in &mut chunks {
            if let Ok(Some(chunk_data)) =
                cached_kv_get(state.kv_storage.as_ref(), &chunk.chunk_id).await
            {
                chunk.start_line = chunk_data
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                chunk.end_line = chunk_data
                    .get("end_line")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
            }
        }

        entity_sources.push(EntitySourceInfo {
            document_id: doc_id,
            document_name: doc_name,
            chunks,
            first_extracted_at: None,
        });
    }

    // Find related entities
    let all_edges = state.graph_storage.get_all_edges().await?;
    let mut related: Vec<RelatedEntityInfo> = Vec::new();

    for edge in all_edges {
        if edge.source == normalized_id {
            related.push(RelatedEntityInfo {
                entity_id: edge.target.clone(),
                entity_name: edge.target.clone(),
                relationship_type: edge
                    .properties
                    .get("keywords")
                    .and_then(|v| v.as_str())
                    .unwrap_or("related_to")
                    .to_string(),
                shared_documents: 1,
            });
        } else if edge.target == normalized_id {
            related.push(RelatedEntityInfo {
                entity_id: edge.source.clone(),
                entity_name: edge.source.clone(),
                relationship_type: edge
                    .properties
                    .get("keywords")
                    .and_then(|v| v.as_str())
                    .unwrap_or("related_to")
                    .to_string(),
                shared_documents: 1,
            });
        }
    }

    Ok(Json(EntityProvenanceResponse {
        entity_id: normalized_id.clone(),
        entity_name: normalized_id,
        entity_type,
        description,
        sources: entity_sources,
        total_extraction_count: sources_count,
        related_entities: related,
    }))
}

pub mod queries;
pub mod export;

pub use queries::*;
pub use export::*;

#[cfg(test)]
mod tests {
    use super::*;
    use super::export::lineage_to_csv;

    #[test]
    fn test_entity_lineage_response_serialization() {
        let response = EntityLineageResponse {
            entity_name: "JOHN_DOE".to_string(),
            entity_type: Some("person".to_string()),
            source_documents: vec![SourceDocumentInfo {
                document_id: "doc-123".to_string(),
                chunk_ids: vec!["doc-123-chunk-0".to_string()],
                line_ranges: vec![LineRangeInfo {
                    start_line: 1,
                    end_line: 10,
                }],
            }],
            source_count: 1,
            description_versions: vec![],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("JOHN_DOE"));
        assert!(json.contains("doc-123"));
    }

    #[test]
    fn test_document_graph_lineage_response_serialization() {
        let response = DocumentGraphLineageResponse {
            document_id: "doc-123".to_string(),
            chunk_count: 5,
            entities: vec![EntitySummaryResponse {
                name: "JOHN_DOE".to_string(),
                entity_type: "person".to_string(),
                source_chunks: vec!["doc-123-chunk-0".to_string()],
                is_shared: false,
            }],
            relationships: vec![RelationshipSummaryResponse {
                source: "JOHN_DOE".to_string(),
                target: "ACME_CORP".to_string(),
                keywords: "works_at".to_string(),
                source_chunks: vec!["doc-123-chunk-0".to_string()],
            }],
            extraction_stats: ExtractionStatsResponse {
                total_entities: 1,
                unique_entities: 1,
                total_relationships: 1,
                unique_relationships: 1,
                processing_time_ms: Some(500),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("doc-123"));
        assert!(json.contains("JOHN_DOE"));
        assert!(json.contains("works_at"));
    }

    #[test]
    fn test_line_range_info_serialization() {
        let info = LineRangeInfo {
            start_line: 10,
            end_line: 20,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"start_line\":10"));
        assert!(json.contains("\"end_line\":20"));
    }

    #[test]
    fn test_extraction_stats_response_serialization() {
        let stats = ExtractionStatsResponse {
            total_entities: 100,
            unique_entities: 50,
            total_relationships: 200,
            unique_relationships: 80,
            processing_time_ms: None,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_entities\":100"));
        assert!(json.contains("\"unique_entities\":50"));
    }

    #[test]
    fn test_description_version_response() {
        let version = DescriptionVersionResponse {
            version: 1,
            description: "Initial description".to_string(),
            source_chunk_id: Some("chunk-123".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&version).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("Initial description"));
    }

    // OODA-22: Export tests
    #[test]
    fn test_lineage_to_csv_basic() {
        let lineage = serde_json::json!({
            "chunks": [
                {
                    "chunk_index": 0,
                    "content": "Hello world",
                    "tokens": 2,
                    "start_line": 1,
                    "end_line": 5,
                    "entity_count": 3
                },
                {
                    "chunk_index": 1,
                    "content": "Second chunk",
                    "tokens": 4,
                    "start_line": 6,
                    "end_line": 10,
                    "entity_count": 1
                }
            ]
        });
        let csv = lineage_to_csv("doc-001", &lineage);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert!(lines[0].starts_with("document_id,chunk_index"));
        assert!(lines[1].contains("doc-001"));
        assert!(lines[1].contains("Hello world"));
        assert!(lines[2].contains("Second chunk"));
    }

    #[test]
    fn test_lineage_to_csv_empty_chunks() {
        let lineage = serde_json::json!({ "chunks": [] });
        let csv = lineage_to_csv("doc-empty", &lineage);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1); // header only
    }

    #[test]
    fn test_lineage_to_csv_no_chunks_key() {
        let lineage = serde_json::json!({ "metadata": {} });
        let csv = lineage_to_csv("doc-no-chunks", &lineage);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1); // header only
    }

    #[test]
    fn test_lineage_to_csv_escapes_quotes() {
        let lineage = serde_json::json!({
            "chunks": [{
                "chunk_index": 0,
                "content": "He said \"hello\" to her",
                "tokens": 5,
                "entity_count": 0
            }]
        });
        let csv = lineage_to_csv("doc-esc", &lineage);
        // Escaped quotes should be doubled inside CSV field
        assert!(csv.contains("\"\"hello\"\""));
    }

    #[test]
    fn test_export_params_default_format() {
        let params: ExportParams = serde_json::from_str("{}").unwrap();
        assert_eq!(params.format, "json");
    }

    #[test]
    fn test_export_params_csv_format() {
        let params: ExportParams = serde_json::from_str(r#"{"format":"csv"}"#).unwrap();
        assert_eq!(params.format, "csv");
    }

    // OODA-23: Cache configuration tests
    #[test]
    fn test_lineage_cache_ttl_is_reasonable() {
        // WHY: TTL must be long enough to absorb polling but short enough for freshness
        assert!(
            LINEAGE_CACHE_TTL.as_secs() >= 30,
            "TTL too short for dashboard polling"
        );
        assert!(
            LINEAGE_CACHE_TTL.as_secs() <= 300,
            "TTL too long for freshness"
        );
    }

    #[test]
    fn test_lineage_cache_max_entries_bounded() {
        // WHY: Unbounded cache = memory leak in production
        assert!(LINEAGE_CACHE_MAX_ENTRIES > 0);
        assert!(LINEAGE_CACHE_MAX_ENTRIES <= 10_000, "Cache too large");
    }

    #[tokio::test]
    async fn test_invalidate_lineage_cache() {
        // Populate cache directly
        {
            let mut cache = LINEAGE_KV_CACHE.write().await;
            cache.insert(
                "test-doc-lineage".to_string(),
                CachedLineage {
                    data: serde_json::json!({"test": true}),
                    cached_at: Instant::now(),
                },
            );
            cache.insert(
                "test-doc-metadata".to_string(),
                CachedLineage {
                    data: serde_json::json!({"meta": true}),
                    cached_at: Instant::now(),
                },
            );
        }

        // Verify entries exist
        {
            let cache = LINEAGE_KV_CACHE.read().await;
            assert!(cache.contains_key("test-doc-lineage"));
            assert!(cache.contains_key("test-doc-metadata"));
        }

        // Invalidate
        invalidate_lineage_cache("test-doc").await;

        // Verify entries removed
        {
            let cache = LINEAGE_KV_CACHE.read().await;
            assert!(!cache.contains_key("test-doc-lineage"));
            assert!(!cache.contains_key("test-doc-metadata"));
        }
    }
}
