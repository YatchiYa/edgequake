/**
 * SPEC-101 — Build create-tenant / create-workspace / reconfigure model payloads.
 * When useServerDefaults is true on create, omit model fields so the inheritance ladder applies.
 * On update, empty strings clear workspace overrides (SPEC-013).
 */

import type { PdfParserBackendDraft } from '@/lib/onboarding/wizard-state';
import { resolveEmbeddingDimension } from '@/lib/onboarding/resolve-embedding-dimension';
import { extractionLanguageToUpdatePayload } from '@/constants/extraction-languages';
import {
  ACC_FAIR_CHUNK_OVERLAP,
  ACC_FAIR_CHUNK_TOKEN_SIZE,
  chunkingToUpdatePayload,
  parseChunkingMode,
  type ChunkingMode,
} from '@/constants/chunking-policy';
import {
  extractBudgetToUpdatePayload,
  LIGHTRAG_EXTRACT_MAX_ENTITIES,
  LIGHTRAG_EXTRACT_MAX_RECORDS,
  parseExtractBudgetMode,
  type ExtractBudgetMode,
} from '@/constants/extract-budget';

export interface ModelSelectionSlash {
  provider: string;
  model: string;
  fullId?: string;
  dimension?: number;
}

export interface TenantModelPayload {
  default_llm_model?: string;
  default_llm_provider?: string;
  default_embedding_model?: string;
  default_embedding_provider?: string;
  default_embedding_dimension?: number;
  default_vision_llm_model?: string;
  default_vision_llm_provider?: string;
  /** SPEC-109: seed fleet/workspace default reasoning effort */
  default_reasoning_effort?: string;
}

export interface WorkspaceModelPayload {
  llm_model?: string;
  llm_provider?: string;
  embedding_model?: string;
  embedding_provider?: string;
  embedding_dimension?: number;
  vision_llm_model?: string;
  vision_llm_provider?: string;
  /** SPEC-109: workspace metadata default_reasoning_effort */
  default_reasoning_effort?: string;
}

export function buildTenantModelPayload(args: {
  useServerDefaults: boolean;
  llm?: ModelSelectionSlash;
  embedding?: ModelSelectionSlash;
  vision?: ModelSelectionSlash;
  reasoningEffort?: string;
}): TenantModelPayload {
  if (args.useServerDefaults) {
    return {};
  }
  const payload: TenantModelPayload = {};
  if (args.llm?.provider && args.llm?.model) {
    payload.default_llm_provider = args.llm.provider;
    payload.default_llm_model = args.llm.model;
  }
  if (args.embedding?.provider && args.embedding?.model) {
    payload.default_embedding_provider = args.embedding.provider;
    payload.default_embedding_model = args.embedding.model;
    if (typeof args.embedding.dimension === 'number' && args.embedding.dimension > 0) {
      payload.default_embedding_dimension = args.embedding.dimension;
    }
  }
  if (args.vision?.provider && args.vision?.model) {
    payload.default_vision_llm_provider = args.vision.provider;
    payload.default_vision_llm_model = args.vision.model;
  }
  if (args.reasoningEffort?.trim()) {
    payload.default_reasoning_effort = args.reasoningEffort.trim();
  }
  return payload;
}

export function buildWorkspaceModelPayload(args: {
  useServerDefaults: boolean;
  llm?: ModelSelectionSlash;
  embedding?: ModelSelectionSlash;
  vision?: ModelSelectionSlash;
  reasoningEffort?: string;
}): WorkspaceModelPayload {
  if (args.useServerDefaults) {
    return {};
  }
  const payload: WorkspaceModelPayload = {};
  if (args.llm?.provider && args.llm?.model) {
    payload.llm_provider = args.llm.provider;
    payload.llm_model = args.llm.model;
  }
  if (args.embedding?.provider && args.embedding?.model) {
    payload.embedding_provider = args.embedding.provider;
    payload.embedding_model = args.embedding.model;
    payload.embedding_dimension = resolveEmbeddingDimension({
      provider: args.embedding.provider,
      model: args.embedding.model,
      dimension: args.embedding.dimension,
    });
  }
  if (args.vision?.provider && args.vision?.model) {
    payload.vision_llm_provider = args.vision.provider;
    payload.vision_llm_model = args.vision.model;
  }
  if (args.reasoningEffort?.trim()) {
    payload.default_reasoning_effort = args.reasoningEffort.trim();
  }
  return payload;
}

/**
 * SPEC-101 Wave 8 — Build PUT /workspaces/{id} payload.
 * Empty strings clear LLM/embedding/vision overrides → server defaults (SPEC-013).
 */
