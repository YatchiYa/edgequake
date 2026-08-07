/**
 * SPEC-101 — Pure wizard step navigation + validation (unit-tested).
 */

export type WizardKind =
  | 'first-run'
  | 'create-tenant'
  | 'create-workspace'
  | 'reconfigure-workspace';

export type WizardStepId =
  | 'admin'
  | 'tenant-basics'
  | 'models'
  | 'workspace-basics'
  | 'document-parsing'
  | 'extraction'
  | 'review';

export type PdfParserBackendDraft = 'none' | 'vision' | 'edgeparse';

export interface WizardDraft {
  adminUsername: string;
  adminEmail: string;
  adminPassword: string;
  adminPasswordConfirm: string;
  tenantName: string;
  tenantDescription: string;
  workspaceName: string;
  workspaceSlug: string;
  workspaceDescription: string;
  useServerDefaults: boolean;
  extractionLanguage: string | null;
  entityTypes: string[];
  /** SPEC-101 Wave 8 — PDF parser (reconfigure). */
  pdfParserBackend: PdfParserBackendDraft;
  /** SPEC-101 Wave 8 — strict entity types (reconfigure). */
  entityTypesStrict: boolean;
  /** SPEC-102 — entity type → hex color overrides. */
  entityTypeColors: Record<string, string>;
  /** SPEC-109 — seed default reasoning effort. */
  reasoningEffort?: string;
}

export const EMPTY_WIZARD_DRAFT: WizardDraft = {
  adminUsername: 'admin',
  adminEmail: '',
  adminPassword: '',
  adminPasswordConfirm: '',
  tenantName: '',
  tenantDescription: '',
  workspaceName: '',
  workspaceSlug: '',
  workspaceDescription: '',
  useServerDefaults: true,
  extractionLanguage: null,
  entityTypes: [],
  pdfParserBackend: 'none',
  entityTypesStrict: true,
  entityTypeColors: {},
  reasoningEffort: undefined,
};

export function stepsForWizard(
  kind: WizardKind,
  opts: { includeAdmin: boolean; includeExtraction: boolean } = {
    includeAdmin: false,
    includeExtraction: true,
  },
): WizardStepId[] {
  if (kind === 'reconfigure-workspace') {
    return ['models', 'document-parsing', 'extraction', 'review'];
  }
  if (kind === 'create-tenant') {
    return ['tenant-basics', 'models', 'workspace-basics', 'extraction', 'review'];
  }
  if (kind === 'create-workspace') {
    const steps: WizardStepId[] = ['workspace-basics', 'models'];
    if (opts.includeExtraction) steps.push('extraction');
    steps.push('review');
    return steps;
  }
  // first-run
  const steps: WizardStepId[] = [];
  if (opts.includeAdmin) steps.push('admin');
  steps.push('tenant-basics', 'models', 'workspace-basics');
  if (opts.includeExtraction) steps.push('extraction');
  steps.push('review');
  return steps;
}

export function canProceed(
  step: WizardStepId,
  draft: WizardDraft,
  opts: {
    hasConfiguredDefaults: boolean;
    advancedModelsValid: boolean;
    /** When true (reconfigure), Apply requires at least one config diff. */
    hasConfigChanges?: boolean;
  } = {
    hasConfiguredDefaults: true,
    advancedModelsValid: true,
  },
): boolean {
  switch (step) {
    case 'admin': {
      const userOk = draft.adminUsername.trim().length >= 3;
      const passOk = draft.adminPassword.length >= 8;
      const match = draft.adminPassword === draft.adminPasswordConfirm;
      return userOk && passOk && match;
    }
    case 'tenant-basics':
      return draft.tenantName.trim().length > 0;
    case 'workspace-basics':
      return draft.workspaceName.trim().length > 0;
    case 'models':
      if (draft.useServerDefaults) {
        return opts.hasConfiguredDefaults;
      }
      return opts.advancedModelsValid;
    case 'document-parsing':
      return true;
    case 'extraction':
      return true;
    case 'review':
      if (opts.hasConfigChanges === false) return false;
      return true;
    default:
      return false;
  }
}

export function clampStepIndex(index: number, stepCount: number): number {
  if (stepCount <= 0) return 0;
  return Math.max(0, Math.min(index, stepCount - 1));
}

export function progressPercent(index: number, stepCount: number): number {
  if (stepCount <= 0) return 0;
  return Math.round(((index + 1) / stepCount) * 100);
}

/** Persist non-secret draft fields (never password). */
export function draftForStorage(draft: WizardDraft): Omit<
  WizardDraft,
  'adminPassword' | 'adminPasswordConfirm'
> {
  const {
    adminPassword: _p,
    adminPasswordConfirm: _c,
    ...safe
  } = draft;
  return safe;
}
