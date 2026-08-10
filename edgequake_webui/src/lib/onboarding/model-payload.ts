/**
 * SPEC-101 — Build create-tenant / create-workspace / reconfigure model payloads.
 * When useServerDefaults is true on create, omit model fields so the inheritance ladder applies.
 * On update, empty strings clear workspace overrides (SPEC-013).
 */

import type { PdfParserBackendDraft } from '@/lib/onboarding/wizard-state';
import { extractionLanguageToUpdatePayload } from '@/constants/extraction-languages';

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
    if (typeof args.embedding.dimension === 'number' && args.embedding.dimension > 0) {
      payload.embedding_dimension = args.embedding.dimension;
    }
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
  extractionLanguage: string | null;
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
  entity_types: string[];
  entity_types_strict: boolean;
  extraction_language: string;
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
      entity_types: args.entityTypes,
      entity_types_strict: args.entityTypesStrict,
      extraction_language: extractionLanguageToUpdatePayload(args.extractionLanguage),
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
      typeof args.embedding?.dimension === 'number' ? args.embedding.dimension : 0,
    vision_llm_model: args.vision?.model ?? '',
    vision_llm_provider: args.vision?.provider ?? '',
    pdf_parser_backend: args.pdfParserBackend,
    entity_types: args.entityTypes,
    entity_types_strict: args.entityTypesStrict,
    extraction_language: extractionLanguageToUpdatePayload(args.extractionLanguage),
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
