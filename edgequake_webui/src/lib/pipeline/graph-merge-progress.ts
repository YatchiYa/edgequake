/**
 * Parse and format graph-merge stage messages from the backend.
 *
 * WHY: Backend emits operator-oriented counters during chunked relationship
 * merge (SPEC-045). UI must show live progress instead of frozen 0/N.
 */

import type { Document } from '@/types';

export interface GraphMergeCounters {
  subPhase: string;
  entitiesProcessed: number;
  entitiesTotal: number;
  entitiesPercent: number;
  relationshipsProcessed: number;
  relationshipsTotal: number;
  relationshipsPercent: number;
}

const GRAPH_MERGE_RE =
  /Storing in knowledge graph — (\w+) \((\d+)\/(\d+) entities \((\d+)%\), (\d+)\/(\d+) relationships \((\d+)%\)/;

export function parseGraphMergeStageMessage(
  message: string,
): GraphMergeCounters | null {
  const match = message.trim().match(GRAPH_MERGE_RE);
  if (!match) {
    return null;
  }

  return {
    subPhase: match[1],
    entitiesProcessed: Number(match[2]),
    entitiesTotal: Number(match[3]),
    entitiesPercent: Number(match[4]),
    relationshipsProcessed: Number(match[5]),
    relationshipsTotal: Number(match[6]),
    relationshipsPercent: Number(match[7]),
  };
}

function formatCount(value: number): string {
  return value.toLocaleString();
}

/** Primary counter line during graph merge (entities vs relationships). */
export function primaryGraphMergeCounter(
  counters: GraphMergeCounters,
): { label: string; processed: number; total: number; percent: number } {
  const entitiesDone =
    counters.entitiesTotal === 0 ||
    counters.entitiesProcessed >= counters.entitiesTotal;
  const relationshipsActive =
    counters.relationshipsTotal > 0 &&
    counters.relationshipsProcessed < counters.relationshipsTotal;

  if (entitiesDone && relationshipsActive) {
    return {
      label: 'relationships',
      processed: counters.relationshipsProcessed,
      total: counters.relationshipsTotal,
      percent: counters.relationshipsPercent,
    };
  }

  if (counters.entitiesTotal > 0) {
    return {
      label: 'entities',
      processed: counters.entitiesProcessed,
      total: counters.entitiesTotal,
      percent: counters.entitiesPercent,
    };
  }

  return {
    label: 'relationships',
    processed: counters.relationshipsProcessed,
    total: counters.relationshipsTotal,
    percent: counters.relationshipsPercent,
  };
}

/** User-facing detail for banner / dialog (includes file name). */
export function formatGraphMergeUserDetail(
  fileName: string,
  counters: GraphMergeCounters,
): string {
  const primary = primaryGraphMergeCounter(counters);
  const step =
    primary.label === 'relationships' ? 'Saving relationships' : 'Saving entities';

  if (primary.total === 0) {
    return `${fileName}: Saving to knowledge graph…`;
  }

  return `${fileName}: ${step} — ${formatCount(primary.processed)}/${formatCount(primary.total)} (${primary.percent}%)`;
}

/** Shorter line for status badges / tooltips (no file name). */
export function formatGraphMergeStageMessage(message: string): string | null {
  const counters = parseGraphMergeStageMessage(message);
  if (!counters) {
    return null;
  }

  const primary = primaryGraphMergeCounter(counters);
  const step =
    primary.label === 'relationships' ? 'Saving relationships' : 'Saving entities';

  if (primary.total === 0) {
    return 'Saving to knowledge graph…';
  }

  return `${step}: ${formatCount(primary.processed)}/${formatCount(primary.total)} (${primary.percent}%)`;
}

/** Highest fractional progress among active documents (0–1). */
export function resolveBannerStageProgress(
  documents: Document[],
): number | undefined {
  return resolveBannerProgressMeta(documents)?.progress01;
}

const GRAPH_SAVE_STAGES = new Set([
  'merging',
  'storing',
  'indexing',
]);

const EXTRACTION_STAGES = new Set([
  'extracting',
  'gleaning',
  'chunking',
]);

/** i18n key for banner progress label — stage-specific (SPEC-048 polish). */
export function bannerProgressLabelKey(stage?: string | null): string {
  const s = (stage || '').toLowerCase();
  if (GRAPH_SAVE_STAGES.has(s)) return 'pipeline.graphMergeProgress';
  if (EXTRACTION_STAGES.has(s)) return 'pipeline.extractionProgress';
  if (s === 'embedding') return 'pipeline.embeddingProgress';
  if (s === 'converting' || s === 'preprocessing' || s === 'uploading') {
    return 'pipeline.conversionProgress';
  }
  return 'pipeline.stageProgress';
}

export interface BannerProgressMeta {
  progress01: number;
  stage: string;
  labelKey: string;
}

/**
 * Pick the best determinate progress for the banner, with a stage-aware label.
 * Prefer the document that owns the max progress so the label matches the bar.
 */
export function resolveBannerProgressMeta(
  documents: Document[],
): BannerProgressMeta | undefined {
  let best: BannerProgressMeta | undefined;

  for (const doc of documents) {
    const value = doc.stage_progress;
    if (typeof value !== 'number' || value <= 0) {
      continue;
    }
    const stage = (doc.current_stage || doc.status || '').toLowerCase();
    if (!best || value > best.progress01) {
      best = {
        progress01: value,
        stage,
        labelKey: bannerProgressLabelKey(stage),
      };
    }
  }

  return best;
}
