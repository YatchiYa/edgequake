/**
 * 068 — early store hydration so WS ChunkProgress is not dropped before poll.
 *
 * Hook effects require @testing-library/react (not in this package's vitest
 * env). We assert the store contract that `useIngestionProgress` calls via
 * `startTracking` before the first successful poll.
 */
import { act } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { useIngestionStore } from "@/stores/use-ingestion-store";

describe("useIngestionProgress 068 hydrate contract", () => {
  beforeEach(() => {
    useIngestionStore.getState().clearAllTracks();
  });

  it("startTracking seeds queued/uploading before poll success", () => {
    const store = useIngestionStore.getState();

    act(() => {
      store.startTracking("insert-abc", "doc-1", "notes.md");
    });

    const track = useIngestionStore.getState().getTrack("insert-abc");
    expect(track).toBeTruthy();
    expect(track?.document_name).toBe("notes.md");
    expect(track?.progress.current_stage).toBe("uploading");
    expect(track?.progress.latest_message).toMatch(/Queued/i);
    expect(track?.progress.latest_message).not.toMatch(/Processing pending/i);
  });
});
