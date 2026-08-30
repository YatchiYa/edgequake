import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "@/lib/api/client";
import { listInjections } from "../injection";

vi.mock("@/lib/api/client", () => ({
  api: {
    get: vi.fn(),
  },
}));

const getMock = api.get as unknown as ReturnType<typeof vi.fn>;

describe("listInjections exhaust", () => {
  beforeEach(() => {
    getMock.mockReset();
  });

  it("follows pages until accumulated >= total", async () => {
    const names = Array.from({ length: 101 }, (_, i) => `inj-${i}`);
    getMock.mockImplementation(async (path: string) => {
      const url = new URL(path, "http://local.invalid");
      const limit = Number(url.searchParams.get("limit") ?? 50);
      const offset = Number(url.searchParams.get("offset") ?? 0);
      return {
        items: names.slice(offset, offset + limit).map((name) => ({
          injection_id: name,
          name,
          status: "completed",
          entity_count: 0,
          source_type: "text",
          created_at: "2026-01-01T00:00:00Z",
          updated_at: "2026-01-01T00:00:00Z",
        })),
        total: names.length,
      };
    });

    const result = await listInjections("ws-1");
    expect(result.items).toHaveLength(101);
    expect(result.items.map((i) => i.name)).toContain("inj-100");
    expect(getMock.mock.calls.length).toBe(2);
  });
});
