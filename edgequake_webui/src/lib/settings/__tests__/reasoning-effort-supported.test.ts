import { describe, expect, it } from "vitest";
import type { ModelResponse } from "@/lib/api/models";
import {
  effectiveEffortWhenAuto,
  formatAutoEffortLabel,
  formatEffectiveBestPracticeHint,
  lowestStructuredEffortForModel,
  supportedReasoningEffortsForModel,
} from "@/lib/settings/reasoning-effort-supported";

function model(
  provider: string,
  name: string,
  reasoning?: { supported: string[]; lowest_structured?: string | null },
): ModelResponse {
  return {
    name,
    display_name: name,
    model_type: "llm",
    provider,
    provider_display_name: provider,
    description: "",
    deprecated: false,
    capabilities: {
      context_length: 128000,
      max_output_tokens: 8192,
      supports_vision: false,
      supports_function_calling: true,
      supports_json_mode: true,
      supports_streaming: true,
      supports_system_message: true,
      embedding_dimension: 0,
      reasoning_effort: reasoning ?? null,
    },
    tags: [],
  };
}

describe("reasoning-effort-supported best-practice effective", () => {
  const catalog = [
    model("openai", "gpt-5-mini", {
      supported: ["minimal", "low", "medium", "high"],
      lowest_structured: "minimal",
    }),
    model("openai", "gpt-5.4-nano", {
      supported: ["none", "low", "medium", "high"],
      lowest_structured: "none",
    }),
    model("mistral", "mistral-small-latest"),
  ];

  it("reads supported + lowest_structured from catalog", () => {
    expect(
      supportedReasoningEffortsForModel(catalog, "openai", "gpt-5-mini"),
    ).toEqual(["minimal", "low", "medium", "high"]);
    expect(
      lowestStructuredEffortForModel(catalog, "openai", "gpt-5-mini"),
    ).toBe("minimal");
    expect(
      lowestStructuredEffortForModel(catalog, "openai", "gpt-5.4-nano"),
    ).toBe("none");
  });

  it("structured/fleet Auto → lowest structured (best practice)", () => {
    expect(
      effectiveEffortWhenAuto(catalog, "openai", "gpt-5-mini", "structured"),
    ).toBe("minimal");
    expect(
      effectiveEffortWhenAuto(catalog, "openai", "gpt-5-mini", "fleet"),
    ).toBe("minimal");
    expect(
      effectiveEffortWhenAuto(catalog, "openai", "gpt-5.4-nano", "structured"),
    ).toBe("none");
  });

  it("query Auto → omit (provider default)", () => {
    expect(
      effectiveEffortWhenAuto(catalog, "openai", "gpt-5-mini", "query"),
    ).toBe("omit");
  });

  it("non-reasoning models Auto → omit", () => {
    expect(
      effectiveEffortWhenAuto(
        catalog,
        "mistral",
        "mistral-small-latest",
        "structured",
      ),
    ).toBe("omit");
  });

  it("formats Auto label and helper with effective best practice", () => {
    expect(formatAutoEffortLabel("minimal")).toContain("effective: minimal");
    expect(formatAutoEffortLabel("minimal")).toContain("best practice");
    expect(formatAutoEffortLabel("omit")).toContain("provider default");
    expect(formatEffectiveBestPracticeHint("minimal")).toBe(
      "Best practice when Auto: minimal",
    );
    expect(formatEffectiveBestPracticeHint("omit")).toContain("omit field");
  });
});
