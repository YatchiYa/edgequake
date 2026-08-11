/**
 * SPEC-117 — Map wizard draft ↔ extract-budget card value (SSOT for all wizard kinds).
 */

import {
  LIGHTRAG_EXTRACT_MAX_ENTITIES,
  LIGHTRAG_EXTRACT_MAX_RECORDS,
  parseExtractBudgetMode,
  validateExtractBudgetPair,
  type ExtractBudgetMode,
} from '@/constants/extract-budget';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';

export type DraftExtractBudgetValue = {
  mode: ExtractBudgetMode;
  entities: number;
  records: number;
};

type ExtractBudgetDraftSlice = Pick<
  WizardDraft,
  'extractBudgetMode' | 'extractMaxEntities' | 'extractMaxRecords'
>;

export function extractBudgetValueFromDraft(
  draft: ExtractBudgetDraftSlice,
): DraftExtractBudgetValue {
  return {
    mode: parseExtractBudgetMode(
      draft.extractBudgetMode,
      draft.extractBudgetMode === 'custom',
    ),
    entities: draft.extractMaxEntities ?? LIGHTRAG_EXTRACT_MAX_ENTITIES,
    records: draft.extractMaxRecords ?? LIGHTRAG_EXTRACT_MAX_RECORDS,
  };
}

export function draftPatchFromExtractBudgetValue(
  next: DraftExtractBudgetValue,
): Partial<WizardDraft> {
  return {
    extractBudgetMode: next.mode === 'inherit' ? null : next.mode,
    extractMaxEntities: next.entities,
    extractMaxRecords: next.records,
  };
}

/** Custom mode must pass entities/records validation before Next. */
export function isExtractBudgetDraftValid(
  draft: ExtractBudgetDraftSlice,
): boolean {
  const value = extractBudgetValueFromDraft(draft);
  if (value.mode !== 'custom') return true;
  return validateExtractBudgetPair(value.entities, value.records) === null;
}
