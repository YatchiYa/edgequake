import { describe, expect, it } from "vitest";
import {
  ENTITY_TYPE_COLORS,
  formatEntityLabel,
  formatMmEntitySubtitle,
  getEntityTypeColor,
  isMmItemId,
} from "./label-utils";

describe("label-utils 066 drawing display", () => {
  it("preserves human display names from Graph API", () => {
    const label = "Architecture overview · p.2 · Fig 1";
    expect(formatEntityLabel(label, 80)).toBe(label);
  });

  it("does not title-case opaque im- ids", () => {
    const raw =
      "IM-019F7028-D3E3-7684-8B3B-A9259368329A-PAGE-0002-FIG-01";
    const formatted = formatEntityLabel(raw, 48);
    expect(formatted.toLowerCase()).toContain("page-0002");
    expect(formatted).not.toMatch(/^Im-/);
  });

  it("detects mm item ids", () => {
    expect(isMmItemId("im-page-0001-fig-01")).toBe(true);
    expect(
      isMmItemId(
        "00000000-0000-0000-0000-000000000003::IM-PAGE-0001-FIG-01",
      ),
    ).toBe(true);
    expect(isMmItemId("SARAH_CHEN")).toBe(false);
  });

  it("has DRAWING / TABLE / EQUATION colors", () => {
    expect(ENTITY_TYPE_COLORS.DRAWING).toBeTruthy();
    expect(getEntityTypeColor("drawing")).toBe(ENTITY_TYPE_COLORS.DRAWING);
    expect(getEntityTypeColor("table")).toBe(ENTITY_TYPE_COLORS.TABLE);
    expect(getEntityTypeColor("equation")).toBe(ENTITY_TYPE_COLORS.EQUATION);
  });

  it("formats mm subtitle with page and fig", () => {
    expect(
      formatMmEntitySubtitle("drawing", {
        page_num: 2,
        figure_index: 1,
        mm_subtype: "Flowchart",
      }),
    ).toBe("Drawing · Flowchart · p.2 · Fig 1");
  });
});
