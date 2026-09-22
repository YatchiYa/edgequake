/**
 * SPEC-149 U-149-20 — authenticated PDF SSE uses Authorization header.
 */
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@/lib/runtime-config", () => ({
  getRuntimeApiBaseUrl: () => "http://api.test/api/v1",
}));

vi.mock("../client", () => ({
  buildHeaders: (extra?: HeadersInit) => {
    const headers = new Headers(extra);
    headers.set("Authorization", "Bearer test-token");
    headers.set("Content-Type", "application/json");
    return headers;
  },
  handleErrorResponse: async (response: Response) => {
    throw new Error(`HTTP ${response.status}`);
  },
}));

import { parseSSEFrame, streamClientFrames } from "../stream-client";

describe("stream-client SPEC-149", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("parseSSEFrame preserves event names", () => {
    const frame = parseSSEFrame('event: progress\ndata: {"ok":true}');
    expect(frame).toEqual({ event: "progress", data: { ok: true } });
  });

  it("U-149-20 sends Authorization on SSE fetch", async () => {
    const chunks = [
      new TextEncoder().encode('event: progress\ndata: {"page":1}\n\n'),
      new TextEncoder().encode('event: complete\ndata: {"done":true}\n\n'),
    ];
    let i = 0;
    const reader = {
      read: vi.fn(async () => {
        if (i >= chunks.length) return { done: true, value: undefined };
        return { done: false, value: chunks[i++] };
      }),
      releaseLock: vi.fn(),
    };

    const fetchMock = vi.fn(async (_url: string, init?: RequestInit) => {
      expect(new Headers(init?.headers).get("Authorization")).toBe(
        "Bearer test-token",
      );
      return {
        ok: true,
        body: { getReader: () => reader },
      } as unknown as Response;
    });
    vi.stubGlobal("fetch", fetchMock);

    const frames: Array<{ event: string; data: unknown }> = [];
    for await (const frame of streamClientFrames("/documents/pdf/progress/stream/t1")) {
      frames.push(frame);
    }

    expect(fetchMock).toHaveBeenCalled();
    expect(frames[0]?.event).toBe("progress");
    expect(frames[1]?.event).toBe("complete");
  });
});
