/**
 * SPEC-117 — Workspace extract budget helpers (LightRAG 40/100 preset).
 */

export type ExtractBudgetMode = 'inherit' | 'custom';

export const LIGHTRAG_EXTRACT_MAX_ENTITIES = 40;
export const LIGHTRAG_EXTRACT_MAX_RECORDS = 100;

export function parseExtractBudgetMode(
  raw: string | null | undefined,
  hasEntities?: boolean,
): ExtractBudgetMode {
  const v = (raw ?? '').trim().toLowerCase();
  if (v === 'custom' || hasEntities) return 'custom';
  return 'inherit';
}

export function validateExtractBudgetPair(
  entities: number,
  records: number,
): string | null {
  if (!Number.isFinite(entities) || entities < 1) {
    return 'Entities must be at least 1.';
  }
  if (!Number.isFinite(records) || records < entities) {
    return 'Records must be greater than or equal to entities.';
  }
  return null;
}

/** Payload fields for create/update workspace. */
export function extractBudgetToUpdatePayload(args: {
  mode: ExtractBudgetMode;
  entities?: number | null;
  records?: number | null;
}): {
  extract_budget_mode: string;
  extract_max_entities?: number;
  extract_max_records?: number;
} {
  if (args.mode === 'inherit') {
    return { extract_budget_mode: 'inherit' };
  }
  return {
    extract_budget_mode: 'custom',
    extract_max_entities: args.entities ?? LIGHTRAG_EXTRACT_MAX_ENTITIES,
    extract_max_records: args.records ?? LIGHTRAG_EXTRACT_MAX_RECORDS,
  };
}

export function formatExtractBudgetBadge(args: {
  mode: ExtractBudgetMode;
  entities?: number | null;
  records?: number | null;
}): string {
  if (args.mode === 'inherit') return 'Inherit';
  const e = args.entities ?? LIGHTRAG_EXTRACT_MAX_ENTITIES;
  const r = args.records ?? LIGHTRAG_EXTRACT_MAX_RECORDS;
  return `Custom · ${e}/${r}`;
}
