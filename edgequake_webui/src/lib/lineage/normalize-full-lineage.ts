/**
 * Normalize persisted KV lineage (/documents/:id/lineage) into DocumentLineageResponse.
 * WHY: Graph endpoint (/lineage/documents/:id) returns summaries without chunk trees.
 */

import type {
  ChunkLineage,
  DocumentFullLineageResponse,
  DocumentLineageResponse,
  EntityLineage,
  LineageStatistics,
} from '@/types/lineage';

interface FullLineageChunk {
  chunk_id: string;
  chunk_index: number;
  content_preview?: string;
  start_line?: number;
  end_line?: number;
  start_offset?: number;
  end_offset?: number;
  entity_ids?: string[];
  relationship_ids?: string[];
  extraction_metadata?: Record<string, unknown>;
  page_start?: number;
  page_end?: number;
  token_count?: number;
}

interface FullLineageEntity {
  entity_id: string;
  entity_name: string;
  entity_type?: string;
  extraction_count: number;
  sources?: Array<{ chunk_ids?: string[] }>;
  description_history?: Array<{ description?: string }>;
}

export function normalizeFullLineageResponse(
  response: DocumentFullLineageResponse,
): DocumentLineageResponse | null {
  const lineage = response.lineage as Record<string, unknown> | undefined;
  if (!lineage) return null;

  const rawChunks = (lineage.chunks ?? []) as FullLineageChunk[];
  const rawEntities = (lineage.entities ?? {}) as Record<string, FullLineageEntity>;

  const entities: EntityLineage[] = Object.values(rawEntities).map((entity) => ({
    id: entity.entity_id,
    name: entity.entity_name,
    entity_type: entity.entity_type ?? '',
    description: entity.description_history?.[0]?.description,
    source_chunks: (entity.sources ?? []).flatMap((source) => source.chunk_ids ?? []),
    extraction_count: entity.extraction_count ?? 1,
  }));

  const chunks: ChunkLineage[] = rawChunks.map((chunk) => {
    const entityIds = chunk.entity_ids ?? [];
    return {
      chunk_id: chunk.chunk_id,
      id: chunk.chunk_id,
      chunk_index: chunk.chunk_index,
      index: chunk.chunk_index,
      content_preview: chunk.content_preview,
      start_line: chunk.start_line,
      end_line: chunk.end_line,
      start_offset: chunk.start_offset,
      end_offset: chunk.end_offset,
      page_start: chunk.page_start,
      page_end: chunk.page_end,
      token_count: chunk.token_count ?? 0,
      extracted_entities: entityIds,
      entities: entityIds,
      extracted_relationships: chunk.relationship_ids ?? [],
      relationships: chunk.relationship_ids ?? [],
    };
  });

  const metadata = response.metadata as Record<string, unknown> | undefined;
  const documentName = String(metadata?.name ?? metadata?.title ?? '');

  const entityTypes = [...new Set(entities.map((entity) => entity.entity_type).filter(Boolean))];
  const summary: LineageStatistics = {
    total_chunks: chunks.length,
    total_entities: entities.length,
    total_relationships: 0,
    deduplication_rate: 0,
    unique_entity_types: entityTypes,
    unique_relationship_types: [],
  };

  return {
    document_id: response.document_id,
    document_name: documentName,
    summary,
    chunks,
    entities,
    relationships: [],
    created_at: String(metadata?.created_at ?? ''),
  };
}
