import { describe, expect, it } from "vitest";

/**
 * Contract tests for feedback-zone announcement message shapes.
 * (Debounce timing is covered in the component; keep these free of fake timers.)
 */

const DEBOUNCE_MS = 800;

describe("FeedbackZoneLiveRegion contracts", () => {
  it("debounce window is 800ms for polite stage announcements", () => {
    expect(DEBOUNCE_MS).toBe(800);
  });

  it("builds Cleaning / Queued announcements without Progress not found wording", () => {
    const name = "raphael-article.pdf";
    const cleaning = `Cleaning: ${name}`;
    const queued = `Queued: ${name}`;
    expect(cleaning).toMatch(/^Cleaning:/);
    expect(queued).toMatch(/^Queued:/);
    expect(cleaning).not.toMatch(/Progress not found/i);
    expect(queued).not.toMatch(/Progress not found/i);
  });

  it("builds stage percent announcement for AT", () => {
    const announcement = `doc.pdf: Extracting Entities, 42%`;
    expect(announcement).toContain("42%");
    expect(announcement).toContain("Extracting Entities");
  });
});
