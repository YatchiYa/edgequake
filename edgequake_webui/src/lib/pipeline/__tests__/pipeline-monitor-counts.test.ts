import { describe, expect, it } from "vitest";

import { pipelineDocumentsQueryKey } from "@/hooks/use-pipeline-documents";
import {
  activeDocumentCount,
  hiddenPreviewCount,
} from "../pipeline-monitor-counts";

describe("pipeline monitor aggregate counts", () => {
  it("reports aggregate active counts beyond the returned preview page", () => {
    const active = activeDocumentCount({
      pending: 60,
      processing: 2,
      completed: 100,
      partial_failure: 0,
      failed: 1,
      cancelled: 0,
    });

    expect(active).toBe(62);
    expect(hiddenPreviewCount(active, 50)).toBe(12);
  });

  it("isolates cache entries by scope and all query variables", () => {
    const base = pipelineDocumentsQueryKey("tenant-a", "workspace-a", 1, 50);
    expect(base).not.toEqual(
      pipelineDocumentsQueryKey("tenant-a", "workspace-b", 1, 50),
    );
    expect(base).not.toEqual(
      pipelineDocumentsQueryKey("tenant-a", "workspace-a", 2, 50),
    );
    expect(base).not.toEqual(
      pipelineDocumentsQueryKey("tenant-a", "workspace-a", 1, 50, "pending"),
    );
  });
});
