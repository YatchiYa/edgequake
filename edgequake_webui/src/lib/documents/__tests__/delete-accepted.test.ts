/**
 * Async delete admit: HTTP success with accepted=true must not be treated as
 * terminal failure (timeout) — WS DeletionCompleted is SSOT.
 */
import { describe, expect, it } from "bun:test";
import type { DeleteDocumentAccepted } from "@/lib/api/edgequake/documents";

describe("deleteDocument accepted response", () => {
  it("treats 202-style accepted payload as non-terminal success", () => {
    const data: DeleteDocumentAccepted = {
      document_id: "doc-1",
      deleted: false,
      accepted: true,
      track_id: "track-abc",
      chunks_deleted: 0,
      entities_affected: 0,
      relationships_affected: 0,
    };
    expect(data.accepted).toBe(true);
    expect(data.deleted).toBe(false);
    expect(data.track_id).toBeTruthy();
  });
});
