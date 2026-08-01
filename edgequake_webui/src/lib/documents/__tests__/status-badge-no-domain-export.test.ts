/**
 * SPEC-099 F-099-01 / LAW-099-1: status-badge is presentation-only.
 * Domain helpers must not be re-exported from the badge module.
 */
import { describe, expect, it } from "vitest";
import * as StatusBadgeModule from "@/components/documents/status-badge";

const FORBIDDEN_DOMAIN_EXPORTS = [
  "normalizeStatus",
  "getDocumentDisplayStatus",
  "isProcessingStatus",
  "isTerminalStatus",
  "documentStageRank",
  "isWaitingDocumentStage",
  "isActiveDocumentStage",
] as const;

describe("spec099-status-domain-single-import", () => {
  it("does not export domain helpers from status-badge", () => {
    for (const name of FORBIDDEN_DOMAIN_EXPORTS) {
      expect(
        Object.prototype.hasOwnProperty.call(StatusBadgeModule, name),
        `status-badge must not export ${name}`,
      ).toBe(false);
    }
  });

  it("still exports the StatusBadge presentation component", () => {
    // memo() wraps the component; accept function or memo exotic object
    expect(StatusBadgeModule.StatusBadge).toBeDefined();
    expect(
      typeof StatusBadgeModule.StatusBadge === "function" ||
        typeof StatusBadgeModule.StatusBadge === "object",
    ).toBe(true);
  });
});
