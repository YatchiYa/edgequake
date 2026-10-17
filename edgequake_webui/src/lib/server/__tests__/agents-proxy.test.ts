import { describe, expect, it } from "vitest";
import { agentsBaseUrl, buildAgentsHeaders, upstreamPath } from "../agents-proxy";

const user = { user_id: "u-1", username: "alice", role: "admin" };

describe("buildAgentsHeaders", () => {
  it("builds identity from /auth/me and drops browser-supplied identity headers", () => {
    const incoming = new Headers({
      "Content-Type": "application/json",
      "X-Tenant-ID": "t-1",
      "X-Workspace-ID": "w-1",
      "X-EQ-User-Id": "attacker",
      "X-EQ-User-Name": "root",
      "X-EQ-User-Role": "admin",
      "X-Agents-Internal-Token": "guess",
      "X-User-ID": "spoofed",
      Cookie: "eq_refresh=secret",
      Authorization: "Bearer browser-other",
    });
    const h = buildAgentsHeaders(incoming, { ...user, role: "viewer" }, "Bearer tok", "internal");
    expect(h.get("x-eq-user-id")).toBe("u-1");
    expect(h.get("x-eq-user-name")).toBe("alice");
    expect(h.get("x-eq-user-role")).toBe("viewer");
    expect(h.get("x-agents-internal-token")).toBe("internal");
    expect(h.get("authorization")).toBe("Bearer tok");
    expect(h.get("x-tenant-id")).toBe("t-1");
    expect(h.get("x-workspace-id")).toBe("w-1");
    expect(h.get("content-type")).toBe("application/json");
    expect(h.get("x-user-id")).toBeNull();
    expect(h.get("cookie")).toBeNull();
  });

  it("percent-encodes non-ASCII usernames (header ByteString)", () => {
    const h = buildAgentsHeaders(new Headers(), { ...user, username: "Zoé 李" }, "Bearer t", "i");
    expect(decodeURIComponent(h.get("x-eq-user-name")!)).toBe("Zoé 李");
  });
});

describe("upstreamPath", () => {
  it("maps segments under /v1 and re-encodes them", () => {
    expect(upstreamPath(["conversations", "c1", "artifacts", "a b.png"])).toBe("/v1/conversations/c1/artifacts/a%20b.png");
    expect(upstreamPath(["x/../../admin"])).toBe("/v1/x%2F..%2F..%2Fadmin");
  });
  it("rejects traversal and empty paths", () => {
    expect(upstreamPath(undefined)).toBeNull();
    expect(upstreamPath(["..", "admin"])).toBeNull();
    expect(upstreamPath(["agents", "."])).toBeNull();
  });
});

describe("agentsBaseUrl", () => {
  it("defaults to the local agents port and trims a trailing slash", () => {
    expect(agentsBaseUrl({})).toBe("http://127.0.0.1:8765");
    expect(agentsBaseUrl({ AGENTS_INTERNAL_URL: "http://agents:8765/" })).toBe("http://agents:8765");
  });
});
