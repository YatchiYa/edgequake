import { describe, expect, it } from "vitest";

import {
  documentLineageToKnowledgeGraph,
  parseRelationshipType,
} from "../document-lineage-to-graph";
import type { DocumentGraphLineageResponse } from "@/types/lineage";

describe("parseRelationshipType", () => {
  it("normalizes plain keywords to uppercase snake case", () => {
    expect(parseRelationshipType("works for")).toBe("WORKS_FOR");
  });

  it("uses first token when keywords are comma-separated", () => {
    expect(parseRelationshipType("WORKS_FOR, employed by")).toBe("WORKS_FOR");
  });

  it("falls back to RELATED_TO for empty input", () => {
    expect(parseRelationshipType("")).toBe("RELATED_TO");
    expect(parseRelationshipType("   ")).toBe("RELATED_TO");
  });
});

describe("documentLineageToKnowledgeGraph", () => {
  const sampleLineage: DocumentGraphLineageResponse = {
    document_id: "doc-123",
    chunk_count: 2,
    entities: [
      {
        name: "ALICE_CHEN",
        entity_type: "person",
        source_chunks: ["chunk-1"],
        is_shared: false,
      },
      {
        name: "ACME_CORP",
        entity_type: "organization",
        source_chunks: ["chunk-1", "chunk-2"],
        is_shared: true,
      },
    ],
    relationships: [
      {
        source: "ALICE_CHEN",
        target: "ACME_CORP",
        keywords: "WORKS_FOR",
        source_chunks: ["chunk-1"],
      },
    ],
    extraction_stats: {
      total_entities: 2,
      unique_entities: 2,
      total_relationships: 1,
      unique_relationships: 1,
    },
  };

  it("maps entities and relationships into a knowledge graph", () => {
    const graph = documentLineageToKnowledgeGraph(sampleLineage);

    expect(graph.nodes).toHaveLength(2);
    expect(graph.edges).toHaveLength(1);
    expect(graph.metadata.node_count).toBe(2);
    expect(graph.metadata.edge_count).toBe(1);
    expect(graph.metadata.entity_types).toEqual(["organization", "person"]);
    expect(graph.metadata.relationship_types).toEqual(["WORKS_FOR"]);
  });

  it("sets node degree from incident edges", () => {
    const graph = documentLineageToKnowledgeGraph(sampleLineage);
    const alice = graph.nodes.find((n) => n.id === "ALICE_CHEN");
    const acme = graph.nodes.find((n) => n.id === "ACME_CORP");

    expect(alice?.degree).toBe(1);
    expect(acme?.degree).toBe(1);
  });

  it("preserves document and chunk provenance on nodes and edges", () => {
    const graph = documentLineageToKnowledgeGraph(sampleLineage);
    const edge = graph.edges[0];

    expect(graph.nodes[0]?.properties?.document_id).toBe("doc-123");
    expect(edge?.properties?.document_id).toBe("doc-123");
    expect(edge?.source_ids).toEqual(["chunk-1"]);
  });
});
