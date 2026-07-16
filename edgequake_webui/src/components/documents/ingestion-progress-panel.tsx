/**
 * @module IngestionProgressPanel
 * @description Real-time ingestion progress display with WebSocket support.
 * Based on WebUI Specification Document WEBUI-004 (13-webui-components.md)
 *
 * @implements UC0007 - User monitors document processing progress
 * @implements FEAT0602 - Real-time progress indicators
 * @implements FEAT0760 - Stage-based progress visualization
 * @implements SPEC-003 - Chunk-level resilience with failure visibility
 *
 * @enforces BR0760 - Progress updates at least every 5 seconds
 * @enforces BR0761 - ETA shown when processing active
 */

'use client';

import { CostBadge } from '@/components/documents/cost-badge';
import { FailedChunksCard } from '@/components/documents/failed-chunks-card';
import { EtaDisplay } from '@/components/progress/eta-display';
import { LiveMessage } from '@/components/progress/live-message';
import { StageIndicator, createDefaultStages, type Stage } from '@/components/progress/stage-indicator';
import { AnimatedProgress } from '@/components/shared/animated-progress';
import { WebSocketStatusDot } from '@/components/shared/websocket-status';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useChunkProgress } from '@/hooks/use-chunk-progress';
import { useIngestionProgress } from '@/hooks/use-ingestion-progress';
import { retryFailedChunks } from '@/lib/api/edgequake';
import { cn } from '@/lib/utils';
import type { IngestionStage } from '@/types/ingestion';
import { RefreshCw, X } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';

interface IngestionProgressPanelProps {
  /** Track ID for the ingestion job */
  trackId: string;
  /** Document name to display */
  documentName: string;
  /** Callback when ingestion completes */
  onComplete?: () => void;
  /** Callback when ingestion fails */
  onFailed?: (error: string) => void;
  /** Callback when cancelled */
  onCancel?: () => void;
  /** Show compact variant */
  compact?: boolean;
  /** Custom class name */
  className?: string;
}

/**
 * Displays real-time ingestion progress for a document.
 * 
 * Features:
 * - Stage-by-stage progress visualization
 * - Live cost tracking
 * - ETA estimation
 * - Cancel controls
 * - WebSocket-powered updates with polling fallback
 */
