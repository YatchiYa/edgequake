"use client";

<<<<<<< HEAD
import {
  getDocumentDisplayStatus,
  isProcessingStatus,
  normalizeStatus,
  StatusBadge,
} from "@/components/documents/status-badge";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { getDocuments } from "@/lib/api/edgequake";
import {
  scopedQueryKey,
  usePipelineWorkspace,
} from "@/lib/pipeline/pipeline-workspace-context";
import { useQuery } from "@tanstack/react-query";
import { FileText, Loader2 } from "lucide-react";

export function PipelineProcessingDocumentsCard() {
  const { selectedTenantId, selectedWorkspaceId } = usePipelineWorkspace();

  const { data: documents, isLoading } = useQuery({
    queryKey: scopedQueryKey("documents", selectedTenantId, selectedWorkspaceId),
    queryFn: () => getDocuments({ page: 1, page_size: 50 }),
    refetchInterval: 2000,
    select: (data) =>
      data.items.filter((doc) =>
        isProcessingStatus(normalizeStatus(doc.status)),
      ),
  });

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center gap-2">
          <FileText className="h-5 w-5" />
          Processing Documents
          {documents && documents.length > 0 && (
            <Badge variant="secondary">{documents.length}</Badge>
          )}
        </CardTitle>
        <CardDescription>Documents currently in the pipeline</CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="flex flex-col justify-center items-center gap-2 py-4">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            <p className="text-sm text-muted-foreground">Loading documents...</p>
          </div>
        ) : documents && documents.length > 0 ? (
=======
import { StatusBadge } from "@/components/documents/status-badge";
import {
  getDocumentDisplayStatus,
  isTerminalStatus,
} from "@/lib/documents/status-domain";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { usePipelineDocuments } from "@/hooks/use-pipeline-documents";
import { isInitialLoading } from "@/lib/layout/cls-stability";
import {
  activeDocumentCount,
  hiddenPreviewCount,
} from "@/lib/pipeline/pipeline-monitor-counts";
import { FileText } from "lucide-react";

export function PipelineProcessingDocumentsCard() {
  const { data, isLoading } = usePipelineDocuments({
    refetchInterval: 2000,
  });
  const cold = isInitialLoading(isLoading, Boolean(data));
  const documents =
    data?.items.filter(
      (doc) => !isTerminalStatus(getDocumentDisplayStatus(doc)),
    ) ?? [];
  const activeCount = activeDocumentCount(data?.status_counts);
  const hiddenActiveCount = hiddenPreviewCount(activeCount, documents.length);

  return (
    <Card data-testid="spec100-pipeline-active-docs">
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center gap-2">
          <FileText className="h-5 w-5" />
          Active Documents
          {activeCount > 0 && <Badge variant="secondary">{activeCount}</Badge>}
        </CardTitle>
        <CardDescription>
          Queued and processing documents
          {hiddenActiveCount > 0
            ? ` · showing ${documents.length} of ${activeCount}`
            : ""}
        </CardDescription>
      </CardHeader>
      <CardContent>
        {cold ? (
          <div className="h-64 space-y-2" data-testid="spec100-pipeline-active-docs-skeleton">
            {Array.from({ length: 4 }).map((_, i) => (
              <div
                key={i}
                className="flex h-14 items-center gap-3 rounded-lg border px-2"
              >
                <Skeleton className="h-4 w-4 shrink-0" />
                <div className="min-w-0 flex-1 space-y-1">
                  <Skeleton className="h-4 w-40" />
                  <Skeleton className="h-3 w-20" />
                </div>
                <Skeleton className="h-5 w-20 rounded-full" />
              </div>
            ))}
          </div>
        ) : documents.length > 0 ? (
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
          <ScrollArea className="h-64">
            <div className="space-y-2">
              {documents.map((doc) => (
                <div
                  key={doc.id}
                  className="flex items-center justify-between p-2 rounded-lg border bg-card"
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <FileText className="h-4 w-4 text-muted-foreground shrink-0" />
                    <div className="min-w-0">
                      <p className="text-sm font-medium truncate">
                        {doc.title || doc.file_name || doc.id.slice(0, 8)}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {doc.content_length
                          ? `${(doc.content_length / 1024).toFixed(1)} KB`
                          : "Unknown size"}
                      </p>
                    </div>
                  </div>
                  <StatusBadge status={getDocumentDisplayStatus(doc)} />
                </div>
              ))}
            </div>
          </ScrollArea>
        ) : (
<<<<<<< HEAD
          <p className="text-sm text-muted-foreground text-center py-4">
            No documents currently processing
          </p>
=======
          <div className="flex h-64 items-center justify-center">
            <p className="text-sm text-muted-foreground text-center">
              No queued or processing documents
            </p>
          </div>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        )}
      </CardContent>
    </Card>
  );
}
