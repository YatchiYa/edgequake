/**
 * E2E: Workspace tools test.
 */
import type { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { callTool, createTestClient, isServerRunning } from "./helpers.js";

describe("workspace tools (e2e)", () => {
  let client: Client;
  let cleanup: () => Promise<void>;
  let serverUp: boolean;

  beforeAll(async () => {
    serverUp = await isServerRunning();
    if (!serverUp) return;
    const ctx = await createTestClient();
    client = ctx.client;
    cleanup = ctx.cleanup;
  });

  afterAll(async () => {
    if (cleanup) await cleanup();
  });

  it("should list workspaces", async () => {
    if (!serverUp) {
      console.log("SKIP: EdgeQuake server not running");
      return;
    }
    const result = await callTool(client, "workspace_list");
    expect(Array.isArray(result)).toBe(true);
  });

  it("should create, get, stats, and delete a workspace", async () => {
    if (!serverUp) {
      console.log("SKIP: EdgeQuake server not running");
      return;
    }

    // Create
    const created = (await callTool(client, "workspace_create", {
      name: "mcp-e2e-test",
      description: "E2E test workspace",
    })) as { id: string; name: string; slug: string };
    expect(created).toHaveProperty("id");
    expect(created.name).toBe("mcp-e2e-test");

    const workspaceId = created.id;

    // Get
    const detail = (await callTool(client, "workspace_get", {
      workspace_id: workspaceId,
    })) as Record<string, unknown>;
    expect(detail.id).toBe(workspaceId);
    expect(detail.name).toBe("mcp-e2e-test");

    // Stats
    const stats = (await callTool(client, "workspace_stats", {
      workspace_id: workspaceId,
    })) as Record<string, unknown>;
    expect(stats).toHaveProperty("document_count");
    expect(stats).toHaveProperty("entity_count");

    // Delete
    const deleted = (await callTool(client, "workspace_delete", {
      workspace_id: workspaceId,
    })) as { success: boolean };
    expect(deleted.success).toBe(true);
  });
});

describe("SPEC-141 workspace_list completeness", () => {
  let client: Client;
  let cleanup: () => Promise<void>;
  let serverUp: boolean;
  const names: string[] = [];

  beforeAll(async () => {
    serverUp = await isServerRunning();
    if (!serverUp) return;
    const { resetClientForTests } = await import("../../src/client.js");
    const baseUrl = process.env.EDGEQUAKE_BASE_URL ?? "http://localhost:8080";
    const suffix = Date.now();
    const tenantRes = await fetch(`${baseUrl}/api/v1/tenants`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name: `spec141-mcp ${suffix}`,
        plan: "pro",
      }),
    });
    const tenantText = await tenantRes.text();
    expect(tenantRes.ok, tenantText).toBe(true);
    const tenant = JSON.parse(tenantText) as { id: string };
    for (let i = 0; i < 21; i += 1) {
      const name = `spec141-mcp-${suffix}-${i}`;
      names.push(name);
      const wsRes = await fetch(
        `${baseUrl}/api/v1/tenants/${tenant.id}/workspaces`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name,
            slug: `${name}-${suffix}`.toLowerCase(),
          }),
        },
      );
      const wsText = await wsRes.text();
      expect(wsRes.ok, wsText).toBe(true);
    }
    process.env.EDGEQUAKE_DEFAULT_TENANT = tenant.id;
    resetClientForTests();
    const ctx = await createTestClient();
    client = ctx.client;
    cleanup = ctx.cleanup;
  });

  afterAll(async () => {
    if (cleanup) await cleanup();
    delete process.env.EDGEQUAKE_DEFAULT_TENANT;
  });

  it("returns all 21 uniquely created workspaces", async () => {
    if (!serverUp) {
      console.log("SKIP: EdgeQuake server not running");
      return;
    }
    const list = (await callTool(client, "workspace_list")) as Array<{
      name?: string;
    }>;
    expect(Array.isArray(list)).toBe(true);
    const listed = new Set(list.map((w) => w.name));
    expect(list.length).toBeGreaterThanOrEqual(21);
    for (const name of names) {
      expect(listed.has(name), `workspace_list missing ${name}`).toBe(true);
    }
  });
});
