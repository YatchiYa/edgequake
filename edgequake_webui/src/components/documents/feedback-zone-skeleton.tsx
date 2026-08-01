/**
 * Reserved footprint for the feedback zone while live work is expected
 * but not yet painted (SPEC-099 CLS / refresh stability).
 */
"use client";

import { Skeleton } from "@/components/ui/skeleton";
import { FEEDBACK_ZONE_RESERVE_MIN_PX } from "@/lib/documents/documents-layout-stability";

export function FeedbackZoneSkeleton() {
  return (
    <div
      className="rounded-md border border-sky-200/80 bg-sky-50/40 p-2 dark:border-sky-900 dark:bg-sky-950/20"
      style={{ minHeight: FEEDBACK_ZONE_RESERVE_MIN_PX }}
      data-testid="spec099-feedback-zone-skeleton"
      role="status"
      aria-busy="true"
      aria-label="Loading active runs"
    >
      <div className="mb-2 flex items-center justify-between gap-2">
        <Skeleton className="h-4 w-28" />
        <Skeleton className="h-3 w-6" />
      </div>
      <Skeleton className="mb-2 h-2 w-full rounded-full" />
      <div className="flex gap-2">
        <Skeleton className="h-3 w-14" />
        <Skeleton className="h-3 w-14" />
        <Skeleton className="h-3 w-14" />
        <Skeleton className="h-3 w-16" />
      </div>
    </div>
  );
}

export default FeedbackZoneSkeleton;