export function buildWorkspaceUpdatePayload(args: {
  useServerDefaults: boolean;
  llm?: ModelSelectionSlash;
  embedding?: ModelSelectionSlash;
  vision?: ModelSelectionSlash;
  pdfParserBackend: PdfParserBackendDraft;
  /** SPEC-015V */
  visionExtractImages?: boolean;
  visionExtractCharts?: boolean;
  visionExtractFigures?: boolean;
  visionPageSystemPrompt?: string;
  visionImageSystemPrompt?: string;
  visionChartSystemPrompt?: string;
  visionFigureSystemPrompt?: string;
  extractionLanguage: string | null;
  /** SPEC-116 */
  chunkingMode?: ChunkingMode | null;
  chunkTokenSize?: number | null;
  chunkOverlapTokenSize?: number | null;
  /** SPEC-117 */
  extractBudgetMode?: ExtractBudgetMode | null;
  extractMaxEntities?: number | null;
  extractMaxRecords?: number | null;
  entityTypes: string[];
  entityTypesStrict: boolean;
  entityTypeColors?: Record<string, string>;
  /** SPEC-114 / 114b */
  relationTypes?: string[];
  relationTypesStrict?: boolean;
  kgSchemaPreset?: string;
  relationEdges?: Array<{ source: string; relation: string; target: string }>;
  reasoningEffort?: string;
}): {
  llm_model: string;
  llm_provider: string;
  embedding_model: string;
  embedding_provider: string;
  embedding_dimension: number;
  vision_llm_model: string;
  vision_llm_provider: string;
  pdf_parser_backend: PdfParserBackendDraft;
  vision_extract_images?: boolean;
  vision_extract_charts?: boolean;
  vision_extract_figures?: boolean;
  vision_page_system_prompt?: string;
  vision_image_system_prompt?: string;
  vision_chart_system_prompt?: string;
  vision_figure_system_prompt?: string;
  entity_types: string[];
  entity_types_strict: boolean;
  extraction_language: string;
  chunking_mode: string;
  chunk_token_size?: number;
  chunk_overlap_token_size?: number;
  extract_budget_mode: string;
  extract_max_entities?: number;
  extract_max_records?: number;
  entity_type_colors: Record<string, string>;
  relation_types: string[];
  relation_types_strict: boolean;
  kg_schema_preset: string;
  relation_edges: Array<{ source: string; relation: string; target: string }>;
  default_reasoning_effort?: string;
} {
  const entity_type_colors = args.entityTypeColors ?? {};
  const relation_types = args.relationTypes ?? [];
  const relation_types_strict = args.relationTypesStrict ?? true;
  const kg_schema_preset = args.kgSchemaPreset?.trim() || 'custom';
  const relation_edges = args.relationEdges ?? [];
  const effort =
    args.reasoningEffort?.trim() && args.reasoningEffort.trim().length > 0
      ? args.reasoningEffort.trim()
      : undefined;
  const schemaFields = {
    relation_types,
    relation_types_strict,
    kg_schema_preset,
    relation_edges,
  };
  const visionExtractFields = {
    vision_extract_images: args.visionExtractImages ?? true,
    vision_extract_charts: args.visionExtractCharts ?? true,
    vision_extract_figures: args.visionExtractFigures ?? true,
    vision_page_system_prompt: args.visionPageSystemPrompt ?? '',
    vision_image_system_prompt: args.visionImageSystemPrompt ?? '',
    vision_chart_system_prompt: args.visionChartSystemPrompt ?? '',
    vision_figure_system_prompt: args.visionFigureSystemPrompt ?? '',
  };
  const chunkingFields = chunkingToUpdatePayload({
    mode: parseChunkingMode(args.chunkingMode),
    size: args.chunkTokenSize ?? ACC_FAIR_CHUNK_TOKEN_SIZE,
    overlap: args.chunkOverlapTokenSize ?? ACC_FAIR_CHUNK_OVERLAP,
  });
  const extractBudgetFields = extractBudgetToUpdatePayload({
    mode: parseExtractBudgetMode(
      args.extractBudgetMode,
      typeof args.extractMaxEntities === 'number' &&
        args.extractBudgetMode === 'custom',
    ),
    entities: args.extractMaxEntities ?? LIGHTRAG_EXTRACT_MAX_ENTITIES,
    records: args.extractMaxRecords ?? LIGHTRAG_EXTRACT_MAX_RECORDS,
  });
  if (args.useServerDefaults) {
    return {
      llm_model: '',
      llm_provider: '',
      embedding_model: '',
      embedding_provider: '',
      embedding_dimension: 0,
      vision_llm_model: '',
      vision_llm_provider: '',
      pdf_parser_backend: args.pdfParserBackend,
      ...visionExtractFields,
      entity_types: args.entityTypes,
      entity_types_strict: args.entityTypesStrict,
      extraction_language: extractionLanguageToUpdatePayload(args.extractionLanguage),
      ...chunkingFields,
      ...extractBudgetFields,
      entity_type_colors,
      ...schemaFields,
      ...(effort ? { default_reasoning_effort: effort } : {}),
    };
  }

  return {
    llm_model: args.llm?.model ?? '',
    llm_provider: args.llm?.provider ?? '',
    embedding_model: args.embedding?.model ?? '',
    embedding_provider: args.embedding?.provider ?? '',
    embedding_dimension:
      args.embedding?.provider && args.embedding?.model
        ? resolveEmbeddingDimension({
            provider: args.embedding.provider,
            model: args.embedding.model,
            dimension: args.embedding.dimension,
          })
        : 0,
    vision_llm_model: args.vision?.model ?? '',
    vision_llm_provider: args.vision?.provider ?? '',
    pdf_parser_backend: args.pdfParserBackend,
    ...visionExtractFields,
    entity_types: args.entityTypes,
    entity_types_strict: args.entityTypesStrict,
    extraction_language: extractionLanguageToUpdatePayload(args.extractionLanguage),
    ...chunkingFields,
    ...extractBudgetFields,
    entity_type_colors,
    ...schemaFields,
    ...(effort ? { default_reasoning_effort: effort } : {}),
  };
}

/** Normalize legacy `provider:model` to slash form. */
export function normalizeModelFullId(value: string): { provider: string; model: string; fullId: string } {
  const slash = value.indexOf('/');
  if (slash !== -1) {
    const provider = value.slice(0, slash);
    const model = value.slice(slash + 1);
    return { provider, model, fullId: `${provider}/${model}` };
  }
  const colon = value.indexOf(':');
  if (colon !== -1) {
    const provider = value.slice(0, colon);
    const model = value.slice(colon + 1);
    return { provider, model, fullId: `${provider}/${model}` };
  }
  return { provider: 'unknown', model: value, fullId: `unknown/${value}` };
}
