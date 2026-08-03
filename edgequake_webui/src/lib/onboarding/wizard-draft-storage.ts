/**
 * SPEC-101 LAW-101-9 / EC-101-02 — sessionStorage draft hydrate/persist/clear.
 * Passwords are never written (via draftForStorage).
 * Reconfigure drafts are scoped by workspace id (EC-101-22).
 */

import {
  draftForStorage,
  type WizardDraft,
  type WizardKind,
} from '@/lib/onboarding/wizard-state';

export const DRAFT_STORAGE_PREFIX = 'edgequake:wizard-draft:';

export type PersistedWizardPayload = {
  version: 1;
  stepIndex: number;
  draft: ReturnType<typeof draftForStorage>;
};

/** Optional scope (e.g. workspaceId) for reconfigure drafts. */
export function draftStorageKey(kind: WizardKind, scope?: string | null): string {
  const base = `${DRAFT_STORAGE_PREFIX}${kind}`;
  if (scope && scope.trim()) return `${base}:${scope.trim()}`;
  return base;
}

export function loadWizardDraft(
  kind: WizardKind,
  scope?: string | null,
): PersistedWizardPayload | null {
  if (typeof sessionStorage === 'undefined') return null;
  try {
    const raw = sessionStorage.getItem(draftStorageKey(kind, scope));
    if (!raw) return null;
    const parsed = JSON.parse(raw) as PersistedWizardPayload;
    if (parsed?.version !== 1 || !parsed.draft) return null;
    // Defense in depth — strip secrets if an older payload leaked them
    const { adminPassword: _p, adminPasswordConfirm: _c, ...safe } = parsed.draft as WizardDraft & {
      adminPassword?: string;
      adminPasswordConfirm?: string;
    };
    return {
      version: 1,
      stepIndex: Math.max(0, Number(parsed.stepIndex) || 0),
      draft: safe as ReturnType<typeof draftForStorage>,
    };
  } catch {
    return null;
  }
}

export function saveWizardDraft(
  kind: WizardKind,
  draft: WizardDraft,
  stepIndex: number,
  scope?: string | null,
): void {
  if (typeof sessionStorage === 'undefined') return;
  const payload: PersistedWizardPayload = {
    version: 1,
    stepIndex,
    draft: draftForStorage(draft),
  };
  try {
    sessionStorage.setItem(draftStorageKey(kind, scope), JSON.stringify(payload));
  } catch {
    // quota / private mode — ignore
  }
}

export function clearWizardDraft(kind: WizardKind, scope?: string | null): void {
  if (typeof sessionStorage === 'undefined') return;
  try {
    sessionStorage.removeItem(draftStorageKey(kind, scope));
  } catch {
    // ignore
  }
}

/** Merge persisted non-secret fields onto a base draft (passwords stay empty). */
export function hydrateWizardDraft(
  base: WizardDraft,
  stored: PersistedWizardPayload | null,
): WizardDraft {
  if (!stored) return base;
  return {
    ...base,
    ...stored.draft,
    adminPassword: '',
    adminPasswordConfirm: '',
  };
}
