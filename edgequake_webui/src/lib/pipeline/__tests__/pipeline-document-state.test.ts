import { describe, expect, it } from 'bun:test';
import type { Document } from '@/types';
import {
  isActiveProcessingStatus,
  isWaitingStatus,
  orphanQueuedTaskCount,
  summarizePipelineDocuments,
} from '../pipeline-document-state';

function doc(overrides: Partial<Document> & { id: string }): Document {
  return {
    title: overrides.id,
    file_name: `${overrides.id}.pdf`,
    status: 'completed',
    ...overrides,
  } as Document;
}

describe('pipeline-document-state', () => {
  it('classifies pending as waiting, not active', () => {
    expect(isWaitingStatus('pending')).toBe(true);
    expect(isActiveProcessingStatus('pending')).toBe(false);
  });

  it('classifies extracting as active processing', () => {
    expect(isWaitingStatus('extracting')).toBe(false);
    expect(isActiveProcessingStatus('extracting')).toBe(true);
  });

  it('summarizes active vs waiting documents', () => {
    const summary = summarizePipelineDocuments([
      doc({ id: 'a', current_stage: 'extracting' }),
      doc({ id: 'b', current_stage: 'pending', stage_message: 'Auto-recovered' }),
      doc({ id: 'c', status: 'completed' }),
    ]);

    expect(summary.activeCount).toBe(1);
    expect(summary.waitingCount).toBe(1);
    expect(summary.activeDocs.map((d) => d.id)).toEqual(['a']);
    expect(summary.waitingDocs.map((d) => d.id)).toEqual(['b']);
  });

  it('computes orphan queued tasks not tied to waiting docs', () => {
    expect(orphanQueuedTaskCount(3, 1)).toBe(2);
    expect(orphanQueuedTaskCount(1, 2)).toBe(0);
  });
});
