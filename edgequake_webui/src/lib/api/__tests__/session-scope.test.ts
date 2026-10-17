import { describe, expect, it } from "vitest";
import { needsScopeSwitch, tokenScope } from "../session-scope";

const T = "00000000-0000-0000-0000-000000000002";
const W = "00000000-0000-0000-0000-000000000003";
const jwt = (claims: object) =>
  `h.${btoa(JSON.stringify(claims)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "")}.s`;

describe("session scope", () => {
  it("reads tenant/workspace claims and tolerates garbage", () => {
    expect(tokenScope(jwt({ tenant_id: T, workspace_id: W }))).toEqual({ tenantId: T, workspaceId: W });
    expect(tokenScope("not-a-jwt")).toEqual({});
    expect(tokenScope(null)).toEqual({});
  });

  it("switches only when the selected scope differs from the token", () => {
    const claims = { tenantId: T, workspaceId: W };
    expect(needsScopeSwitch(claims, T, W)).toBe(false);
    expect(needsScopeSwitch(claims, "default", "default")).toBe(false);
    expect(needsScopeSwitch(claims, T.toUpperCase(), null)).toBe(false);
    expect(needsScopeSwitch(claims, T, "55a78878-845c-4abe-a8ba-8ceb9d067b90")).toBe(true);
    expect(needsScopeSwitch(claims, "f62e6a67-9a75-4654-b2b6-1a6168bda5ba", null)).toBe(true);
    // Unscoped token (API key / auth off) or nothing selected: never switch.
    expect(needsScopeSwitch({}, T, W)).toBe(false);
    expect(needsScopeSwitch(claims, null, null)).toBe(false);
  });
});

describe("serializeAuth", () => {
  it("never overlaps refresh-token mutations, and starts at once when idle", async () => {
    const { serializeAuth } = await import("../client-context");
    const log: string[] = [];
    let releaseFirst!: () => void;
    const first = serializeAuth(async () => {
      log.push("first:start");
      await new Promise<void>((r) => (releaseFirst = r));
      log.push("first:end");
    });
    expect(log).toEqual(["first:start"]); // synchronous start when idle
    const second = serializeAuth(async () => {
      log.push("second:start");
    });
    await Promise.resolve();
    expect(log).toEqual(["first:start"]); // queued, not overlapping
    releaseFirst();
    await Promise.all([first, second]);
    expect(log).toEqual(["first:start", "first:end", "second:start"]);
  });
});
