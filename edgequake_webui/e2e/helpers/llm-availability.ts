/**
 * LLM provider availability probes for conditional live-stack E2E.
 * @implements SPEC-020 — single place for Ollama/OpenAI reachability (DRY).
 */

const OLLAMA_HOST = process.env.OLLAMA_HOST ?? "http://localhost:11434";

export async function isOllamaAvailable(): Promise<boolean> {
  try {
    const res = await fetch(`${OLLAMA_HOST}/api/tags`);
    return res.ok;
  } catch {
    return false;
  }
}

export async function resolveOllamaLlmModel(): Promise<string> {
  if (process.env.SPEC020_OLLAMA_MODEL) {
    return process.env.SPEC020_OLLAMA_MODEL;
  }
  try {
    const res = await fetch(`${OLLAMA_HOST}/api/tags`);
    if (!res.ok) return "gemma4:latest";
    const body = (await res.json()) as { models?: Array<{ name: string }> };
    const names = (body.models ?? []).map((m) => m.name);
    const preferred = ["gemma4:latest", "gemma4:e4b", "gemma3:latest", "qwen3.5:latest"];
    for (const p of preferred) {
      if (names.includes(p)) return p;
    }
    const first = names.find((n) => !n.includes("embed") && !n.includes("ocr"));
    return first ?? "gemma4:latest";
  } catch {
    return "gemma4:latest";
  }
}

export const OLLAMA_EMBEDDING_MODEL =
  process.env.OLLAMA_EMBEDDING_MODEL ?? "embeddinggemma:latest";

type PlaywrightTest = {
  skip: (condition: boolean, description?: string) => void;
};

/**
 * Skip when Ollama is down; fail instead when SPEC020_REQUIRE_OLLAMA=1 (prod gate).
 */
export async function guardOllamaAvailability(test: PlaywrightTest): Promise<boolean> {
  const up = await isOllamaAvailable();
  if (!up && process.env.SPEC020_REQUIRE_OLLAMA === "1") {
    throw new Error(
      "SPEC020_REQUIRE_OLLAMA=1 but Ollama is unreachable — start Ollama or unset the flag",
    );
  }
  test.skip(!up, "Ollama not reachable — skip live LLM proof");
  return up;
}
