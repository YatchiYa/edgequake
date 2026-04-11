import type { GraphEdge } from "@/types";

export interface GraphEdgeKeyParts {
  source: string;
  target: string;
  relationship_type: string;
}

export function getGraphEdgeKey({
  source,
  target,
  relationship_type,
}: GraphEdgeKeyParts): string {
  return `${source}-${target}-${relationship_type}`;
}

export function getGraphEdgeKeyFromEdge(edge: Pick<GraphEdge, "source" | "target" | "relationship_type">): string {
  return getGraphEdgeKey(edge);
}
