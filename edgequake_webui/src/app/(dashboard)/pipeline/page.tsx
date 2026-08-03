/**
 * @module PipelinePage
 * @description Dedicated page for monitoring document ingestion pipeline.
 *
 * @implements FEAT0004 - Processing status tracking
 * @implements SPEC-100 — CLS shell
 */
import { PipelineMonitor } from '@/components/pipeline/pipeline-monitor';

export default function PipelinePage() {
  return (
    <div className="h-full min-h-0 overflow-clip" data-testid="pipeline-page">
      <PipelineMonitor />
    </div>
  );
}
