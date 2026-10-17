/**
 * SPEC-165 — same-origin proxy to the internal agents service (`/v1/...`).
 *
 * 1. Requires the browser's bearer and verifies it with GET /api/v1/auth/me (fail closed).
 * 2. Forwards to `${AGENTS_INTERNAL_URL}/v1/<path>` with the shared internal token and
 *    identity headers built from /auth/me — never from the browser.
 * 3. Streams the upstream body back untouched (SSE token-by-token, binary artifacts).
 *    Client disconnect aborts the upstream request (which cancels the agent run).
 */
import { backendBaseUrl } from "@/lib/server/copilot/tools";
import {
  agentsBaseUrl,
  buildAgentsHeaders,
  MAX_AGENTS_BODY_BYTES,
  PASSED_BACK,
  upstreamPath,
  type VerifiedUser,
} from "@/lib/server/agents-proxy";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

type Context = { params: Promise<{ path?: string[] }> };

const json = (error: string, status: number) => Response.json({ error }, { status });

async function handle(request: Request, { params }: Context): Promise<Response> {
  const path = upstreamPath((await params).path);
  if (!path) return json("Not found", 404);

  const authorization = request.headers.get("authorization");
  if (!authorization?.toLowerCase().startsWith("bearer ")) return json("Authentication required", 401);

  const internalToken = process.env.AGENTS_INTERNAL_TOKEN?.trim();
  if (!internalToken) return json("Agents service is not configured (AGENTS_INTERNAL_TOKEN).", 503);

  // Fail closed: only a caller the EdgeQuake API positively recognizes reaches the agents service.
  const me = await fetch(`${backendBaseUrl()}/api/v1/auth/me`, {
    headers: { Authorization: authorization },
    signal: AbortSignal.timeout(10_000),
  }).catch(() => null);
  if (!me?.ok) {
    const status = me?.status === 401 || me?.status === 403 ? 401 : 503;
    return json(status === 401 ? "Authentication required" : "Backend unavailable", status);
  }
  const user = ((await me.json().catch(() => ({}))) as { user?: Partial<VerifiedUser> }).user;
  if (!user?.user_id) return json("Authentication required", 401);

  let body: ArrayBuffer | undefined;
  if (request.method !== "GET" && request.method !== "HEAD") {
    if (Number(request.headers.get("content-length") ?? 0) > MAX_AGENTS_BODY_BYTES) return json("Request too large", 413);
    body = await request.arrayBuffer();
    if (body.byteLength > MAX_AGENTS_BODY_BYTES) return json("Request too large", 413);
  }

  const headers = buildAgentsHeaders(
    request.headers,
    { user_id: user.user_id, username: user.username ?? "", role: user.role ?? "" },
    authorization,
    internalToken,
  );
  const upstream = await fetch(`${agentsBaseUrl()}${path}${new URL(request.url).search}`, {
    method: request.method,
    headers,
    body: body?.byteLength ? body : undefined,
    signal: request.signal,
    redirect: "manual",
    cache: "no-store",
  }).catch((error: unknown) => (request.signal.aborted ? null : error));
  if (upstream === null) return new Response(null, { status: 499 });
  if (!(upstream instanceof Response)) {
    console.error("[agents-proxy] upstream unreachable", upstream);
    return json("Agents service unavailable", 502);
  }

  const out = new Headers();
  for (const name of PASSED_BACK) {
    const value = upstream.headers.get(name);
    if (value) out.set(name, value);
  }
  // Artifacts are agent-generated (HTML included): never sniffed or run on our origin.
  out.set("X-Content-Type-Options", "nosniff");
  out.set("Content-Security-Policy", "sandbox");
  if (out.get("content-type")?.startsWith("text/event-stream")) {
    out.set("Cache-Control", "no-cache, no-transform");
    out.set("X-Accel-Buffering", "no");
  }
  const empty = upstream.status === 204 || upstream.status === 304;
  return new Response(empty ? null : upstream.body, { status: upstream.status, headers: out });
}

export const GET = handle;
export const POST = handle;
export const PUT = handle;
export const PATCH = handle;
export const DELETE = handle;
