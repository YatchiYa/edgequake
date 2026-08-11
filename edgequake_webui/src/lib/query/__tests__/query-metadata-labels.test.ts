import { describe, expect, test } from "bun:test";
import { getQueryModeMeta, QUERY_MODE_META } from "@/lib/query/query-mode-meta";
import type { QueryMode } from "@/types/query";

/**
 * Metadata bar shows UX mode labels (Focused, Broad, …) next to tokens/sec.
 */
describe("query response metadata labels", () => {
  test("maps API modes to UX labels used beside tokens/sec", () => {
    const expected: Record<QueryMode, string> = {
      local: "Focused",
      global: "Broad",
      hybrid: "Linked",
      mix: "Smart",
      naive: "Chunks",
      bypass: "Chat",
    };

    for (const meta of QUERY_MODE_META) {
      expect(getQueryModeMeta(meta.id).label).toBe(expected[meta.id]);
    }
  });

  test("formats provider/model lineage label", () => {
    const format = (provider?: string, model?: string) =>
      provider && model
        ? `${provider}/${model}`
        : provider || model || undefined;

    expect(format("ollama", "gemma3:latest")).toBe("ollama/gemma3:latest");
    expect(format("openai", undefined)).toBe("openai");
    expect(format(undefined, "gpt-5-nano")).toBe("gpt-5-nano");
    expect(format(undefined, undefined)).toBeUndefined();
  });
});
