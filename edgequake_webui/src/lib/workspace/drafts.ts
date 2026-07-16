import type { PdfParserBackendChoice } from "@/components/settings/pdf-parser-backend-field";
import type { EmbeddingSelection } from "@/components/workspace/embedding-model-selector";
import type { LLMSelection } from "@/components/workspace/llm-model-selector";
import type { Workspace } from "@/types";

/** Mock must never be selected/saved in the application UI. */
function isMockProvider(provider?: string | null): boolean {
  const id = provider?.trim().toLowerCase() ?? "";
  return id === "mock" || id === "mock-imagegen";
}

/**
 * Infer a real provider from a model name when the workspace still has mock leftovers.
 * Mirrors server `Workspace::detect_provider_from_model` heuristics.
 */
function healMockProvider(provider: string, model: string): string {
  if (!isMockProvider(provider)) {
    return provider;
  }
  if (model.startsWith("text-embedding") || model.startsWith("ada")) {
    return "openai";
  }
  if (model.includes(":")) {
    return "ollama";
  }
  if (
    model.startsWith("mistral") ||
    model.startsWith("magistral") ||
    model.startsWith("codestral") ||
    model.startsWith("pixtral")
  ) {
    return "mistral";
  }
  if (model.startsWith("gpt-") || model.startsWith("o1") || model.startsWith("o3")) {
    return "openai";
  }
  return "ollama";
}

export function getWorkspaceLlmSelection(
  workspace?: Workspace | null,
): LLMSelection | undefined {
  if (!workspace?.llm_provider || !workspace.llm_model) {
    return undefined;
  }

  const provider = healMockProvider(workspace.llm_provider, workspace.llm_model);
  return {
    model: workspace.llm_model,
    provider,
    fullId: `${provider}/${workspace.llm_model}`,
  };
}

export function getWorkspaceEmbeddingSelection(
  workspace?: Workspace | null,
): EmbeddingSelection | undefined {
  if (!workspace?.embedding_provider || !workspace.embedding_model) {
    return undefined;
  }

  const provider = healMockProvider(
    workspace.embedding_provider,
    workspace.embedding_model,
  );
  return {
    model: workspace.embedding_model,
    provider,
    dimension: workspace.embedding_dimension ?? 768,
  };
}

export function getWorkspaceVisionSelection(
  workspace?: Workspace | null,
): LLMSelection | undefined {
  if (!workspace?.vision_llm_provider || !workspace.vision_llm_model) {
    return undefined;
  }

  const provider = healMockProvider(
    workspace.vision_llm_provider,
    workspace.vision_llm_model,
  );
  return {
    model: workspace.vision_llm_model,
    provider,
    fullId: `${provider}/${workspace.vision_llm_model}`,
  };
}

export function getWorkspacePdfParserBackend(
  workspace?: Workspace | null,
): PdfParserBackendChoice {
  return workspace?.pdf_parser_backend ?? "none";
}
