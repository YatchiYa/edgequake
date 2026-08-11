import { describe, expect, it } from 'vitest';
import {
  draftPatchFromExtractBudgetValue,
  extractBudgetValueFromDraft,
  isExtractBudgetDraftValid,
} from '../draft-extract-budget';
import { EMPTY_WIZARD_DRAFT } from '../wizard-state';

describe('draft-extract-budget', () => {
  it('maps inherit draft to card value', () => {
    expect(extractBudgetValueFromDraft(EMPTY_WIZARD_DRAFT)).toEqual({
      mode: 'inherit',
      entities: 40,
      records: 100,
    });
  });

  it('round-trips LightRAG custom through patch helper', () => {
    const patch = draftPatchFromExtractBudgetValue({
      mode: 'custom',
      entities: 40,
      records: 100,
    });
    expect(patch).toEqual({
      extractBudgetMode: 'custom',
      extractMaxEntities: 40,
      extractMaxRecords: 100,
    });
    expect(
      extractBudgetValueFromDraft({ ...EMPTY_WIZARD_DRAFT, ...patch }),
    ).toEqual({ mode: 'custom', entities: 40, records: 100 });
  });

  it('treats inherit mode as clearing extractBudgetMode', () => {
    expect(
      draftPatchFromExtractBudgetValue({
        mode: 'inherit',
        entities: 40,
        records: 100,
      }),
    ).toMatchObject({ extractBudgetMode: null });
  });

  it('validates custom pairs', () => {
    expect(isExtractBudgetDraftValid(EMPTY_WIZARD_DRAFT)).toBe(true);
    expect(
      isExtractBudgetDraftValid({
        ...EMPTY_WIZARD_DRAFT,
        extractBudgetMode: 'custom',
        extractMaxEntities: 80,
        extractMaxRecords: 20,
      }),
    ).toBe(false);
  });
});
