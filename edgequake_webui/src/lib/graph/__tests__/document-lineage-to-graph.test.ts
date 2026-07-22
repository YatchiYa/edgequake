import { describe, expect, it } from "vitest";

import {
  documentLineageToKnowledgeGraph,
  normalizeEntityType,
  parseRelationshipType,
} from "../document-lineage-to-graph";
import type { DocumentGraphLineageResponse } from "@/types/lineage";

describe("normalizeEntityType", () => {
  it("uppercases and snake-cases labels", () => {
    expect(normalizeEntityType("concept")).toBe("CONCEPT");
    expect(normalizeEntityType("  works for  ")).toBe("WORKS_FOR");
    expect(normalizeEntityType(undefined)).toBe("UNKNOWN");
  });
});

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
        id: "ALICE_CHEN",
        name: "ALICE_CHEN",
        label: "ALICE_CHEN",
        entity_type: "person",
        source_chunks: ["chunk-1"],
        is_shared: false,
      },
      {
        id: "ACME_CORP",
        name: "ACME_CORP",
        label: "ACME_CORP",
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
    expect(graph.metadata.entity_types).toEqual(["ORGANIZATION", "PERSON"]);
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

  it("072: prefers API soft-label over opaque id for canvas labels", () => {
    const opaqueId = "84b69e27-e38b-444a-83dd-5e6a537c6f12";
    const lineage: DocumentGraphLineageResponse = {
      document_id: "doc-opaque",
      chunk_count: 1,
      entities: [
        {
          id: opaqueId,
          name: "Opaque ID · CONCEPT",
          label: "Future of work theme from the agenda",
          entity_type: "concept",
          source_chunks: ["doc-opaque-chunk-0"],
          is_shared: false,
          description: "Future of work theme from the agenda",
        },
        {
          id: "AI_NEXT_CONFERENCE",
          name: "AI_NEXT_CONFERENCE",
          label: "AI_NEXT_CONFERENCE",
          entity_type: "event",
          source_chunks: ["doc-opaque-chunk-0"],
          is_shared: false,
        },
      ],
      relationships: [
        {
          source: opaqueId,
          target: "AI_NEXT_CONFERENCE",
          keywords: "RELATED_TO",
          source_chunks: ["doc-opaque-chunk-0"],
        },
      ],
      extraction_stats: {
        total_entities: 2,
        unique_entities: 2,
        total_relationships: 1,
        unique_relationships: 1,
      },
    };

    const graph = documentLineageToKnowledgeGraph(lineage);
    const opaqueNode = graph.nodes.find((n) => n.id === opaqueId);
    expect(opaqueNode?.label.toLowerCase()).toContain("future of work");
    expect(opaqueNode?.label).not.toMatch(/84b69e27/i);
    expect(graph.edges[0]?.source).toBe(opaqueId);
    expect(graph.edges[0]?.target).toBe("AI_NEXT_CONFERENCE");
  });

  it("072: BC fallback when API omits label still formats name", () => {
    const lineage: DocumentGraphLineageResponse = {
      document_id: "doc-bc",
      chunk_count: 1,
      entities: [
        {
          name: "SARAH_CHEN",
          entity_type: "person",
          source_chunks: ["c0"],
          is_shared: false,
        },
      ],
      relationships: [],
      extraction_stats: {
        total_entities: 1,
        unique_entities: 1,
        total_relationships: 0,
        unique_relationships: 0,
      },
    };
    const graph = documentLineageToKnowledgeGraph(lineage);
    expect(graph.nodes[0]?.id).toBe("SARAH_CHEN");
    expect(graph.nodes[0]?.label).toContain("Sarah");
  });
});
