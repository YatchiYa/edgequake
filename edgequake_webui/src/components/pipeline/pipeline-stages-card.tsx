"use client";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
<<<<<<< HEAD
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  getDocuments,
  getEnhancedPipelineStatus,
  requestPipelineCancellation,
} from "@/lib/api/edgequake";
import { countDocumentsByPhase } from "@/lib/pipeline/pipeline-formatters";
=======
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  getEnhancedPipelineStatus,
  requestPipelineCancellation,
} from "@/lib/api/edgequake";
import { usePipelineDocuments } from "@/hooks/use-pipeline-documents";
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { PIPELINE_PHASES } from "@/lib/pipeline/pipeline-phases";
import {
  scopedQueryKey,
  usePipelineWorkspace,
} from "@/lib/pipeline/pipeline-workspace-context";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
<<<<<<< HEAD
import {
  CheckCircle,
  Clock,
  Layers,
  Loader2,
  StopCircle,
} from "lucide-react";
import { useMemo } from "react";
=======
import { CheckCircle, Clock, Layers, Loader2, StopCircle } from "lucide-react";
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { toast } from "sonner";

export function PipelineStagesCard() {
  const { selectedTenantId, selectedWorkspaceId } = usePipelineWorkspace();
  const queryClient = useQueryClient();

<<<<<<< HEAD
  const { data: documents } = useQuery({
    queryKey: scopedQueryKey("documents", selectedTenantId, selectedWorkspaceId),
    queryFn: () => getDocuments({ page: 1, page_size: 100 }),
    refetchInterval: 3000,
    select: (data) => data.items,
=======
  const { data: documents } = usePipelineDocuments({
    refetchInterval: 3000,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  });

  const { data: status } = useQuery({
    queryKey: scopedQueryKey(
      "enhanced-pipeline-status",
      selectedTenantId,
      selectedWorkspaceId,
    ),
    queryFn: () =>
      getEnhancedPipelineStatus(
        selectedTenantId ?? undefined,
        selectedWorkspaceId ?? undefined,
      ),
    refetchInterval: 2000,
  });

  const cancelMutation = useMutation({
    mutationFn: requestPipelineCancellation,
    onSuccess: () => {
      toast.success("Pipeline cancellation requested");
      queryClient.invalidateQueries({
        queryKey: scopedQueryKey(
          "enhanced-pipeline-status",
          selectedTenantId,
          selectedWorkspaceId,
        ),
      });
    },
    onError: (error) => {
      toast.error(
        `Cancel failed: ${error instanceof Error ? error.message : "Unknown"}`,
      );
    },
  });

<<<<<<< HEAD
  const phaseCounts = useMemo(
    () => countDocumentsByPhase(documents?.map((doc) => doc.status) ?? []),
    [documents],
  );

  const totalDocs = documents?.length || 0;
=======
  const phaseCounts = documents?.status_counts ?? {
    pending: 0,
    processing: 0,
    completed: 0,
    partial_failure: 0,
    failed: 0,
    cancelled: 0,
  };
  const totalDocs = documents?.total ?? 0;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  const isActive = phaseCounts.processing > 0 || phaseCounts.pending > 0;

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg flex items-center gap-2">
              <Layers className="h-5 w-5" />
              Document Pipeline
            </CardTitle>
<<<<<<< HEAD
            <CardDescription>{totalDocs} documents in workspace</CardDescription>
=======
            <CardDescription>
              {totalDocs} documents in workspace
            </CardDescription>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
          </div>
          {status?.is_busy && (
            <Button
              variant="destructive"
              size="sm"
              onClick={() => cancelMutation.mutate()}
<<<<<<< HEAD
              disabled={cancelMutation.isPending || status.cancellation_requested}
=======
              disabled={
                cancelMutation.isPending || status.cancellation_requested
              }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            >
              {cancelMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <StopCircle className="mr-2 h-4 w-4" />
              )}
              {status.cancellation_requested ? "Cancelling..." : "Cancel"}
            </Button>
          )}
          {!status?.is_busy && isActive && (
<<<<<<< HEAD
            <Badge variant="outline" className="text-yellow-500 border-yellow-500">
=======
            <Badge
              variant="outline"
              className="text-yellow-500 border-yellow-500"
            >
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
              <Clock className="h-3 w-3 mr-1" />
              Queued
            </Badge>
          )}
          {!status?.is_busy && !isActive && totalDocs > 0 && (
<<<<<<< HEAD
            <Badge variant="outline" className="text-green-500 border-green-500">
=======
            <Badge
              variant="outline"
              className="text-green-500 border-green-500"
            >
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
              <CheckCircle className="h-3 w-3 mr-1" />
              Idle
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {PIPELINE_PHASES.map((phase) => {
<<<<<<< HEAD
            const count = phaseCounts[phase.key as keyof typeof phaseCounts] || 0;
=======
            const count =
              phaseCounts[phase.key as keyof typeof phaseCounts] || 0;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            const Icon = phase.icon;
            const isActivePhase = count > 0;

            return (
              <div
                key={phase.key}
                className={`flex flex-col items-center p-4 rounded-lg border-2 transition-all ${
                  isActivePhase ? phase.bgColor : "bg-muted/50 border-muted"
                }`}
              >
                <Icon
                  className={`h-6 w-6 ${
                    isActivePhase ? phase.color : "text-muted-foreground"
                  } ${
                    phase.key === "processing" && isActivePhase
                      ? "animate-pulse"
                      : ""
                  }`}
                />
                <span
                  className={`text-sm font-medium mt-2 ${
                    isActivePhase ? phase.color : "text-muted-foreground"
                  }`}
                >
                  {phase.label}
                </span>
                <span
                  className={`text-2xl font-bold ${
                    isActivePhase ? phase.color : "text-muted-foreground"
                  }`}
                >
                  {count}
                </span>
              </div>
            );
          })}
        </div>

        {totalDocs > 0 && (
          <div className="mt-4">
            <div className="flex justify-between text-xs text-muted-foreground mb-1">
              <span>Pipeline Progress</span>
              <span>
                {phaseCounts.completed} / {totalDocs} completed
              </span>
            </div>
            <Progress
<<<<<<< HEAD
              value={totalDocs > 0 ? (phaseCounts.completed / totalDocs) * 100 : 0}
=======
              value={
                totalDocs > 0 ? (phaseCounts.completed / totalDocs) * 100 : 0
              }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
              className="h-2"
            />
          </div>
        )}
      </CardContent>
    </Card>
  );
}
