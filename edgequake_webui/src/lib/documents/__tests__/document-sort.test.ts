/**
 * Unit tests for document list sort SSOT.
 */
import { describe, expect, it } from "bun:test";
import type { Document } from "@/types";
import {
  ariaSortForColumn,
  compareDocumentsBySort,
  documentSortValue,
  nextDocumentSortState,
  sortDocuments,
} from "@/lib/documents/document-sort";

function doc(partial: Partial<Document> & { id: string }): Document {
  return {
    title: partial.title ?? partial.file_name ?? partial.id,
    file_name: partial.file_name ?? `${partial.id}.pdf`,
    status: partial.status ?? "completed",
    created_at: partial.created_at ?? "2026-01-01T00:00:00Z",
    updated_at: partial.updated_at ?? partial.created_at ?? "2026-01-01T00:00:00Z",
    entity_count: partial.entity_count,
    chunk_count: partial.chunk_count,
    cost_usd: partial.cost_usd,
    ...partial,
  } as Document;
}

describe("nextDocumentSortState", () => {
  it("toggles direction when same field is clicked", () => {
    expect(nextDocumentSortState("entity_count", "desc", "entity_count")).toEqual({
      field: "entity_count",
      direction: "asc",
    });
  });

  it("defaults title/status to ascending on first click", () => {
    expect(nextDocumentSortState("created_at", "desc", "title")).toEqual({
      field: "title",
      direction: "asc",
    });
  });

  it("defaults metrics/dates to descending on first click", () => {
    expect(nextDocumentSortState("title", "asc", "cost_usd")).toEqual({
      field: "cost_usd",
      direction: "desc",
    });
  });
});

describe("documentSortValue / sortDocuments", () => {
  const docs = [
    doc({
      id: "a",
      title: "zebra.pdf",
      entity_count: 10,
      cost_usd: 0.5,
      created_at: "2026-01-01T00:00:00Z",
      updated_at: "2026-01-03T00:00:00Z",
      status: "completed",
    }),
    doc({
      id: "b",
      title: "alpha.pdf",
      entity_count: 100,
      cost_usd: 0.1,
      created_at: "2026-01-02T00:00:00Z",
      updated_at: "2026-01-02T00:00:00Z",
      status: "failed",
    }),
    doc({
      id: "c",
      title: "middle.pdf",
      entity_count: 50,
      cost_usd: 0.9,
      created_at: "2026-01-03T00:00:00Z",
      updated_at: "2026-01-01T00:00:00Z",
      status: "processing",
    }),
  ];

  it("sorts by entity_count descending", () => {
    const sorted = sortDocuments(docs, "entity_count", "desc");
    expect(sorted.map((d) => d.id)).toEqual(["b", "c", "a"]);
  });

  it("sorts by cost_usd ascending", () => {
    const sorted = sortDocuments(docs, "cost_usd", "asc");
    expect(sorted.map((d) => d.id)).toEqual(["b", "a", "c"]);
  });

  it("sorts by title ascending", () => {
    const sorted = sortDocuments(docs, "title", "asc");
    expect(sorted.map((d) => d.id)).toEqual(["b", "c", "a"]);
  });

  it("uses updated_at (not created_at) for updated_at field", () => {
    expect(documentSortValue(docs[0], "updated_at")).toBeGreaterThan(
      documentSortValue(docs[2], "updated_at"),
    );
    const sorted = sortDocuments(docs, "updated_at", "desc");
    expect(sorted.map((d) => d.id)).toEqual(["a", "b", "c"]);
  });

  it("compareDocumentsBySort is stable on ties via title", () => {
    const twins = [
      doc({ id: "z", title: "z.pdf", entity_count: 5 }),
      doc({ id: "a", title: "a.pdf", entity_count: 5 }),
    ];
    expect(compareDocumentsBySort(twins[0], twins[1], "entity_count", "asc")).toBeGreaterThan(
      0,
    );
  });
});

describe("ariaSortForColumn", () => {
  it("marks only the active column", () => {
    expect(ariaSortForColumn("title", "title", "asc")).toBe("ascending");
    expect(ariaSortForColumn("title", "title", "desc")).toBe("descending");
    expect(ariaSortForColumn("cost_usd", "title", "asc")).toBe("none");
  });
});
