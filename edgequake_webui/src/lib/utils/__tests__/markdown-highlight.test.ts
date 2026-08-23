import { describe, expect, it } from "vitest";
import {
  findChunkIndexForRange,
  findHighlightIndex,
  localHighlightForSlice,
  offsetOfLine,
  resolveMarkdownHighlightRange,
} from "../markdown-highlight";
import { splitMarkdownIntoChunks } from "@/components/query/markdown/VirtualizedMarkdownContent";

describe("resolveMarkdownHighlightRange", () => {
  const content = [
    "# Title",
    "",
    "## 2.1 Challenges in the Prefill Stage: Transfer and Recomputation Costs",
    "",
    "Prefill determines TTFT of every agent turn.",
  ].join("\n");

  it("prefers explicit line range over highlight text", () => {
    expect(
      resolveMarkdownHighlightRange({
        content,
        startLine: 3,
        endLine: 5,
        highlightText: "Title",
      }),
    ).toEqual({ startLine: 3, endLine: 5 });
  });

  it("maps truncated citation highlight text to the heading lines", () => {
    const highlight =
      "## 2.1 Challenges in the Prefill Stage: Transfer and Recomputation Costs\n\nPrefill determines TTFT of";
    expect(
      resolveMarkdownHighlightRange({ content, highlightText: highlight }),
    ).toEqual({ startLine: 3, endLine: 5 });
  });

  it("returns undefined when neither lines nor text match", () => {
    expect(
      resolveMarkdownHighlightRange({
        content,
        highlightText: "zzzz-not-in-doc",
      }),
    ).toBeUndefined();
  });
});

describe("findHighlightIndex", () => {
  it("rejects short needles", () => {
    expect(findHighlightIndex("abcdef", "abc")).toBe(-1);
  });
});

describe("virtualized slice mapping", () => {
  it("maps a heading in a later 25k slice to a non-zero chunk index", () => {
    const prefix = "x".repeat(26_000);
    const heading = "## 2.1 Challenges in the Prefill Stage: Transfer and Recomputation Costs\n";
    const content = `${prefix}\n${heading}body\n`;
    const chunks = splitMarkdownIntoChunks(content);
    expect(chunks.length).toBeGreaterThan(1);
    const range = resolveMarkdownHighlightRange({
      content,
      highlightText: heading.trim(),
    });
    expect(range).toBeDefined();
    const index = findChunkIndexForRange(chunks, content, range!);
    expect(index).toBeGreaterThan(0);
    const sliceStart = chunks.slice(0, index).reduce((n, c) => n + c.length, 0);
    const local = localHighlightForSlice(chunks[index], sliceStart, content, range!);
    expect(local).toBeDefined();
    expect(local!.startLine).toBeGreaterThanOrEqual(1);
  });

  it("offsetOfLine is 0 for line 1", () => {
    expect(offsetOfLine("a\nb", 1)).toBe(0);
    expect(offsetOfLine("a\nb", 2)).toBe(2);
  });
});
