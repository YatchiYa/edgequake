import { afterEach, describe, expect, it, vi } from "vitest";
import { applyOperations, describeOperation, parseOperations, summarizeOntology } from "../tools";

const ontology = {
  name: "Propulsion",
  entity_types: [{ id: "ENGINE", description: "A gas turbine engine.", parent: "PRODUCT" }],
  terms: [{ name: "LEAP-1A", entity_type: "ENGINE", aliases: ["LEAP 1A", "Leap1A"] }],
};
const scope = { authorization: "Bearer t", tenantId: "t1", workspaceId: "ws-1" };

afterEach(() => vi.unstubAllGlobals());

describe("copilot ontology operations (SPEC-184)", () => {
  it("accepts ontology operations and drops a set_ontology that is not an ontology", () => {
    const ops = parseOperations([
      { op: "set_ontology", ontology },
      { op: "set_ontology", ontology: "drop table" },
      { op: "set_ontology", ontology: { name: "no lists" } },
      { op: "realign_ontology" },
      { op: "rebuild_knowledge_graph" },
    ]);
    expect(ops.map((o) => o.op)).toEqual(["set_ontology", "realign_ontology", "rebuild_knowledge_graph"]);
  });

  it("states the exact effect on the approval card", () => {
    expect(describeOperation({ op: "set_ontology", ontology })).toContain(
      "Save ontology “Propulsion”: 1 entity types, 0 relation types, 1 terms (2 aliases)",
    );
    expect(describeOperation({ op: "rebuild_knowledge_graph" })).toContain("re-extract every document");
    expect(describeOperation({ op: "realign_ontology" })).toContain("merge alias entities");
  });

  it("summarizes without dumping the whole ontology", () => {
    const big = { ...ontology, terms: Array.from({ length: 500 }, (_, i) => ({ name: `PART-${i}`, aliases: [`P${i}`] })) };
    const summary = JSON.parse(summarizeOntology(big));
    expect(summary.terms_total).toBe(500);
    expect(summary.terms_sample).toHaveLength(40);
  });

  it("applies save, realign rounds and rebuild against the workspace routes in order", async () => {
    const calls: string[] = [];
    let realignRounds = 0;
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string, init: RequestInit) => {
        calls.push(`${init.method} ${url.replace(/^.*\/api\/v1/, "")}`);
        if (url.endsWith("/ontology/realign")) {
          realignRounds += 1;
          const remaining = realignRounds === 1 ? 5 : 0;
          return new Response(
            JSON.stringify({ steps: [{ op: "merge", entity: "LEAP_1A", ok: true, detail: "" }], merged: 1, retyped: 0, failed: 0, remaining }),
          );
        }
        return new Response("{}");
      }),
    );
    const results = await applyOperations(
      parseOperations([{ op: "set_ontology", ontology }, { op: "realign_ontology" }, { op: "rebuild_knowledge_graph" }]),
      scope,
    );
    expect(results.every((r) => r.ok)).toBe(true);
    expect(calls).toEqual([
      "PUT /workspaces/ws-1/ontology",
      "POST /workspaces/ws-1/ontology/realign",
      "POST /workspaces/ws-1/ontology/realign",
      "POST /workspaces/ws-1/rebuild-knowledge-graph",
      "POST /workspaces/ws-1/reprocess-documents",
    ]);
    expect(results[1].entities).toEqual(["LEAP_1A", "LEAP_1A"]);
  });
});
