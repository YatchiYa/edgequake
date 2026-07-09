import { getGraphEdgeKey } from "@/lib/graph/ids";
import { formatEntityLabel } from "@/lib/graph/label-utils";
import type { GraphEdge, GraphNode, KnowledgeGraph } from "@/types";
import type { DocumentGraphLineageResponse } from "@/types/lineage";

/** Normalize entity type labels for graph filter consistency. */
export function normalizeEntityType(entityType: string | undefined): string {
  const trimmed = (entityType ?? "unknown").trim();
  if (!trimmed) return "UNKNOWN";
  return trimmed.toUpperCase().replace(/\s+/g, "_");
}

/** Normalize relationship keywords from lineage API into a graph edge label. */
export function parseRelationshipType(keywords: string): string {
  const trimmed = keywords.trim();
  if (!trimmed) return "RELATED_TO";
  const first = trimmed.split(/[,|]/)[0]?.trim() ?? "";
  if (/^[A-Z][A-Z0-9_]*$/.test(first)) return first;
  return first.toUpperCase().replace(/\s+/g, "_") || "RELATED_TO";
}

/**
 * Convert document-scoped lineage (`GET /lineage/documents/:id`) into a
 * {@link KnowledgeGraph} suitable for the Sigma.js viewer.
 */
export function documentLineageToKnowledgeGraph(
  lineage: DocumentGraphLineageResponse,
): KnowledgeGraph {
  const documentId = lineage.document_id;
  const degree = new Map<string, number>();

  const bump = (id: string) => degree.set(id, (degree.get(id) ?? 0) + 1);

  const nodes: GraphNode[] = lineage.entities.map((entity) => {
    const id = entity.name;
    return {
      id,
      label: formatEntityLabel(id),
      node_type: normalizeEntityType(entity.entity_type),
      description: entity.is_shared
        ? "Shared with other documents in this workspace"
        : undefined,
      properties: {
        source_chunks: entity.source_chunks,
        is_shared: entity.is_shared,
        document_id: documentId,
      },
    };
  });

  const edges: GraphEdge[] = lineage.relationships.map((rel) => {
    const relationship_type = parseRelationshipType(rel.keywords);
    bump(rel.source);
    bump(rel.target);
    const id = getGraphEdgeKey({
      source: rel.source,
      target: rel.target,
      relationship_type,
    });
    return {
      id,
      source: rel.source,
      target: rel.target,
      relationship_type,
      weight: 1,
      description: rel.keywords || undefined,
      source_ids: rel.source_chunks,
      properties: {
        source_chunks: rel.source_chunks,
        document_id: documentId,
      },
      created_at: new Date().toISOString(),
    };
  });

  for (const node of nodes) {
    node.degree = degree.get(node.id) ?? 0;
  }

  const entity_types = [...new Set(nodes.map((n) => n.node_type))].sort();
  const relationship_types = [
    ...new Set(edges.map((e) => e.relationship_type)),
  ].sort();

  return {
    nodes,
    edges,
    metadata: {
      node_count: nodes.length,
      edge_count: edges.length,
      entity_types,
      relationship_types,
    },
    is_truncated: false,
    total_nodes: nodes.length,
    total_edges: edges.length,
  };
}
