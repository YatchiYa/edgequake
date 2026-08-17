import type { DocumentStatusCounts } from "@/types";

export function activeDocumentCount(counts?: DocumentStatusCounts): number {
  return (counts?.pending ?? 0) + (counts?.processing ?? 0);
}

export function hiddenPreviewCount(
  aggregateCount: number,
  visibleRows: number,
): number {
  return Math.max(0, aggregateCount - visibleRows);
}
