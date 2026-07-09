"use client";

/**
 * Document-scoped graph mode — SSOT (SPEC-045, SRP/DRY).
 *
 * First principle: when a document is selected, the graph MUST show only that
 * document's entity subgraph from `GET /lineage/documents/:id` — never the
 * streamed workspace graph.
 */

import { documentLineageToKnowledgeGraph } from "@/lib/graph/document-lineage-to-graph";
import { useGraphStore } from "@/stores/use-graph-store";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { useDocumentLineage } from "./use-lineage";

export interface GraphDocumentScopeResult {
  isDocumentScoped: boolean;
  isLoading: boolean;
  isError: boolean;
  error: unknown;
  refetch: () => void;
}

/**
 * Load and apply document-scoped subgraph; clear stale full-graph state on enter.
 */
export function useGraphDocumentScope(
  documentFilterId: string | null,
): GraphDocumentScopeResult {
  const isDocumentScoped = !!documentFilterId;
  const queryClient = useQueryClient();
  const clearGraphForStreaming = useGraphStore((s) => s.clearGraphForStreaming);
  const setGraph = useGraphStore((s) => s.setGraph);
  const setTruncationInfo = useGraphStore((s) => s.setTruncationInfo);

  const {
    data: lineageData,
    isLoading,
    isError,
    error,
    refetch,
  } = useDocumentLineage(documentFilterId);

  const prevDocumentRef = useRef<string | null>(null);

  // Entering or switching document scope: drop workspace graph + stale query cache.
  useEffect(() => {
    if (prevDocumentRef.current === documentFilterId) return;
    prevDocumentRef.current = documentFilterId;

    if (documentFilterId) {
      clearGraphForStreaming();
      void queryClient.removeQueries({ queryKey: ["graph"] });
    }
  }, [documentFilterId, clearGraphForStreaming, queryClient]);

  // Apply lineage subgraph when response matches the active document filter.
  useEffect(() => {
    if (!documentFilterId || !lineageData) return;
    if (lineageData.document_id !== documentFilterId) return;

    const scopedGraph = documentLineageToKnowledgeGraph(lineageData);
    setGraph(scopedGraph);
    setTruncationInfo(
      false,
      scopedGraph.nodes.length,
      scopedGraph.edges.length,
    );
  }, [documentFilterId, lineageData, setGraph, setTruncationInfo]);

  return {
    isDocumentScoped,
    isLoading,
    isError,
    error,
    refetch: () => {
      void refetch();
    },
  };
}
