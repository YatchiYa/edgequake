'use client';

/**
 * SPEC-116 — Dedicated wizard step for workspace chunking (SRP).
 * Reuses {@link WorkspaceChunkingCard}; draft mapping via draft-chunking SSOT.
 */

import {
  WorkspaceChunkingCard,
  type WorkspaceChunkingValue,
} from '@/components/workspace/workspace-chunking-card';
import {
  chunkingValueFromDraft,
  draftPatchFromChunkingValue,
} from '@/lib/onboarding/draft-chunking';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { Scissors } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export interface WorkspaceChunkingStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
}

export function WorkspaceChunkingStep({
  draft,
  onChange,
}: WorkspaceChunkingStepProps) {
  const { t } = useTranslation();
  const value = chunkingValueFromDraft(draft);

  return (
    <div
      className="mx-auto flex w-full max-w-2xl flex-col gap-4"
      data-testid="wizard-step-chunking"
    >
      <header className="flex items-start gap-3">
        <div
          className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-indigo-500/10"
          aria-hidden
        >
          <Scissors className="h-4 w-4 text-indigo-600" />
        </div>
        <div className="min-w-0 space-y-1">
          <h4 className="text-sm font-medium leading-none">
            {t('onboarding.chunkingStepHeading', 'Split documents into chunks')}
          </h4>
          <p className="text-xs leading-relaxed text-muted-foreground">
            {t(
              'onboarding.chunkingStepLead',
              'Chunk size shapes what each extraction call sees. Match LightRAG for Acc-fair parity, or leave Inherit for fleet defaults.',
            )}
          </p>
        </div>
      </header>

      <WorkspaceChunkingCard
        isEditing
        workspace={{
          chunking_mode: draft.chunkingMode,
          chunk_token_size: draft.chunkTokenSize,
          chunk_overlap_token_size: draft.chunkOverlapTokenSize,
        }}
        value={value}
        onChange={(next: WorkspaceChunkingValue) =>
          onChange(draftPatchFromChunkingValue(next))
        }
      />

      <aside
        className="rounded-lg border border-dashed bg-muted/20 px-3 py-2.5 text-[11px] leading-relaxed text-muted-foreground"
        data-testid="wizard-chunking-extract-hint"
      >
        {t(
          'onboarding.chunkingExtractBudgetHint',
          'Next: extract budget caps how many entities each LLM response may emit. Adaptive chunking + a high budget can inflate mentions — tune both deliberately.',
        )}
      </aside>
    </div>
  );
}
