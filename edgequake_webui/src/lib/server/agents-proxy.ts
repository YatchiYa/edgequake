/**
 * SPEC-165 — pure helpers for the `/api/agents/[...path]` proxy (unit-tested).
 *
 * The browser is untrusted: identity headers (X-EQ-User-*) are built ONLY from the
 * backend's `/auth/me` answer, and only an allowlist of request headers is forwarded.
 */

export const MAX_AGENTS_BODY_BYTES = 256 * 1024; // = the agents service cap for non-message bodies

export interface VerifiedUser {
  user_id: string;
  username: string;
  role: string;
}

export function agentsBaseUrl(env: Record<string, string | undefined> = process.env): string {
  return (env.AGENTS_INTERNAL_URL?.trim() || "http://127.0.0.1:8765").replace(/\/$/, "");
}

/** Browser headers we pass through as-is (everything else is dropped). */
const FORWARDED = ["content-type", "accept", "accept-language", "x-request-id", "traceparent", "x-tenant-id", "x-workspace-id"];

export function buildAgentsHeaders(
  incoming: Headers,
  user: VerifiedUser,
  authorization: string,
  internalToken: string,
): Headers {
  const out = new Headers();
  for (const name of FORWARDED) {
    const value = incoming.get(name);
    if (value) out.set(name, value);
  }
  out.set("Authorization", authorization);
  out.set("X-Agents-Internal-Token", internalToken);
  out.set("X-EQ-User-Id", user.user_id);
  // Header values must be ByteStrings: printable ASCII as-is, anything else percent-encoded.
  out.set("X-EQ-User-Name", /^[\x20-\x7e]*$/.test(user.username) ? user.username : encodeURIComponent(user.username));
  out.set("X-EQ-User-Role", user.role);
  return out;
}

/**
 * Upstream `/v1/<path>` from the catch-all segments; null when a segment tries to
 * escape (`..`, `.`, empty) — Next already decodes segments, so re-encode each one.
 */
export function upstreamPath(segments: string[] | undefined): string | null {
  if (!segments?.length) return null;
  if (segments.some((s) => !s || s === "." || s === "..")) return null;
  return `/v1/${segments.map(encodeURIComponent).join("/")}`;
}

/** Response headers worth passing back (no content-length/-encoding: fetch already decoded the body). */
export const PASSED_BACK = ["content-type", "content-disposition", "cache-control", "etag", "last-modified"];
