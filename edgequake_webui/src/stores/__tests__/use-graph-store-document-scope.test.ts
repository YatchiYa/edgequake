import { describe, expect, it } from "vitest";

import type { GraphEdge, GraphNode } from "@/types";
import { useGraphStore } from "@/stores/use-graph-store";

describe("useGraphStore document scope guard", () => {
  it("blocks addNodesToGraph while documentFilterId is set", () => {
    const store = useGraphStore.getState();
    store.clearGraphForStreaming();
    store.setDocumentFilterId("doc-scope-test");

    const node: GraphNode = {
      id: "LEAKED_NODE",
      label: "Leaked",
      node_type: "CONCEPT",
    };
    const edge: GraphEdge = {
      id: "e1",
      source: "LEAKED_NODE",
      target: "LEAKED_NODE",
      relationship_type: "RELATED_TO",
      weight: 1,
      created_at: new Date().toISOString(),
    };

    store.addNodesToGraph([node], [edge]);

    const after = useGraphStore.getState();
    expect(after.nodes).toHaveLength(0);
    expect(after.edges).toHaveLength(0);

    store.setDocumentFilterId(null);
    store.addNodesToGraph([node], [edge]);
    expect(useGraphStore.getState().nodes).toHaveLength(1);
    expect(useGraphStore.getState().edges).toHaveLength(1);

    store.clearGraphForStreaming();
    store.setDocumentFilterId(null);
  });
});
