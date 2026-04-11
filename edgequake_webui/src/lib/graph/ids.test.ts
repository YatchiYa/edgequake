import { describe, expect, it } from "vitest";

import { getGraphEdgeKey, getGraphEdgeKeyFromEdge } from "./ids";

describe("graph edge keys", () => {
  it("builds deterministic keys from edge parts", () => {
    expect(
      getGraphEdgeKey({
        source: "ALICE",
        target: "ACME",
        relationship_type: "WORKS_FOR",
      }),
    ).toBe("ALICE-ACME-WORKS_FOR");
  });

  it("accepts GraphEdge-compatible objects", () => {
    expect(
      getGraphEdgeKeyFromEdge({
        source: "A",
        target: "B",
        relationship_type: "KNOWS",
      }),
    ).toBe("A-B-KNOWS");
  });
});
