import { describe, expect, it } from "vitest";
import { E2E_ENABLED, createE2EClient } from "./helpers.js";

const describeE2E = E2E_ENABLED ? describe : describe.skip;

describeE2E("E2E: shared live scenario", () => {
  it("covers health, documents, graph, query, connections, and chat top_k", async () => {
    const client = createE2EClient();
    expect(client).toBeDefined();
    if (!client) return;
    const health = await client.health();
    expect(health.status).toBe("healthy");
    expect(JSON.stringify(await client.operations.setupStatus())).toContain("needs_setup");
    expect(client.operations.wsPipelineProgress()).toMatch(/^wss?:\/\/.+\/ws\/pipeline\/progress$/);
    const listed = await client.documents.list({ page: 1, page_size: 5 });
    expect(listed.documents).toBeDefined();
    for (const item of listed.documents ?? []) {
      void item.display_status;
    }
    const uploaded = await client.documents.upload({
      content: "Shared SDK live scenario.",
      title: `shared-live-${Date.now()}`,
    });
    expect(uploaded.document_id).toBeTruthy();
    expect(await client.graph.get()).toBeDefined();
    expect(await client.graph.entities.list({ page: 1, page_size: 5 })).toBeDefined();
    expect(await client.query.execute({ query: "What is a knowledge graph?", max_results: 3 })).toBeDefined();
    expect(await client.connections.list()).toBeDefined();
    expect(await client.chat.completions({ message: "ping", top_k: 2 })).toBeDefined();
  }, 120_000);
});
