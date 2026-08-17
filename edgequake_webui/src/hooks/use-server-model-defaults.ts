'use client';

<<<<<<< HEAD
import { fetchModelsConfig } from '@/lib/api/models';
import { useQuery } from '@tanstack/react-query';

/**
 * Server-configured default models from /api/v1/models (env / models.toml).
 * @implements SPEC-013 / GitHub #233
 */
export function useServerModelDefaults() {
  const { data, isLoading } = useQuery({
=======
import { apiClient } from '@/lib/api/client';
import { fetchModelsConfig } from '@/lib/api/models';
import { useQuery } from '@tanstack/react-query';

interface LlmDefaultsEffectiveSlice {
  effective?: {
    llm_provider?: string | null;
    llm_model?: string | null;
    embedding_provider?: string | null;
    embedding_model?: string | null;
    vision_provider?: string | null;
    vision_model?: string | null;
  };
}

/**
 * Server-configured default models from /api/v1/models (+ optional llm-defaults for vision).
 * @implements SPEC-013 / GitHub #233
 * @implements SPEC-101 — explicit LLM · Embedding · Vision
 */
export function useServerModelDefaults() {
  const modelsQuery = useQuery({
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    queryKey: ['models', 'defaults'],
    queryFn: fetchModelsConfig,
    staleTime: 5 * 60_000,
  });

<<<<<<< HEAD
  const hasConfiguredDefaults = Boolean(
    data?.default_llm_model &&
      data?.default_embedding_model &&
      data.default_llm_model.length > 0 &&
      data.default_embedding_model.length > 0
  );

  return {
    isLoading,
    hasConfiguredDefaults,
    defaultLlmProvider: data?.default_llm_provider,
    defaultLlmModel: data?.default_llm_model,
    defaultEmbeddingProvider: data?.default_embedding_provider,
    defaultEmbeddingModel: data?.default_embedding_model,
=======
  const llmDefaultsQuery = useQuery({
    queryKey: ['settings', 'llm-defaults', 'vision-slice'],
    queryFn: () => apiClient<LlmDefaultsEffectiveSlice>('/settings/llm-defaults'),
    staleTime: 5 * 60_000,
    retry: false,
  });

  const data = modelsQuery.data;
  const effective = llmDefaultsQuery.data?.effective;

  const defaultLlmProvider = data?.default_llm_provider ?? effective?.llm_provider ?? undefined;
  const defaultLlmModel = data?.default_llm_model ?? effective?.llm_model ?? undefined;
  const defaultEmbeddingProvider =
    data?.default_embedding_provider ?? effective?.embedding_provider ?? undefined;
  const defaultEmbeddingModel =
    data?.default_embedding_model ?? effective?.embedding_model ?? undefined;
  const defaultVisionProvider =
    effective?.vision_provider ?? defaultLlmProvider ?? undefined;
  const defaultVisionModel = effective?.vision_model ?? defaultLlmModel ?? undefined;

  const hasConfiguredDefaults = Boolean(
    defaultLlmModel &&
      defaultEmbeddingModel &&
      defaultLlmModel.length > 0 &&
      defaultEmbeddingModel.length > 0,
  );

  return {
    isLoading: modelsQuery.isLoading,
    hasConfiguredDefaults,
    defaultLlmProvider,
    defaultLlmModel,
    defaultEmbeddingProvider,
    defaultEmbeddingModel,
    defaultVisionProvider,
    defaultVisionModel,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  };
}
