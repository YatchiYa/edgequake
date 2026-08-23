import { describe, expect, it } from 'vitest';
import { EMPTY_WIZARD_DRAFT } from '../wizard-state';
import {
  draftPatchAfterLlmPick,
  embeddingSelectionFromPick,
  llmSelectionFromPick,
  pickFromLlm,
} from '../wizard-picks';

describe('wizard-picks', () => {
  it('round-trips LLM selection through a pick', () => {
    const sel = llmSelectionFromPick(
      pickFromLlm({ provider: 'mistral', model: 'mistral-small-latest' }),
    );
    expect(sel).toEqual({
      provider: 'mistral',
      model: 'mistral-small-latest',
      fullId: 'mistral/mistral-small-latest',
    });
  });

  it('defaults embedding dimension when the pick omits it', () => {
    expect(
      embeddingSelectionFromPick({ provider: 'ollama', model: 'embeddinggemma' })
        ?.dimension,
    ).toBe(768);
  });

  it('flips useServerDefaults off when a concrete LLM is picked', () => {
    const patch = draftPatchAfterLlmPick(EMPTY_WIZARD_DRAFT, {
      provider: 'mistral',
      model: 'mistral-small-latest',
    });
    expect(patch.useServerDefaults).toBe(false);
    expect(patch.advancedOpen).toBe(true);
    expect(patch.llmPick?.provider).toBe('mistral');
  });

  it('restores inherit only when every model pick is cleared', () => {
    const withOthers = {
      ...EMPTY_WIZARD_DRAFT,
      embeddingPick: { provider: 'ollama', model: 'embeddinggemma', dimension: 768 },
      useServerDefaults: false,
    };
    expect(draftPatchAfterLlmPick(withOthers, undefined).useServerDefaults).toBe(
      false,
    );
    expect(
      draftPatchAfterLlmPick(EMPTY_WIZARD_DRAFT, undefined).useServerDefaults,
    ).toBe(true);
  });
});
