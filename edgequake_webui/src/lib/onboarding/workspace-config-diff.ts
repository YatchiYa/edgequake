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
  | 'entityTypes'
  | 'entityTypesStrict'
  | 'entityTypeColors';

export interface WorkspaceConfigSnapshot {
  useServerDefaults: boolean;
  llm?: ModelSelectionSlash;
  embedding?: ModelSelectionSlash;
  vision?: ModelSelectionSlash;
  pdfParserBackend: PdfParserBackendDraft;
  extractionLanguage: string | null;
  entityTypes: string[];
  entityTypesStrict: boolean;
  entityTypeColors?: Record<string, string>;
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

  const docs = opts.documentCount ?? 0;
  const modelChanged = (key: WorkspaceConfigChangedKey) => changedKeys.includes(key);

  // Rebuild hints only when there are documents to rebuild (EC-101-16…18).
  const rebuildHints: WorkspaceRebuildHints = {
    embeddings: docs > 0 && modelChanged('embedding'),
    extraction: docs > 0 && modelChanged('llm'),
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
