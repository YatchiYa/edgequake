import Graph from "graphology";
import { describe, expect, it } from "vitest";

import {
  applyLayoutToGraph,
  calculateLayoutPositions,
  getGraphPerformanceProfile,
} from "./layouts";

function buildTestGraph(): Graph {
  const graph = new Graph();

  graph.addNode("a", { x: 0, y: 0, entityType: "PERSON" });
  graph.addNode("b", { x: 1, y: 0, entityType: "PERSON" });
  graph.addNode("c", { x: 0, y: 1, entityType: "ORG" });
  graph.addEdgeWithKey("a-b", "a", "b");
  graph.addEdgeWithKey("a-c", "a", "c");

  return graph;
}

describe("graph layouts", () => {
  it("computes performance thresholds consistently", () => {
    expect(getGraphPerformanceProfile(50, 80)).toMatchObject({
      isLargeGraph: false,
      isVeryLargeGraph: false,
      disableEdgeEvents: false,
      labelGridCellSize: 80,
    });

    expect(getGraphPerformanceProfile(250, 300)).toMatchObject({
      isLargeGraph: true,
      isVeryLargeGraph: false,
      disableEdgeEvents: false,
      labelGridCellSize: 100,
    });

    expect(getGraphPerformanceProfile(600, 1200)).toMatchObject({
      isLargeGraph: true,
      isVeryLargeGraph: true,
      disableEdgeEvents: true,
      labelGridCellSize: 150,
    });
  });

  it("returns positions for each node", () => {
    const graph = buildTestGraph();
    const positions = calculateLayoutPositions(graph, "force", "interactive");

    expect(Object.keys(positions).sort()).toEqual(["a", "b", "c"]);
    expect(Number.isFinite(positions.a.x)).toBe(true);
    expect(Number.isFinite(positions.a.y)).toBe(true);
  });

  it("applies hierarchical layout by grouping entity types into levels", () => {
    const graph = buildTestGraph();
    applyLayoutToGraph(graph, "hierarchical");

    expect(graph.getNodeAttribute("a", "y")).toBe(graph.getNodeAttribute("b", "y"));
    expect(graph.getNodeAttribute("a", "y")).not.toBe(graph.getNodeAttribute("c", "y"));
  });
});
