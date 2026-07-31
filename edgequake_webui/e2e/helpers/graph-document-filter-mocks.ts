/**
 * Mock routes + tenant seed for graph document filter E2E (SPEC-045).
 */
import type { Page } from "@playwright/test";

export const GRAPH_FILTER_MOCK_TENANT =
  "tenant-graph-filter-aaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
export const GRAPH_FILTER_MOCK_WORKSPACE =
  "ws-graph-filter-aaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
export const GRAPH_FILTER_DOC_A =
  "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
export const GRAPH_FILTER_DOC_B =
  "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

export async function seedGraphFilterTenantContext(page: Page): Promise<void> {
  await page.goto("/", { waitUntil: "domcontentloaded" });
  await page.evaluate(
    ({ tenantId, workspaceId }) => {
      localStorage.clear();
      sessionStorage.clear();
      const userId = crypto.randomUUID();
      localStorage.setItem("userId", userId);
      localStorage.setItem("tenantId", tenantId);
      localStorage.setItem("workspaceId", workspaceId);
      localStorage.setItem(
        "edgequake-tenant",
        JSON.stringify({
          state: {
            selectedTenantId: tenantId,
            selectedWorkspaceId: workspaceId,
            workspaces: [
              {
                id: workspaceId,
                tenant_id: tenantId,
                name: "Graph Filter WS",
                slug: "graph-filter-ws",
              },
            ],
            tenants: [
              {
                id: tenantId,
                name: "GraphFilterTenant",
                slug: "graph-filter-tenant",
              },
            ],
          },
          version: 1,
        }),
      );
    },
    {
      tenantId: GRAPH_FILTER_MOCK_TENANT,
      workspaceId: GRAPH_FILTER_MOCK_WORKSPACE,
    },
  );
  await page.reload({ waitUntil: "domcontentloaded" });
}

export async function mockGraphDocumentFilterRoutes(page: Page): Promise<void> {
  await page.route("**/health", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ status: "healthy" }),
    });
  });

  await page.route("**/ready", async (route) => {
    await route.fulfill({ status: 200, body: "OK" });
  });

  await page.route("**/live", async (route) => {
    await route.fulfill({ status: 200, body: "OK" });
  });

  await page.route("**/api/v1/graph/stream**", async (route) => {
    await route.fulfill({
      status: 500,
      body: "graph stream must not run in document-scoped mode",
    });
  });

  await page.route("**/api/v1/lineage/documents/**", async (route) => {
    const url = route.request().url();
    const docId = url.split("/lineage/documents/")[1]?.split("?")[0] ?? "";
    const isDocA = docId === GRAPH_FILTER_DOC_A;
    const entities = isDocA
      ? [
          {
            name: "PE8_ENTITY_A",
            entity_type: "concept",
            source_chunks: [`${docId}-chunk-0`],
            is_shared: false,
          },
          {
            name: "PE8_ENTITY_B",
            entity_type: "technology",
            source_chunks: [`${docId}-chunk-0`],
            is_shared: false,
          },
        ]
      : [
          {
            name: "OTHER_A",
            entity_type: "concept",
            source_chunks: [`${docId}-chunk-0`],
            is_shared: false,
          },
        ];
    const relationships = isDocA
      ? [
          {
            source: "PE8_ENTITY_A",
            target: "PE8_ENTITY_B",
            keywords: "relates_to",
            source_chunks: [`${docId}-chunk-0`],
          },
        ]
      : [];

    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        document_id: docId,
        chunk_count: 1,
        entities,
        relationships,
        extraction_stats: {
          total_entities: entities.length,
          unique_entities: entities.length,
          total_relationships: relationships.length,
          unique_relationships: relationships.length,
        },
      }),
    });
  });

  await page.route("**/api/v1/**", async (route) => {
    const url = route.request().url();
    const method = route.request().method();

    if (url.includes("/graph/stream") || url.includes("/lineage/documents/")) {
      await route.fallback();
      return;
    }

    if (url.includes("/tenants/") && url.includes("/workspaces") && method === "GET") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([
          {
            id: GRAPH_FILTER_MOCK_WORKSPACE,
            tenant_id: GRAPH_FILTER_MOCK_TENANT,
            name: "Graph Filter WS",
            slug: "graph-filter-ws",
          },
        ]),
      });
      return;
    }

    if (url.endsWith("/tenants") || url.match(/\/tenants\?/)) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify([
          {
            id: GRAPH_FILTER_MOCK_TENANT,
            name: "GraphFilterTenant",
            slug: "graph-filter-tenant",
          },
        ]),
      });
      return;
    }

    // Single-document detail (pill label SSOT on cold ?document= deep link)
    const detailMatch = url.match(/\/documents\/([^/?]+)(?:\?|$)/);
    if (detailMatch && method === "GET" && !url.includes("/documents/search")) {
      const docId = decodeURIComponent(detailMatch[1] ?? "");
      const titles: Record<string, string> = {
        [GRAPH_FILTER_DOC_A]: "manifold_2605.13438v3.pdf",
        [GRAPH_FILTER_DOC_B]: "cognifold_2605.13438v3.pdf",
      };
      const title = titles[docId];
      if (title) {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            id: docId,
            title,
            file_name: title,
            status: "completed",
          }),
        });
        return;
      }
    }

    if (url.includes("/documents") && method === "GET") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          documents: [
            {
              id: GRAPH_FILTER_DOC_A,
              title: "manifold_2605.13438v3.pdf",
              status: "completed",
              entity_count: 2,
            },
            {
              id: GRAPH_FILTER_DOC_B,
              title: "cognifold_2605.13438v3.pdf",
              status: "completed",
              entity_count: 1,
            },
          ],
          total: 2,
        }),
      });
      return;
    }

    if (url.includes("/graph") && method === "GET") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          nodes: Array.from({ length: 1200 }, (_, i) => ({
            id: `FULL_NODE_${i}`,
            label: `Node ${i}`,
            node_type: "CONCEPT",
          })),
          edges: [],
          metadata: {
            node_count: 1200,
            edge_count: 0,
            entity_types: ["CONCEPT"],
            relationship_types: [],
          },
        }),
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({}),
    });
  });
}
