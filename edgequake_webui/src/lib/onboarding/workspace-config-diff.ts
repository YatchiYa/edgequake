/**
 * SPEC-101 LAW-101-12 — Pure workspace config diff + rebuild hints (unit-tested).
 */

import type { PdfParserBackendDraft } from '@/lib/onboarding/wizard-state';
import type { ModelSelectionSlash } from '@/lib/onboarding/model-payload';

export type WorkspaceConfigChangedKey =
  | 'llm'
  | 'embedding'
  | 'vision'
  | 'pdfParser'
  | 'extractionLanguage'
  | 'chunking'
  | 'extractBudget'
  | 'entityTypes'
  | 'entityTypesStrict'
  | 'entityTypeColors'
  | 'relationTypes'
  | 'relationTypesStrict'
  | 'kgSchemaPreset'
  | 'relationEdges';

export interface WorkspaceConfigSnapshot {
  useServerDefaults: boolean;
  llm?: ModelSelectionSlash;
  embedding?: ModelSelectionSlash;
  vision?: ModelSelectionSlash;
  pdfParserBackend: PdfParserBackendDraft;
  extractionLanguage: string | null;
  chunkingMode?: 'inherit' | 'adaptive' | 'fixed' | null;
  chunkTokenSize?: number | null;
  chunkOverlapTokenSize?: number | null;
  extractBudgetMode?: 'inherit' | 'custom' | null;
  extractMaxEntities?: number | null;
  extractMaxRecords?: number | null;
  entityTypes: string[];
  entityTypesStrict: boolean;
  entityTypeColors?: Record<string, string>;
  relationTypes?: string[];
  relationTypesStrict?: boolean;
  kgSchemaPreset?: string;
  relationEdges?: Array<{ source: string; relation: string; target: string }>;
}

export interface WorkspaceRebuildHints {
  embeddings: boolean;
  extraction: boolean;
  vision: boolean;
}

export interface WorkspaceConfigDiff {
  changedKeys: WorkspaceConfigChangedKey[];
  rebuildHints: WorkspaceRebuildHints;
  hasChanges: boolean;
}

function selectionKey(sel?: ModelSelectionSlash): string {
  if (!sel?.provider || !sel?.model) return '';
  const dim =
    typeof sel.dimension === 'number' && sel.dimension > 0 ? `:${sel.dimension}` : '';
  return `${sel.provider}/${sel.model}${dim}`;
}

function languageKey(lang: string | null): string {
  return lang ?? '';
}

function entityTypesKey(types: string[]): string {
  return [...types].map((t) => t.trim()).filter(Boolean).sort().join('|');
}

function entityTypeColorsKey(colors?: Record<string, string>): string {
  if (!colors) return '';
  return Object.entries(colors)
    .map(([k, v]) => `${k.toUpperCase()}=${v.toLowerCase()}`)
    .sort()
    .join('|');
}

function relationEdgesKey(
  edges?: Array<{ source: string; relation: string; target: string }>,
): string {
  if (!edges?.length) return '';
  return edges
    .map(
      (e) =>
        `${e.source.trim().toUpperCase()}|${e.relation.trim().toUpperCase()}|${e.target.trim().toUpperCase()}`,
    )
    .sort()
    .join(';');
}

function modelsDiffer(
  a: ModelSelectionSlash | undefined,
  b: ModelSelectionSlash | undefined,
  useDefaultsA: boolean,
  useDefaultsB: boolean,
): boolean {
  if (useDefaultsA !== useDefaultsB) return true;
  if (useDefaultsA && useDefaultsB) return false;
  return selectionKey(a) !== selectionKey(b);
}

/**
 * Compare baseline (opened-from) vs draft (current wizard) workspace config.
 * `documentCount` softens rebuild urgency messaging when zero (EC-101-18).
 */
