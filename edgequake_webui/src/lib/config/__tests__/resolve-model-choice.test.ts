/**
 * SPEC-123 model priority mirror tests (LLM / embedding / vision LLM).
 */
import { describe, expect, test } from "bun:test";
import {
  effectiveEmbeddingFromWorkspace,
  effectiveLlmFromWorkspace,
  resolveEmbeddingChoice,
  resolveLlmChoice,
  resolveVisionLlmChoice,
} from "../resolve-model-choice";

describe("SPEC-123 resolveLlmChoice", () => {
  test("request wins over workspace tenant env", () => {
    const r = resolveLlmChoice({
      requestProvider: "openai",
      requestModel: "gpt-4.1-mini",
      workspaceProvider: "ollama",
      workspaceModel: "gemma4:latest",
      tenantProvider: "mistral",
      tenantModel: "mistral-small-latest",
      envProvider: "ollama",
      envModel: "gemma4:latest",
    });
    expect(r.provider).toBe("openai");
    expect(r.model).toBe("gpt-4.1-mini");
    expect(r.source).toBe("request");
  });

  test("workspace wins over tenant", () => {
    const r = resolveLlmChoice({
      workspaceProvider: "ollama",
      workspaceModel: "gemma4:latest",
      tenantProvider: "mistral",
      tenantModel: "mistral-small-latest",
    });
    expect(r.source).toBe("workspace");
    expect(r.provider).toBe("ollama");
  });

  test("tenant wins when workspace fields lack deliberate override", () => {
    const r = resolveLlmChoice({
      // Painted concrete fields without marking as override → omit from call
      tenantProvider: "mistral",
      tenantModel: "mistral-small-latest",
      envProvider: "ollama",
      envModel: "gemma4:latest",
    });
    expect(r.source).toBe("tenant");
    expect(r.provider).toBe("mistral");
  });
});

describe("SPEC-123 resolveEmbeddingChoice", () => {
  test("workspace embedding wins with dimension", () => {
    const r = resolveEmbeddingChoice({
      workspaceProvider: "ollama",
      workspaceModel: "embeddinggemma:latest",
      workspaceDimension: 768,
      tenantProvider: "openai",
      tenantModel: "text-embedding-3-small",
      tenantDimension: 1536,
    });
    expect(r.provider).toBe("ollama");
    expect(r.dimension).toBe(768);
    expect(r.source).toBe("workspace");
  });
});

describe("SPEC-123 resolveVisionLlmChoice", () => {
  test("upload wins over workspace vision", () => {
    const r = resolveVisionLlmChoice({
      uploadProvider: "openai",
      uploadModel: "gpt-4.1-nano",
      workspaceVisionProvider: "ollama",
      workspaceVisionModel: "gemma4:latest",
    });
    expect(r.provider).toBe("openai");
    expect(r.model).toBe("gpt-4.1-nano");
    expect(r.source).toBe("request");
  });

  test("tenant vision wins when workspace vision unset", () => {
    const r = resolveVisionLlmChoice({
      workspaceLlmProvider: "ollama",
      workspaceLlmModel: "gemma4:latest",
      tenantVisionProvider: "mistral",
      tenantVisionModel: "mistral-small-latest",
      envProvider: "ollama",
      envModel: "gemma4:latest",
    });
    expect(r.provider).toBe("mistral");
    expect(r.model).toBe("mistral-small-latest");
    expect(r.source).toBe("tenant");
  });
});

describe("SPEC-123 effective*FromWorkspace prefers API resolved_*", () => {
  test("effectiveLlmFromWorkspace uses resolved fields + source", () => {
    const r = effectiveLlmFromWorkspace({
      llm_provider: "ollama",
      llm_model: "painted",
      resolved_llm_provider: "mistral",
      resolved_llm_model: "mistral-small-latest",
      llm_resolution_source: "tenant",
    });
    expect(r.provider).toBe("mistral");
    expect(r.model).toBe("mistral-small-latest");
    expect(r.source).toBe("tenant");
  });

  test("effectiveEmbeddingFromWorkspace uses resolved fields + source", () => {
    const r = effectiveEmbeddingFromWorkspace({
      embedding_provider: "ollama",
      embedding_model: "painted",
      embedding_dimension: 768,
      resolved_embedding_provider: "openai",
      resolved_embedding_model: "text-embedding-3-small",
      resolved_embedding_dimension: 1536,
      embedding_resolution_source: "tenant",
    });
    expect(r.provider).toBe("openai");
    expect(r.dimension).toBe(1536);
    expect(r.source).toBe("tenant");
  });
});
