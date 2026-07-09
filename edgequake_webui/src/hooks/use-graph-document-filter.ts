"use client";

import { useGraphStore } from "@/stores/use-graph-store";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useCallback, useLayoutEffect } from "react";

/**
 * Sync graph document filter with `?document=` (and legacy `?highlight=`) URL params.
 *
 * URL is the synchronous source of truth on first paint so document-scoped mode
 * activates before graph streaming can load the full workspace graph.
 */
export function useGraphDocumentFilterUrl() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const storeDocumentFilterId = useGraphStore((s) => s.documentFilterId);
  const setDocumentFilterId = useGraphStore((s) => s.setDocumentFilterId);

  const urlDocumentId =
    searchParams.get("document") ?? searchParams.get("highlight");

  const documentFilterId = urlDocumentId ?? storeDocumentFilterId;

  // Sync store before paint so streaming does not start for deep-linked documents.
  useLayoutEffect(() => {
    const next = documentFilterId ?? null;
    if (next !== storeDocumentFilterId) {
      setDocumentFilterId(next);
    }
  }, [documentFilterId, storeDocumentFilterId, setDocumentFilterId]);

  const setDocumentFilter = useCallback(
    (documentId: string | null) => {
      setDocumentFilterId(documentId);
      const params = new URLSearchParams(searchParams.toString());
      params.delete("highlight");
      if (documentId) {
        params.set("document", documentId);
      } else {
        params.delete("document");
      }
      const qs = params.toString();
      router.replace(qs ? `${pathname}?${qs}` : pathname);
    },
    [pathname, router, searchParams, setDocumentFilterId],
  );

  return {
    documentFilterId,
    setDocumentFilter,
  };
}
