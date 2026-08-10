/**
 * SPEC-101 Wave 8 — Prefill reconfigure wizard from an existing Workspace.
 */

import { ENTITY_PRESETS, type PresetKey } from '@/constants/entity-presets';
import {
  EDGE_PRESETS,
  RELATION_PRESETS,
  detectKgSchemaPreset,
  type RelationEdge,
} from '@/constants/kg-schema-presets';
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

/**
 * Resolve relation allow-list + typed edges for the reconfigure draft.
 *
 * - Persisted relations/edges win.
 * - When relations empty but `kg_schema_preset` names a known domain,
 *   seed RELATION_PRESETS + EDGE_PRESETS (snapshot matches draft).
 * - Otherwise keep empty (free-form / unconstrained endpoints).
 */
function resolveSchemaForPrefill(
  workspace: Workspace,
  entityTypes: string[],
): {
  relationTypes: string[];
  relationEdges: RelationEdge[];
  kgSchemaPreset: PresetKey | string;
} {
  const persistedEdges: RelationEdge[] = (workspace.relation_edges ?? []).map(
    (e) => ({
      source: e.source,
      relation: e.relation,
      target: e.target,
    }),
  );

  if (workspace.relation_types?.length) {
    const preset =
      workspace.kg_schema_preset ??
      detectKgSchemaPreset(
        entityTypes,
        workspace.relation_types,
        persistedEdges,
      );
    return {
      relationTypes: [...workspace.relation_types],
      relationEdges: persistedEdges,
      kgSchemaPreset: preset,
    };
  }

  const named = workspace.kg_schema_preset?.trim();
  if (named && named !== 'custom' && named in RELATION_PRESETS) {
    const key = named as Exclude<PresetKey, 'custom'>;
    // Blank: keep empty vocabulary (do not seed General defaults).
    if (key === 'blank') {
      return {
        relationTypes: [],
        relationEdges: persistedEdges,
        kgSchemaPreset: 'blank',
      };
    }
    return {
      relationTypes: [...RELATION_PRESETS[key]],
      relationEdges: persistedEdges.length
        ? persistedEdges
        : EDGE_PRESETS[key].map((e) => ({ ...e })),
      kgSchemaPreset: key,
    };
  }

  return {
    relationTypes: [],
    relationEdges: persistedEdges,
    kgSchemaPreset:
      workspace.kg_schema_preset ??
      detectKgSchemaPreset(entityTypes, [], persistedEdges),
  };
}

export function prefillReconfigureFromWorkspace(workspace: Workspace): ReconfigurePrefill {
  const llm = getWorkspaceLlmSelection(workspace);
  const embedding = getWorkspaceEmbeddingSelection(workspace);
  const vision = getWorkspaceVisionSelection(workspace);
  const pdfParserBackend = getWorkspacePdfParserBackend(workspace);
  const hasOverrides = Boolean(llm || embedding || vision);

  const entityTypes = workspace.entity_types?.length
    ? [...workspace.entity_types]
    : [...ENTITY_PRESETS.general.types];
  const { relationTypes, relationEdges, kgSchemaPreset } = resolveSchemaForPrefill(
    workspace,
    entityTypes,
  );

  const draft: WizardDraft = {
    ...EMPTY_WIZARD_DRAFT,
    workspaceName: workspace.name,
    workspaceSlug: workspace.slug ?? '',
    workspaceDescription: workspace.description ?? '',
    useServerDefaults: !hasOverrides,
    extractionLanguage: workspace.extraction_language ?? null,
    entityTypes,
    entityTypesStrict: workspace.entity_types_strict ?? true,
    entityTypeColors: { ...(workspace.entity_type_colors ?? {}) },
    relationTypes,
    relationTypesStrict: workspace.relation_types_strict ?? true,
    kgSchemaPreset,
    relationEdges,
    pdfParserBackend,
    visionExtractImages: workspace.vision_extract_images ?? true,
    visionExtractCharts: workspace.vision_extract_charts ?? true,
    visionExtractFigures: workspace.vision_extract_figures ?? true,
    visionPageSystemPrompt: workspace.vision_page_system_prompt ?? '',
    visionImageSystemPrompt: workspace.vision_image_system_prompt ?? '',
    visionChartSystemPrompt: workspace.vision_chart_system_prompt ?? '',
    visionFigureSystemPrompt: workspace.vision_figure_system_prompt ?? '',
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
    relationTypes: [...draft.relationTypes],
    relationTypesStrict: draft.relationTypesStrict,
    kgSchemaPreset: draft.kgSchemaPreset,
    relationEdges: draft.relationEdges.map((e) => ({ ...e })),
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
    relationTypes: [...(args.draft.relationTypes ?? [])],
    relationTypesStrict: args.draft.relationTypesStrict ?? true,
    kgSchemaPreset: args.draft.kgSchemaPreset,
    relationEdges: (args.draft.relationEdges ?? []).map((e) => ({ ...e })),
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
