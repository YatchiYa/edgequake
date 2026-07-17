/**
 * Route-level loading UI for /documents.
 * Avoids a blank white first paint while the dashboard shell streams in.
 */
import { Skeleton } from "@/components/ui/skeleton";

export default function DocumentsLoading() {
  return (
    <div
      className="flex h-full min-h-0 flex-col"
      role="status"
      aria-busy="true"
      aria-label="Loading documents page"
      data-testid="documents-route-loading"
    >
      <div className="shrink-0 space-y-3 border-b px-4 py-3">
        <Skeleton className="h-7 w-40" />
        <div className="flex flex-wrap items-center gap-2">
          <Skeleton className="h-9 w-64" />
          <Skeleton className="h-9 w-28" />
          <Skeleton className="h-9 w-28" />
          <Skeleton className="h-9 w-32 ml-auto" />
        </div>
      </div>
      <div className="flex-1 min-h-0 px-4 py-3 space-y-2">
        {[...Array(8)].map((_, i) => (
          <div
            key={i}
            className="flex items-center gap-4 border-b py-3 last:border-b-0"
            aria-hidden="true"
          >
            <Skeleton className="h-4 w-4 shrink-0 rounded" />
            <Skeleton className="h-4 w-48 shrink-0" />
            <Skeleton className="h-5 w-20 rounded-full shrink-0" />
            <Skeleton className="h-4 w-8 shrink-0" />
            <Skeleton className="h-4 w-12 shrink-0" />
            <Skeleton className="h-4 w-24 shrink-0" />
            <Skeleton className="h-6 w-6 rounded-full shrink-0 ml-auto" />
          </div>
        ))}
      </div>
    </div>
  );
}