export function diffWorkspaceConfig(
  baseline: WorkspaceConfigSnapshot,
  draft: WorkspaceConfigSnapshot,
  opts: { documentCount?: number } = {},
): WorkspaceConfigDiff {
  const changedKeys: WorkspaceConfigChangedKey[] = [];

  if (
    modelsDiffer(
      baseline.llm,
      draft.llm,
      baseline.useServerDefaults,
      draft.useServerDefaults,
    )
  ) {
    changedKeys.push('llm');
  }
  if (
    modelsDiffer(
      baseline.embedding,
      draft.embedding,
      baseline.useServerDefaults,
      draft.useServerDefaults,
    )
  ) {
    changedKeys.push('embedding');
  }
  if (
    modelsDiffer(
      baseline.vision,
      draft.vision,
      baseline.useServerDefaults,
      draft.useServerDefaults,
    )
  ) {
    changedKeys.push('vision');
  }
  if (baseline.pdfParserBackend !== draft.pdfParserBackend) {
    changedKeys.push('pdfParser');
  }
  if (languageKey(baseline.extractionLanguage) !== languageKey(draft.extractionLanguage)) {
    changedKeys.push('extractionLanguage');
  }
  const baseChunk = `${baseline.chunkingMode ?? 'inherit'}:${baseline.chunkTokenSize ?? ''}:${baseline.chunkOverlapTokenSize ?? ''}`;
  const draftChunk = `${draft.chunkingMode ?? 'inherit'}:${draft.chunkTokenSize ?? ''}:${draft.chunkOverlapTokenSize ?? ''}`;
  if (baseChunk !== draftChunk) {
    changedKeys.push('chunking');
  }
  const baseBudget = `${baseline.extractBudgetMode ?? 'inherit'}:${baseline.extractMaxEntities ?? ''}:${baseline.extractMaxRecords ?? ''}`;
  const draftBudget = `${draft.extractBudgetMode ?? 'inherit'}:${draft.extractMaxEntities ?? ''}:${draft.extractMaxRecords ?? ''}`;
  if (baseBudget !== draftBudget) {
    changedKeys.push('extractBudget');
  }
  if (entityTypesKey(baseline.entityTypes) !== entityTypesKey(draft.entityTypes)) {
    changedKeys.push('entityTypes');
  }
  if (baseline.entityTypesStrict !== draft.entityTypesStrict) {
    changedKeys.push('entityTypesStrict');
  }
  if (
    entityTypeColorsKey(baseline.entityTypeColors) !==
    entityTypeColorsKey(draft.entityTypeColors)
  ) {
    changedKeys.push('entityTypeColors');
  }
  if (
    entityTypesKey(baseline.relationTypes ?? []) !==
    entityTypesKey(draft.relationTypes ?? [])
  ) {
    changedKeys.push('relationTypes');
  }
  if ((baseline.relationTypesStrict ?? true) !== (draft.relationTypesStrict ?? true)) {
    changedKeys.push('relationTypesStrict');
  }
  if ((baseline.kgSchemaPreset ?? '') !== (draft.kgSchemaPreset ?? '')) {
    changedKeys.push('kgSchemaPreset');
  }
  if (
    relationEdgesKey(baseline.relationEdges) !== relationEdgesKey(draft.relationEdges)
  ) {
    changedKeys.push('relationEdges');
  }

  const docs = opts.documentCount ?? 0;
  const modelChanged = (key: WorkspaceConfigChangedKey) => changedKeys.includes(key);
  const schemaChanged =
    modelChanged('entityTypes') ||
    modelChanged('relationTypes') ||
    modelChanged('relationEdges') ||
    modelChanged('extractionLanguage') ||
    modelChanged('chunking') ||
    modelChanged('extractBudget');

  // Rebuild hints only when there are documents to rebuild (EC-101-16…18).
  // SPEC-114: schema changes also suggest KG rebuild (honest future-only apply).
  const rebuildHints: WorkspaceRebuildHints = {
    embeddings: docs > 0 && modelChanged('embedding'),
    extraction: docs > 0 && (modelChanged('llm') || schemaChanged),
    vision: docs > 0 && modelChanged('vision'),
  };

  return {
    changedKeys,
    rebuildHints,
    hasChanges: changedKeys.length > 0,
  };
}

/** Pending-rebuild shape used by WorkspaceActionsCard. */
export function toPendingRebuild(
  hints: WorkspaceRebuildHints,
): WorkspaceRebuildHints | null {
  if (!hints.embeddings && !hints.extraction && !hints.vision) return null;
  return { ...hints };
}
