import { describe, expect, it } from "bun:test";

import {
  getQueryEmptyCopy,
  isChatQueryMode,
} from "@/lib/query/query-empty-copy";

describe("query-empty-copy Chat mode", () => {
  it("uses chatbot copy for bypass and not KG framing", () => {
    expect(isChatQueryMode("bypass")).toBe(true);
    const copy = getQueryEmptyCopy("bypass");
    expect(copy.title).toBe("Chat with your assistant");
    expect(copy.description.toLowerCase()).toContain("without document");
    expect(copy.suggestions.length).toBe(4);
    expect(copy.title.toLowerCase()).not.toContain("knowledge graph");
  });

  it("keeps KG copy for RAG modes", () => {
    expect(isChatQueryMode("mix")).toBe(false);
    const copy = getQueryEmptyCopy("mix");
    expect(copy.title).toContain("knowledge graph");
  });
});
