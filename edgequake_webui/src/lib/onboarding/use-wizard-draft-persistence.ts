'use client';

import {
  clearWizardDraft,
  hydrateWizardDraft,
  loadWizardDraft,
  saveWizardDraft,
} from '@/lib/onboarding/wizard-draft-storage';
import {
  clampStepIndex,
  type WizardDraft,
  type WizardKind,
} from '@/lib/onboarding/wizard-state';
import { useEffect, useRef } from 'react';

/**
 * Hydrate once when the wizard opens; persist non-secret draft + step on change;
 * clear via returned `clearDraft` after successful finish.
 * Pass `scope` (e.g. workspaceId) for reconfigure drafts (EC-101-22).
 */
export function useWizardDraftPersistence(
  kind: WizardKind,
  open: boolean,
  draft: WizardDraft,
  setDraft: React.Dispatch<React.SetStateAction<WizardDraft>>,
  stepIndex: number,
  setStepIndex: React.Dispatch<React.SetStateAction<number>>,
  stepCount: number,
  scope?: string | null,
  /** When false, caller hydrates (e.g. reconfigure prefills from workspace). Default true. */
  hydrateOnOpen = true,
): { clearDraft: () => void } {
  const hydrated = useRef(false);
  const skipPersist = useRef(false);

  useEffect(() => {
    if (!open) {
      hydrated.current = false;
      return;
    }
    if (hydrated.current) return;
    hydrated.current = true;
    if (!hydrateOnOpen) return;
    const stored = loadWizardDraft(kind, scope);
    if (!stored) return;
    setDraft((base) => hydrateWizardDraft(base, stored));
    setStepIndex(clampStepIndex(stored.stepIndex, stepCount || 1));
  }, [open, kind, scope, hydrateOnOpen, setDraft, setStepIndex, stepCount]);

  useEffect(() => {
    if (!open || !hydrated.current) return;
    if (skipPersist.current) {
      skipPersist.current = false;
      return;
    }
    saveWizardDraft(kind, draft, stepIndex, scope);
  }, [open, kind, scope, draft, stepIndex]);

  return {
    clearDraft: () => {
      skipPersist.current = true;
      clearWizardDraft(kind, scope);
    },
  };
}
