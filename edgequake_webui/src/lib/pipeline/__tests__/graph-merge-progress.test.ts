import { describe, expect, it } from 'bun:test';
import type { Document } from '@/types';
import {
  bannerProgressLabelKey,
  formatGraphMergeStageMessage,
  formatGraphMergeUserDetail,
  parseGraphMergeStageMessage,
  primaryGraphMergeCounter,
  resolveBannerProgressMeta,
  resolveBannerStageProgress,
} from '../graph-merge-progress';

const BACKEND_MSG =
  'Storing in knowledge graph — RelationshipGraph (2654/2654 entities (100%), 128/1999 relationships (6%))';

describe('graph-merge-progress', () => {
  it('parses backend graph merge stage message', () => {
    const parsed = parseGraphMergeStageMessage(BACKEND_MSG);
    expect(parsed).not.toBeNull();
    expect(parsed?.subPhase).toBe('RelationshipGraph');
    expect(parsed?.entitiesProcessed).toBe(2654);
    expect(parsed?.entitiesTotal).toBe(2654);
    expect(parsed?.relationshipsProcessed).toBe(128);
    expect(parsed?.relationshipsTotal).toBe(1999);
    expect(parsed?.relationshipsPercent).toBe(6);
  });

  it('returns null for unrelated stage messages', () => {
    expect(parseGraphMergeStageMessage('Extracting entities...')).toBeNull();
  });

  it('prioritizes relationship counter when entities are complete', () => {
    const parsed = parseGraphMergeStageMessage(BACKEND_MSG)!;
    const primary = primaryGraphMergeCounter(parsed);
    expect(primary.label).toBe('relationships');
    expect(primary.processed).toBe(128);
    expect(primary.total).toBe(1999);
  });

  it('formats user-facing detail with file name', () => {
    const parsed = parseGraphMergeStageMessage(BACKEND_MSG)!;
    const detail = formatGraphMergeUserDetail('deep_2604.pdf', parsed);
    expect(detail).toContain('deep_2604.pdf');
    expect(detail).toContain('Saving relationships');
    expect(detail).toContain('128');
    expect(detail).toContain('1,999');
    expect(detail).toContain('(6%)');
  });

  it('formats compact stage message for badges', () => {
    const compact = formatGraphMergeStageMessage(BACKEND_MSG);
    expect(compact).toBe('Saving relationships: 128/1,999 (6%)');
  });

  it('resolves max stage_progress across active documents', () => {
    const docs = [
      { id: 'a', stage_progress: 0.12 } as Document,
      { id: 'b', stage_progress: 0.67 } as Document,
      { id: 'c', stage_progress: 0 } as Document,
    ];
    expect(resolveBannerStageProgress(docs)).toBe(0.67);
  });

  it('gates banner progress label by stage (SPEC-048)', () => {
    expect(bannerProgressLabelKey('extracting')).toBe('pipeline.extractionProgress');
    expect(bannerProgressLabelKey('storing')).toBe('pipeline.graphMergeProgress');
    expect(bannerProgressLabelKey('embedding')).toBe('pipeline.embeddingProgress');

    const meta = resolveBannerProgressMeta([
      {
        id: 'a',
        current_stage: 'extracting',
        stage_progress: 0.12,
      } as Document,
    ]);
    expect(meta?.labelKey).toBe('pipeline.extractionProgress');
    expect(meta?.progress01).toBe(0.12);
  });
});
