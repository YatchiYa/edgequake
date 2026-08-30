import { describe, expect, it } from "vitest";
import {
  FETCH_ALL_PAGES_MAX,
  fetchAllPages,
  fetchAllPagesByIndex,
  SELECTOR_PAGE_LIMIT,
} from "../fetch-all-pages";

describe("fetchAllPages", () => {
  it("returns a single short page", async () => {
    const rows = await fetchAllPages(async () => ({
      items: [{ id: "a" }, { id: "b" }],
      total: 2,
    }));
    expect(rows.map((r) => r.id)).toEqual(["a", "b"]);
  });

  it("loops until accumulated >= total", async () => {
    const all = Array.from({ length: 21 }, (_, i) => ({ id: `w${i}` }));
    let calls = 0;
    const rows = await fetchAllPages(async (offset, limit) => {
      calls += 1;
      return {
        items: all.slice(offset, offset + limit),
        total: all.length,
      };
    }, 10);
    expect(rows).toHaveLength(21);
    expect(calls).toBe(3);
  });

  it("stops on a short last page even if total is a lie", async () => {
    const rows = await fetchAllPages(async (offset) => {
      if (offset === 0) {
        return { items: [{ id: "1" }, { id: "2" }], total: 99 };
      }
      return { items: [{ id: "3" }], total: 99 };
    }, 2);
    expect(rows.map((r) => r.id)).toEqual(["1", "2", "3"]);
  });

  it("stops on an empty page", async () => {
    const rows = await fetchAllPages(async () => ({ items: [], total: 0 }));
    expect(rows).toEqual([]);
  });

  it("caps runaway servers", async () => {
    let calls = 0;
    await fetchAllPages(async (offset) => {
      calls += 1;
      return {
        items: [{ id: `x${offset}` }],
        total: 10_000,
      };
    }, 1);
    expect(calls).toBe(FETCH_ALL_PAGES_MAX);
  });

  it("defaults page size to API max 100", () => {
    expect(SELECTOR_PAGE_LIMIT).toBe(100);
  });
});

describe("fetchAllPagesByIndex", () => {
  it("loops 1-based pages until accumulated >= total", async () => {
    const all = Array.from({ length: 21 }, (_, i) => ({ id: `t${i}` }));
    let calls = 0;
    const rows = await fetchAllPagesByIndex(async (page, pageSize) => {
      calls += 1;
      const start = (page - 1) * pageSize;
      return {
        items: all.slice(start, start + pageSize),
        total: all.length,
      };
    }, 10);
    expect(rows).toHaveLength(21);
    expect(calls).toBe(3);
  });

  it("stops on a short last page", async () => {
    const rows = await fetchAllPagesByIndex(async (page) => {
      if (page === 1) {
        return { items: [{ id: "1" }, { id: "2" }], total: 99 };
      }
      return { items: [{ id: "3" }], total: 99 };
    }, 2);
    expect(rows.map((r) => r.id)).toEqual(["1", "2", "3"]);
  });
});
