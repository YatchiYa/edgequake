import { describe, expect, it } from "vitest";

import { shouldUsePdfReprocessPanel } from "../use-reprocess-tracking";

describe("shouldUsePdfReprocessPanel", () => {
  it("uses PdfUploadProgress only for full PDF reprocess", () => {
    expect(shouldUsePdfReprocessPanel(true, "full")).toBe(true);
  });

<<<<<<< HEAD
  it("uses IngestionProgressPanel for entities-only PDF", () => {
    expect(shouldUsePdfReprocessPanel(true, "entities")).toBe(false);
  });

  it("uses IngestionProgressPanel for non-PDF full mode", () => {
=======
  it("uses IngestionRunCard (no PDF nest) for entities-only PDF", () => {
    expect(shouldUsePdfReprocessPanel(true, "entities")).toBe(false);
  });

  it("uses IngestionRunCard (no PDF nest) for non-PDF full mode", () => {
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    expect(shouldUsePdfReprocessPanel(false, "full")).toBe(false);
  });

  it("defaults safely when mode is missing", () => {
    expect(shouldUsePdfReprocessPanel(true, undefined)).toBe(false);
  });
});