export function IngestionProgressPanel({
  trackId,
  documentName,
  onComplete,
  onFailed,
  onCancel,
  compact = false,
  className,
}: IngestionProgressPanelProps) {
  const { progress, isLive, isLoading, error, cost, cancel, refetch } =
    useIngestionProgress(trackId);
  
  // SPEC-003: Get chunk-level progress including failed chunks
  const { getProgress, getFailedChunks, hasFailedChunks } = useChunkProgress();
  
  // OODA-03: State for chunk retry
  const [isRetrying, setIsRetrying] = useState(false);

  // OODA-03: Handle chunk retry via API
  const handleRetryChunks = useCallback(async (chunkIndices: number[]) => {
    if (!progress?.document_id) return;
    
    setIsRetrying(true);
    try {
      const response = await retryFailedChunks(progress.document_id, chunkIndices);
      // If not implemented, show a toast or log
      if (!response.implemented) {
        console.info('Chunk retry:', response.message);
      }
      // Refresh progress to pick up any changes
      refetch();
    } catch (error) {
      console.error('Failed to retry chunks:', error);
    } finally {
      setIsRetrying(false);
    }
  }, [progress?.document_id, refetch]);

  // Build stages from progress data
  const stages: Stage[] = useMemo(() => {
    if (!progress?.progress?.stages) {
      return createDefaultStages(progress?.progress?.current_stage as IngestionStage);
    }

    const stageOrder: IngestionStage[] = [
      'preprocessing',
      'chunking',
      'extracting',
      'gleaning',
      'merging',
      'summarizing',
      'indexing',
    ];

    return stageOrder.map(stageId => {
      const stageData = progress.progress.stages.find(s => s.stage === stageId);
      const isCurrent = stageId === progress.progress.current_stage;
      
      if (!stageData) {
        return {
          id: stageId,
          label: stageId,
          status: 'pending' as const,
        };
      }

      let status: Stage['status'] = 'pending';
      if (stageData.status === 'completed') {
        status = 'completed';
      } else if (isCurrent) {
        status = 'running';
      } else if (stageData.status === 'failed') {
        status = 'failed';
      }

      return {
        id: stageId,
        label: stageId,
        status,
        progress: isCurrent ? stageData.progress : undefined,
        duration: stageData.duration_ms,
        message: stageData.message,
      };
    });
  }, [progress]);

  // Current stage message
  const currentMessage = useMemo(() => {
    if (!progress?.progress?.stages) return 'Starting...';
    
    const currentStage = progress.progress.current_stage;
    if (!currentStage) return 'Preparing...';
    
    const stageData = progress.progress.stages.find(s => s.stage === currentStage);
    return stageData?.message || `Processing ${currentStage}...`;
  }, [progress]);

  // Handle cancel
  const handleCancel = useCallback(() => {
    cancel();
    onCancel?.();
  }, [cancel, onCancel]);

  // Check if complete / failed
  const isComplete = progress?.status === 'completed';
  const isFailed = progress?.status === 'failed';
  const isProcessing =
    progress?.status === 'processing' || progress?.status === 'pending';

  const completionFiredRef = useRef(false);
  const failureFiredRef = useRef(false);

  // WHY: useEffect (not useMemo) because parent callbacks are side effects.
  useEffect(() => {
    if (isComplete && onComplete && !completionFiredRef.current) {
      completionFiredRef.current = true;
      onComplete();
    }
  }, [isComplete, onComplete]);

  useEffect(() => {
    if (isFailed && onFailed && !failureFiredRef.current) {
      failureFiredRef.current = true;
      const stageError = progress?.progress?.stages?.find((s) => s.status === 'failed');
      onFailed(stageError?.message ?? 'Ingestion failed');
    }
  }, [isFailed, onFailed, progress?.progress?.stages]);

  if (isLoading && !progress) {
    return (
      <Card className={cn('', className)}>
        <CardContent className="py-8 text-center">
          <RefreshCw className="h-6 w-6 animate-spin mx-auto mb-2 text-muted-foreground" />
          <p className="text-sm text-muted-foreground">Loading progress...</p>
        </CardContent>
      </Card>
    );
  }

  if (error && !progress) {
    if (compact) {
      return (
        <div
          className={cn('flex flex-col gap-1 w-full pr-8', className)}
          data-testid="ingestion-progress-load-error"
          role="alert"
        >
          <p className="text-sm font-medium truncate">{documentName}</p>
          <div className="flex items-center gap-2">
            <p className="text-xs text-red-600 dark:text-red-400 truncate flex-1 min-w-0">
              Failed to load progress: {error.message}
            </p>
            <Button
              size="sm"
              variant="outline"
              className="shrink-0 h-7"
              onClick={() => refetch()}
            >
              <RefreshCw className="h-3.5 w-3.5 mr-1" />
              Retry
            </Button>
          </div>
        </div>
      );
    }
    return (
      <Card className={cn('', className)} data-testid="ingestion-progress-load-error">
        <CardContent className="py-6 space-y-2" role="alert">
          <p className="text-sm font-medium">Ingesting: {documentName}</p>
          <p className="text-sm text-red-600 dark:text-red-400">
            Failed to load progress: {error.message}
          </p>
          <Button variant="outline" size="sm" onClick={() => refetch()}>
            <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  // Compact variant — parity with PdfUploadProgress compact row
  if (compact) {
    return (
      <div className={cn('flex flex-col gap-1 w-full pr-8', className)}>
        <div className="flex items-center gap-3">
          {isLive && <WebSocketStatusDot className="shrink-0" />}
          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between mb-1 gap-2">
              <span className="text-sm font-medium truncate">{documentName}</span>
              <span className="text-xs text-muted-foreground shrink-0">
                {Math.round(progress?.overall_progress ?? 0)}%
              </span>
            </div>
            <AnimatedProgress
              value={progress?.overall_progress ?? 0}
              size="sm"
              variant={isFailed ? 'error' : isProcessing ? 'info' : 'default'}
            />
          </div>
        </div>
        {isProcessing && currentMessage && (
          <p className="text-xs text-blue-600 dark:text-blue-400 truncate">
            {currentMessage}
          </p>
        )}
        {isFailed && (
          <div
            className="flex items-center gap-2"
            data-testid="ingestion-progress-compact-error"
            role="alert"
          >
            <p className="text-xs text-red-600 dark:text-red-400 truncate flex-1 min-w-0">
              {currentMessage || 'Ingestion failed'}
            </p>
            <Button
              size="sm"
              variant="outline"
              className="shrink-0 h-7"
              onClick={() => refetch()}
            >
              <RefreshCw className="h-3.5 w-3.5 mr-1" />
              Retry
            </Button>
          </div>
        )}
      </div>
    );
  }

  return (
    <Card className={cn('', className)}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <WebSocketStatusDot />
            <CardTitle className="text-base font-medium">
              Ingesting: {documentName}
            </CardTitle>
          </div>
          
          <div className="flex items-center gap-1">
            {!isLive && (
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={() => refetch()}
                title="Refresh"
              >
                <RefreshCw className="h-4 w-4" />
              </Button>
            )}
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 text-destructive hover:text-destructive"
              onClick={handleCancel}
              title="Cancel"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Overall progress bar */}
        <div className="space-y-1.5">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">Overall</span>
            <div className="flex items-center gap-3">
              <EtaDisplay
                startedAt={progress?.started_at}
                progress={progress?.overall_progress ?? 0}
                isComplete={isComplete}
                size="sm"
              />
              <span className="text-sm font-medium">
                {Math.round(progress?.overall_progress ?? 0)}%
              </span>
            </div>
          </div>
          <AnimatedProgress
            value={progress?.overall_progress ?? 0}
            variant={progress?.status === 'failed' ? 'error' : 'info'}
            size="md"
            animated
          />
        </div>

        {/* Stage indicators */}
        <div className="pt-2">
          <StageIndicator
            stages={stages}
            currentStage={progress?.progress?.current_stage as IngestionStage || 'preprocessing'}
            variant="horizontal"
            showDetails
          />
        </div>

        {/* Live message */}
        <LiveMessage
          message={currentMessage}
          isActive={progress?.status === 'processing'}
        />

        {/* Cost display */}
        <div className="flex items-center justify-between pt-2 border-t">
          <span className="text-sm text-muted-foreground">Cost</span>
          <CostBadge
            cost={cost}
            size="md"
            showBreakdown={cost > 0}
          />
        </div>

        {/* SPEC-003: Display failed chunks if any */}
        {progress?.document_id && hasFailedChunks(progress.document_id) && (
          <FailedChunksCard
            failedChunks={getFailedChunks(progress.document_id)}
            totalChunks={getProgress(progress.document_id)?.totalChunks ?? 0}
            successfulChunks={getProgress(progress.document_id)?.successfulChunks ?? 0}
            onRetry={handleRetryChunks}
            isRetrying={isRetrying}
            className="mt-3"
          />
        )}
      </CardContent>
    </Card>
  );
}

/**
 * Minimal inline progress for table rows.
 */
export function InlineProgress({
  trackId,
  className,
}: {
  trackId: string;
  className?: string;
}) {
  const { progress, isLive } = useIngestionProgress(trackId, {
    autoSubscribe: true,
  });

  return (
    <div className={cn('flex items-center gap-2', className)}>
      {isLive && <WebSocketStatusDot className="shrink-0" />}
      <div className="flex-1 min-w-0 max-w-[100px]">
        <AnimatedProgress
          value={progress?.overall_progress ?? 0}
          size="sm"
          variant={progress?.status === 'failed' ? 'error' : 'default'}
        />
      </div>
      <span className="text-xs text-muted-foreground shrink-0">
        {Math.round(progress?.overall_progress ?? 0)}%
      </span>
    </div>
  );
}

export default IngestionProgressPanel;
