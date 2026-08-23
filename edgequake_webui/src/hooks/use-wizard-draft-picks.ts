'use client';

import type { EmbeddingSelection } from '@/components/workspace/embedding-model-selector';
import type { LLMSelection } from '@/components/workspace/llm-model-selector';
import {
  draftPatchAfterEmbeddingPick,
  draftPatchAfterLlmPick,
  draftPatchAfterVisionPick,
  embeddingSelectionFromPick,
  llmSelectionFromPick,
} from '@/lib/onboarding/wizard-picks';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { useCallback } from 'react';

/**
 * Model overrides live on {@link WizardDraft} so persistence is a single snapshot.
 */
export function useWizardDraftPicks(
  draft: WizardDraft,
  setDraft: React.Dispatch<React.SetStateAction<WizardDraft>>,
) {
  const commitLlm = useCallback(
    (next: LLMSelection | undefined) => {
      setDraft((cur) => ({ ...cur, ...draftPatchAfterLlmPick(cur, next) }));
    },
    [setDraft],
  );

  const commitEmbedding = useCallback(
    (next: EmbeddingSelection | undefined) => {
      setDraft((cur) => ({ ...cur, ...draftPatchAfterEmbeddingPick(cur, next) }));
    },
    [setDraft],
  );

  const commitVision = useCallback(
    (next: LLMSelection | undefined) => {
      setDraft((cur) => ({ ...cur, ...draftPatchAfterVisionPick(cur, next) }));
    },
    [setDraft],
  );

  const setAdvancedOpen = useCallback(
    (open: boolean) => {
      setDraft((cur) => ({ ...cur, advancedOpen: open }));
    },
    [setDraft],
  );

  return {
    llm: llmSelectionFromPick(draft.llmPick),
    embedding: embeddingSelectionFromPick(draft.embeddingPick),
    vision: llmSelectionFromPick(draft.visionPick),
    advancedOpen: draft.advancedOpen,
    commitLlm,
    commitEmbedding,
    commitVision,
    setAdvancedOpen,
  };
}
