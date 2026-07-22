import { describe, expect, it } from "vitest";
import {
  displayEntityLabel,
  ENTITY_TYPE_COLORS,
  formatEntityLabel,
  formatMmEntitySubtitle,
  getEntityTypeColor,
  isMmItemId,
  isOpaqueIdentifier,
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

describe("label-utils 067 opaque identifiers", () => {
  it("detects UUID / GUID shapes", () => {
    expect(
      isOpaqueIdentifier("84b69e27-e38b-444a-83dd-5e6a537c6f12"),
    ).toBe(true);
    expect(
      isOpaqueIdentifier(
        "00000000-0000-0000-0000-000000000003::84B69E27-E38B-444A-83DD-5E6A537C6F12",
      ),
    ).toBe(true);
    expect(isOpaqueIdentifier("ACME_CORP")).toBe(false);
    expect(isOpaqueIdentifier("im-page-0001-fig-01")).toBe(false);
  });

  it("does not title-case raw UUID labels", () => {
    const uuid = "84b69e27-e38b-444a-83dd-5e6a537c6f12";
    const formatted = formatEntityLabel(uuid, 35);
    expect(formatted).not.toMatch(/^84b69e27-E38b/);
    expect(formatted.includes("…") || formatted === uuid).toBe(true);
  });

  it("preserves Opaque ID soft-labels from API", () => {
    expect(formatEntityLabel("Opaque ID · ORGANIZATION", 40)).toBe(
      "Opaque ID · ORGANIZATION",
    );
  });
});

describe("label-utils 072 prefixed opaque identifiers", () => {
  it("detects RESOURCE_UUID and org:uuid shapes", () => {
    expect(
      isOpaqueIdentifier(
        "RESOURCE_84B69E27-E38B-444A-83DD-5E6A537C6F12",
      ),
    ).toBe(true);
    expect(
      isOpaqueIdentifier("org:84b69e27-e38b-444a-83dd-5e6a537c6f12"),
    ).toBe(true);
    expect(
      isOpaqueIdentifier("uuid:84b69e27-e38b-444a-83dd-5e6a537c6f12"),
    ).toBe(true);
    expect(isOpaqueIdentifier("ACME_CORP")).toBe(false);
    expect(isOpaqueIdentifier("Room 84b6")).toBe(false);
  });
});

describe("label-utils 073 displayEntityLabel", () => {
  it("prefers API soft-label over opaque id", () => {
    expect(
      displayEntityLabel({
        label: "Future of work theme from the agenda",
        id: "84b69e27-e38b-444a-83dd-5e6a537c6f12",
        maxLen: 80,
      }),
    ).toContain("Future of work");
  });

  it("falls back to formatted id when label missing", () => {
    const out = displayEntityLabel({
      id: "SARAH_CHEN",
      maxLen: 40,
    });
    expect(out).toContain("Sarah");
  });
});
