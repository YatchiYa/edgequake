/**
 * Unit coverage for ModelPickerPanel helpers + provider subtitles.
 */
import { describe, expect, it } from "bun:test";
import {
  formatModelFullId,
  parseModelFullId,
} from "@/components/models/model-picker-panel";
import {
  getProviderDisplayName,
  getProviderSubtitle,
} from "@/lib/provider-display";

describe("ModelPickerPanel id helpers", () => {
  it("formats and parses provider/model full ids", () => {
    expect(formatModelFullId("openai", "gpt-4.1-nano")).toBe("openai/gpt-4.1-nano");
    expect(parseModelFullId("openai/gpt-4.1-nano")).toEqual({
      provider: "openai",
      model: "gpt-4.1-nano",
    });
  });

  it("keeps colon inside ollama model segment", () => {
    expect(parseModelFullId("ollama/gemma4:latest")).toEqual({
      provider: "ollama",
      model: "gemma4:latest",
    });
  });

  it("falls back when slash is missing", () => {
    expect(parseModelFullId("orphan-model")).toEqual({
      provider: "unknown",
      model: "orphan-model",
    });
  });
});

describe("getProviderSubtitle (two-step UX)", () => {
  it("labels gateways and local runtimes", () => {
    expect(getProviderSubtitle("openrouter")).toBe("Multi-model gateway");
    expect(getProviderSubtitle("ollama")).toBe("Local");
    expect(getProviderSubtitle("lmstudio")).toBe("Local");
    expect(getProviderDisplayName("openrouter")).toBe("OpenRouter");
  });

  it("returns null for standard cloud APIs", () => {
    expect(getProviderSubtitle("openai")).toBeNull();
    expect(getProviderSubtitle("anthropic")).toBeNull();
  });
});
