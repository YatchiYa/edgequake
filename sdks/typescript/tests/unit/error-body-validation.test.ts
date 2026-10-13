import { describe, expect, it, vi } from "vitest";
import { FetchTransport } from "../../src/transport/fetch.js";
import { BadRequestError, InternalError } from "../../src/errors.js";

function request(body: unknown, status: number, statusText = "") {
  const fetchFn = vi.fn(async () => new Response(JSON.stringify(body), {
    status, statusText, headers: { "Content-Type": "application/json" },
  })) as unknown as typeof fetch;
  return new FetchTransport({ baseUrl: "http://localhost:8080", headers: {}, timeout: 0,
    maxRetries: 0, retryDelay: 0, retryStatusCodes: [], fetchFn,
  }).request({ method: "GET", path: "/query" });
}

describe("HTTP error JSON shape validation", () => {
  it.each([null, [], "proxy failure", 42, true, {}, { message: 123 }])(
    "maps malformed JSON payload %j to the HTTP status error", async (body) => {
      await expect(request(body, 400, "Bad Request")).rejects.toMatchObject({
        name: "BadRequestError", message: "Bad Request", status: 400,
      });
    },
  );
  it("falls back to HTTP status when there is no status text", async () => {
    await expect(request(null, 500)).rejects.toMatchObject({
      name: "InternalError", message: "HTTP 500", status: 500,
    });
  });
  it.each([[400, BadRequestError], [500, InternalError]])(
    "preserves valid error diagnostics for status %s", async (status, errorClass) => {
      const error = await request({ code: "QUERY_REJECTED", message: "query rejected", details: { field: "query" } }, status as number).catch((error) => error);
      expect(error).toBeInstanceOf(errorClass);
      expect(error.message).toBe("query rejected");
      expect(error.details).toEqual({ field: "query" });
    },
  );
});
