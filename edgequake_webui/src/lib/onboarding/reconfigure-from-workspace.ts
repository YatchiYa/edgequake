/**
 * SPEC-101 Wave 8 — Prefill reconfigure wizard from an existing Workspace.
 */

import { ENTITY_PRESETS } from '@/constants/entity-presets';
import type { EmbeddingSelection } from '@/components/workspace/embedding-model-selector';
import type { LLMSelection } from '@/components/workspace/llm-model-selector';
import {
  EMPTY_WIZARD_DRAFT,
  type WizardDraft,
} from '@/lib/onboarding/wizard-state';
import type { WorkspaceConfigSnapshot } from '@/lib/onboarding/workspace-config-diff';
import {
  getWorkspaceEmbeddingSelection,
  getWorkspaceLlmSelection,
  getWorkspacePdfParserBackend,
  getWorkspaceVisionSelection,
} from '@/lib/workspace/drafts';
import type { Workspace } from '@/types';

export interface ReconfigurePrefill {
  draft: WizardDraft;
  llm: LLMSelection | undefined;
  embedding: EmbeddingSelection | undefined;
  vision: LLMSelection | undefined;
  /** Open Advanced when workspace already has model overrides. */
  advancedOpen: boolean;
  snapshot: WorkspaceConfigSnapshot;
}

export function prefillReconfigureFromWorkspace(workspace: Workspace): ReconfigurePrefill {
  const llm = getWorkspaceLlmSelection(workspace);
  const embedding = getWorkspaceEmbeddingSelection(workspace);
  const vision = getWorkspaceVisionSelection(workspace);
  const pdfParserBackend = getWorkspacePdfParserBackend(workspace);
  const hasOverrides = Boolean(llm || embedding || vision);

  const draft: WizardDraft = {
    ...EMPTY_WIZARD_DRAFT,
    workspaceName: workspace.name,
    workspaceSlug: workspace.slug ?? '',
    workspaceDescription: workspace.description ?? '',
    useServerDefaults: !hasOverrides,
    extractionLanguage: workspace.extraction_language ?? null,
    entityTypes: workspace.entity_types?.length
      ? [...workspace.entity_types]
      : [...ENTITY_PRESETS.general.types],
    entityTypesStrict: workspace.entity_types_strict ?? true,
    entityTypeColors: { ...(workspace.entity_type_colors ?? {}) },
    pdfParserBackend,
  };

  const snapshot: WorkspaceConfigSnapshot = {
    useServerDefaults: draft.useServerDefaults,
    llm,
    embedding,
    vision,
    pdfParserBackend,
    extractionLanguage: draft.extractionLanguage,
    entityTypes: [...draft.entityTypes],
    entityTypesStrict: draft.entityTypesStrict,
    entityTypeColors: { ...draft.entityTypeColors },
  };

  return {
    draft,
    llm,
    embedding,
    vision,
    advancedOpen: hasOverrides,
    snapshot,
  };
}

export function snapshotFromWizardState(args: {
  draft: WizardDraft;
  llm?: LLMSelection;
  embedding?: EmbeddingSelection;
  vision?: LLMSelection;
}): WorkspaceConfigSnapshot {
  return {
    useServerDefaults: args.draft.useServerDefaults,
    llm: args.llm,
    embedding: args.embedding,
    vision: args.vision,
    pdfParserBackend: args.draft.pdfParserBackend,
    extractionLanguage: args.draft.extractionLanguage,
    entityTypes: [...args.draft.entityTypes],
    entityTypesStrict: args.draft.entityTypesStrict,
    entityTypeColors: { ...(args.draft.entityTypeColors ?? {}) },
  };
}

/**
 * When restoring a session draft, never keep useServerDefaults=true while Advanced
 * is open or the workspace already has concrete picks — Apply would clear overrides.
 */
export function resolveHydratedUseServerDefaults(args: {
  prefillAdvancedOpen: boolean;
  prefillUseServerDefaults: boolean;
  hasPrefillPicks: boolean;
  storedUseServerDefaults: boolean;
}): boolean {
  const advanced =
    args.prefillAdvancedOpen ||
    !args.prefillUseServerDefaults ||
    args.hasPrefillPicks;
  return advanced ? false : args.storedUseServerDefaults;
}
