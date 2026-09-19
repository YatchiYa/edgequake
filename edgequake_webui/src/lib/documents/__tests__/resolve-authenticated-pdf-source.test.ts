/**
 * Authenticated PDF URL resolution (PDF viewer 401 fix).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  extractPdfSourceUrl,
  fetchAuthenticatedPdfBlobUrl,
  isApiProtectedPdfUrl,
} from "../resolve-authenticated-pdf-source";

vi.mock("@/lib/api/client", () => ({
  buildHeaders: () => {
    const h = new Headers();
    h.set("Authorization", "Bearer test-token-149");
    h.set("Content-Type", "application/json");
    h.set("X-Workspace-ID", "ws-1");
    return h;
  },
}));

describe("resolve-authenticated-pdf-source", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        const bytes = new Uint8Array([0x25, 0x50, 0x44, 0x46]); // %PDF
        return new Response(bytes, {
          status: 200,
          headers: { "Content-Type": "application/pdf" },
        });
      }),
    );
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: vi.fn(() => "blob:mock-pdf-url"),
      revokeObjectURL: vi.fn(),
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("detects protected download URLs", () => {
    expect(
      isApiProtectedPdfUrl(
        "https://demo.edgequake.com/api/v1/documents/pdf/abc/download",
      ),
    ).toBe(true);
    expect(
      isApiProtectedPdfUrl(
        "/api/v1/documents/01a0/download/original",
      ),
    ).toBe(true);
    expect(isApiProtectedPdfUrl("https://cdn.example/file.pdf")).toBe(false);
    expect(isApiProtectedPdfUrl("blob:http://localhost/x")).toBe(false);
  });

  it("extracts url from string or {url} sources", () => {
    expect(extractPdfSourceUrl("https://x/api/v1/documents/pdf/a/download")).toBe(
      "https://x/api/v1/documents/pdf/a/download",
    );
    expect(
      extractPdfSourceUrl({ url: "/api/v1/documents/pdf/a/download" }),
    ).toBe("/api/v1/documents/pdf/a/download");
    expect(extractPdfSourceUrl({ data: new Uint8Array([1]) })).toBeNull();
    expect(extractPdfSourceUrl(null)).toBeNull();
  });

  it("fetches with Authorization and returns a blob URL", async () => {
    const blobUrl = await fetchAuthenticatedPdfBlobUrl(
      "https://demo.edgequake.com/api/v1/documents/pdf/ead6/download",
    );

    expect(blobUrl).toBe("blob:mock-pdf-url");
    expect(fetch).toHaveBeenCalledTimes(1);
    const [url, init] = vi.mocked(fetch).mock.calls[0]!;
    expect(url).toContain("/documents/pdf/ead6/download");
    expect(init?.method).toBe("GET");
    const headers = init?.headers as Headers;
    expect(headers.get("Authorization")).toBe("Bearer test-token-149");
    expect(headers.get("Content-Type")).toBeNull();
    expect(headers.get("X-Workspace-ID")).toBe("ws-1");
    expect(URL.createObjectURL).toHaveBeenCalled();
  });

  it("maps non-OK responses to ResponseException", async () => {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response("nope", { status: 401 }),
    );
    await expect(
      fetchAuthenticatedPdfBlobUrl("/api/v1/documents/pdf/x/download"),
    ).rejects.toThrow("ResponseException: Unexpected server response (401)");
  });
});
