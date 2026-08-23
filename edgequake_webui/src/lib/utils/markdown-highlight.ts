/**
 * Resolve citation highlight intent to a 1-based inclusive markdown line range.
 *
 * SSOT for document-detail and query deeplinks (SPEC-033 / SPEC-135):
 * explicit `start_line`/`end_line` wins; otherwise match `highlight` text.
 */

export type HighlightLineRange = {
  startLine: number;
  endLine: number;
};

const MIN_HIGHLIGHT_CHARS = 10;

export function offsetOfLine(content: string, line: number): number {
  if (line <= 1) return 0;
  let seen = 1;
  for (let i = 0; i < content.length; i++) {
    if (content[i] === "\n") {
      seen += 1;
      if (seen === line) return i + 1;
    }
  }
  return content.length;
}

export function lineAtOffset(content: string, offset: number): number {
  const clamped = Math.max(0, Math.min(offset, content.length));
  let line = 1;
  for (let i = 0; i < clamped; i++) {
    if (content[i] === "\n") line += 1;
  }
  return line;
}

/** Index of `searchText` in `content`, or -1. */
export function findHighlightIndex(content: string, searchText: string): number {
  if (!searchText || searchText.trim().length < MIN_HIGHLIGHT_CHARS) return -1;

  const normalizedContent = content.toLowerCase();
  const normalizedSearch = searchText.toLowerCase().trim();

  let matchIndex = normalizedContent.indexOf(normalizedSearch);
  if (matchIndex === -1 && normalizedSearch.length > 50) {
    matchIndex = normalizedContent.indexOf(normalizedSearch.slice(0, 50));
  }
  if (matchIndex === -1) {
    const words = normalizedSearch.split(/\s+/).filter((w) => w.length > 4);
    for (const word of words.slice(0, 3)) {
      matchIndex = normalizedContent.indexOf(word);
      if (matchIndex !== -1) break;
    }
  }
  return matchIndex;
}

export function resolveMarkdownHighlightRange({
  content,
  startLine,
  endLine,
  highlightText,
}: {
  content: string;
  startLine?: number;
  endLine?: number;
  highlightText?: string;
}): HighlightLineRange | undefined {
  if (
    startLine !== undefined &&
    endLine !== undefined &&
    startLine >= 1 &&
    endLine >= startLine
  ) {
    return { startLine, endLine };
  }

  if (!highlightText || !content) return undefined;
  const matchIndex = findHighlightIndex(content, highlightText);
  if (matchIndex < 0) return undefined;

  const matchLen = Math.min(Math.max(highlightText.trim().length, 1), content.length - matchIndex);
  const start = lineAtOffset(content, matchIndex);
  const end = lineAtOffset(content, matchIndex + Math.max(matchLen - 1, 0));
  return { startLine: start, endLine: Math.max(start, end) };
}

/**
 * Map a document-absolute line range onto one virtualized markdown slice.
 * Returns chunk-local 1-based lines, or undefined when the slice does not overlap.
 */
export function localHighlightForSlice(
  slice: string,
  sliceStart: number,
  fullContent: string,
  range: HighlightLineRange,
): HighlightLineRange | undefined {
  if (!slice) return undefined;
  const globalStart = offsetOfLine(fullContent, range.startLine);
  const globalEndExclusive = offsetOfLine(fullContent, range.endLine + 1);
  const sliceEnd = sliceStart + slice.length;
  const overlapStart = Math.max(globalStart, sliceStart);
  const overlapEnd = Math.min(globalEndExclusive, sliceEnd);
  if (overlapStart >= overlapEnd) return undefined;

  const localStart = lineAtOffset(slice, overlapStart - sliceStart);
  const localEnd = lineAtOffset(slice, Math.max(overlapStart, overlapEnd - 1) - sliceStart);
  return { startLine: localStart, endLine: Math.max(localStart, localEnd) };
}

export function findChunkIndexForRange(
  chunks: string[],
  fullContent: string,
  range: HighlightLineRange,
): number {
  const target = offsetOfLine(fullContent, range.startLine);
  let cursor = 0;
  for (let i = 0; i < chunks.length; i++) {
    const next = cursor + chunks[i].length;
    if (target >= cursor && target < next) return i;
    if (target === next && i === chunks.length - 1) return i;
    cursor = next;
  }
  return 0;
}
