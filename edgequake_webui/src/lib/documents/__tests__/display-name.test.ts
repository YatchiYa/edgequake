import { QueryClient } from "@tanstack/react-query";
import { describe, expect, it } from "vitest";

import {
  documentDetailQueryKey,
  findCachedDocumentLabel,
  resolveDocumentDisplayName,
} from "../display-name";

const DOC_ID = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";

describe("resolveDocumentDisplayName", () => {
  it("prefers title over file_name", () => {
    expect(
      resolveDocumentDisplayName({
        id: DOC_ID,
        title: "Report.pdf",
        file_name: "raw.bin",
      }),
    ).toBe("Report.pdf");
  });

  it("falls back to file_name when title missing", () => {
    expect(
      resolveDocumentDisplayName({
        id: DOC_ID,
        title: null,
        file_name: "notes.md",
      }),
    ).toBe("notes.md");
  });

  it("skips title that equals the document id", () => {
    expect(
      resolveDocumentDisplayName({
        id: DOC_ID,
        title: DOC_ID,
        file_name: "real-name.pdf",
      }),
    ).toBe("real-name.pdf");
  });

  it("never returns a full GUID as last resort", () => {
    const name = resolveDocumentDisplayName({ id: DOC_ID });
    expect(name).toBe(`Document ${DOC_ID.slice(0, 8)}`);
    expect(name).not.toBe(DOC_ID);
  });
});

describe("findCachedDocumentLabel", () => {
  it("reads title from search cache", () => {
    const qc = new QueryClient();
    qc.setQueryData(["documents", "search", "", 50], {
      items: [{ id: DOC_ID, title: "from-search.pdf", status: "completed" }],
      total: 1,
      has_more: false,
    });
    expect(findCachedDocumentLabel(qc, DOC_ID)).toBe("from-search.pdf");
  });

  it("reads from workspace-scoped list cache (prefix match)", () => {
    const qc = new QueryClient();
    qc.setQueryData(
      ["documents", "tenant-1", "ws-1", 1, 20, "all"],
      {
        items: [
          {
            id: DOC_ID,
            title: "scoped-list.pdf",
            file_name: "scoped-list.pdf",
          },
        ],
      },
    );
    expect(findCachedDocumentLabel(qc, DOC_ID)).toBe("scoped-list.pdf");
  });

  it("prefers detail cache over list", () => {
    const qc = new QueryClient();
    qc.setQueryData(["documents", "tenant-1", "ws-1", 1, 20, "all"], {
      items: [{ id: DOC_ID, title: "list.pdf" }],
    });
    qc.setQueryData(documentDetailQueryKey(DOC_ID), {
      id: DOC_ID,
      title: "detail.pdf",
    });
    expect(findCachedDocumentLabel(qc, DOC_ID)).toBe("detail.pdf");
  });

  it("returns undefined when nothing cached", () => {
    const qc = new QueryClient();
    expect(findCachedDocumentLabel(qc, DOC_ID)).toBeUndefined();
  });
});
