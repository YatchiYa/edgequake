import { describe, expect, it } from 'vitest';
import {
  chunkingValueFromDraft,
  draftPatchFromChunkingValue,
  isChunkingDraftValid,
} from '../draft-chunking';
import { EMPTY_WIZARD_DRAFT } from '../wizard-state';

describe('draft-chunking', () => {
  it('maps inherit draft to card value', () => {
    expect(chunkingValueFromDraft(EMPTY_WIZARD_DRAFT)).toEqual({
      mode: 'inherit',
      size: 1200,
      overlap: 100,
    });
  });

  it('round-trips Acc-fair fixed through patch helper', () => {
    const patch = draftPatchFromChunkingValue({
      mode: 'fixed',
      size: 1200,
      overlap: 100,
    });
    expect(patch).toEqual({
      chunkingMode: 'fixed',
      chunkTokenSize: 1200,
      chunkOverlapTokenSize: 100,
    });
    expect(
      chunkingValueFromDraft({ ...EMPTY_WIZARD_DRAFT, ...patch }),
    ).toEqual({ mode: 'fixed', size: 1200, overlap: 100 });
  });

  it('treats inherit mode as clearing chunkingMode', () => {
    expect(draftPatchFromChunkingValue({ mode: 'inherit', size: 1200, overlap: 100 }))
      .toMatchObject({ chunkingMode: null });
  });

  it('validates fixed pairs', () => {
    expect(isChunkingDraftValid(EMPTY_WIZARD_DRAFT)).toBe(true);
    expect(
      isChunkingDraftValid({
        ...EMPTY_WIZARD_DRAFT,
        chunkingMode: 'fixed',
        chunkTokenSize: 50,
        chunkOverlapTokenSize: 80,
      }),
    ).toBe(false);
  });
});
