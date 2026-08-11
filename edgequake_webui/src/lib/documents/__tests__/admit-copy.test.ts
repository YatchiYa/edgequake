/**
 * SPEC-122 — admit-copy pure builders (LAW-122-1).
 */
import { describe, expect, it } from "vitest";
import {
  ADMIT_FORBIDDEN_CLAIM_RE,
  admitPartialMessage,
  admitSuccessMessage,
  bulkIngestBannerLine,
  concurrencyLaneHint,
  shouldShowBulkBanner,
  transferCompleteHeader,
} from "../admit-copy";

describe("admit-copy (SPEC-122 LAW-122-1)", () => {
  it("U1: admit success for N=1 and N=3 never claims ready/searchable", () => {
    for (const n of [1, 3]) {
      const msg = admitSuccessMessage(n);
      expect(msg.toLowerCase()).toContain("admitted");
      expect(msg.toLowerCase()).toContain("queued");
      expect(msg).not.toMatch(ADMIT_FORBIDDEN_CLAIM_RE);
    }
  });

  it("U1: serial hint only when tenant lane is 1", () => {
    const serial = concurrencyLaneHint(1);
    expect(serial).toMatch(/one document at a time/i);
    expect(serial).not.toMatch(/local LLM/i);
    expect(serial).not.toMatch(ADMIT_FORBIDDEN_CLAIM_RE);
    expect(concurrencyLaneHint(6)).toMatch(/up to 6/i);
    expect(concurrencyLaneHint(null)).toBeNull();
    expect(concurrencyLaneHint(undefined)).toBeNull();
    expect(concurrencyLaneHint(0)).toBeNull();
  });

  it("U1: admit copy never pollutes with query/graph unavailability", () => {
    for (const msg of [
      admitSuccessMessage(2),
      admitPartialMessage(1, 1),
      transferCompleteHeader(),
      concurrencyLaneHint(1)!,
      bulkIngestBannerLine({
        pending: 1,
        processing: 1,
        completed: 0,
        maxTasksPerTenant: 1,
      })!,
    ]) {
      expect(msg).not.toMatch(ADMIT_FORBIDDEN_CLAIM_RE);
      expect(msg.toLowerCase()).not.toContain("view in graph");
      expect(msg.toLowerCase()).not.toContain("not available");
    }
  });

  it("U2: banner builder empty when no pending/processing", () => {
    expect(
      bulkIngestBannerLine({
        pending: 0,
        processing: 0,
        completed: 5,
        maxTasksPerTenant: 1,
      }),
    ).toBeNull();
    expect(shouldShowBulkBanner(0, 0)).toBe(false);
  });

  it("U2: banner shows counts and serial/parallel clause", () => {
    const serial = bulkIngestBannerLine({
      pending: 2,
      processing: 1,
      completed: 3,
      maxTasksPerTenant: 1,
    });
    expect(serial).toContain("Processing 1");
    expect(serial).toContain("2 queued");
    expect(serial).toContain("3 completed");
    expect(serial).toMatch(/one document at a time/i);

    const parallel = bulkIngestBannerLine({
      pending: 1,
      processing: 2,
      completed: 0,
      maxTasksPerTenant: 6,
    });
    expect(parallel).toMatch(/up to 6/i);

    const noHint = bulkIngestBannerLine({
      pending: 1,
      processing: 0,
      completed: 0,
    });
    expect(noHint).toBe("Processing 0 · 1 queued · 0 completed");
  });

  it("partial + transfer header stay honest", () => {
    expect(admitPartialMessage(2, 1)).not.toMatch(ADMIT_FORBIDDEN_CLAIM_RE);
    expect(admitPartialMessage(2, 1).toLowerCase()).toContain("admitted");
    expect(transferCompleteHeader().toLowerCase()).toContain("processing queued");
    expect(transferCompleteHeader()).not.toMatch(ADMIT_FORBIDDEN_CLAIM_RE);
  });
});
