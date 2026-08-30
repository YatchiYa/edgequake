/**
 * SPEC-144 / SPEC-083 X-27 — pure proxy helpers (no Next file-convention export).
 *
 * Kept separate from `src/proxy.ts` so unit tests can exercise auth + swagger
 * redirects without loading the Next proxy entrypoint.
 */
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export const AUTH_COOKIE = "edgequake_access_token";

export const PUBLIC_PREFIXES = [
  "/login",
  "/api",
  "/_next",
  "/favicon",
  "/e2e-fixtures",
] as const;

type EnvLike = Record<string, string | undefined>;

export function authRequired(env: EnvLike = process.env): boolean {
  const authEnabled = env.NEXT_PUBLIC_AUTH_ENABLED === "true";
  const disableDemo = env.NEXT_PUBLIC_DISABLE_DEMO_LOGIN === "true";
  return authEnabled || disableDemo;
}

export function isPublicPath(pathname: string): boolean {
  if (pathname === "/") return false;
  return PUBLIC_PREFIXES.some(
    (p) => pathname === p || pathname.startsWith(`${p}/`),
  );
}

/** Returns a redirect Response, or null to continue. */
export function applySwaggerSlashRedirect(
  request: NextRequest,
): NextResponse | null {
  if (request.nextUrl.pathname === "/swagger-ui") {
    // Plain URL — NextURL strips trailing slashes when skipTrailingSlashRedirect is set.
    return NextResponse.redirect(new URL("/swagger-ui/", request.url), 307);
  }
  return null;
}

/**
 * Coarse HTML navigation guard. Not a cryptographic authorization boundary —
 * API handlers still enforce JWT/API-key auth.
 *
 * Returns a redirect Response, or null to continue.
 */
export function applyAuthGuard(
  request: NextRequest,
  env: EnvLike = process.env,
): NextResponse | null {
  if (!authRequired(env)) {
    return null;
  }

  const { pathname } = request.nextUrl;
  if (isPublicPath(pathname)) {
    return null;
  }

  const token = request.cookies.get(AUTH_COOKIE)?.value;
  if (!token) {
    const login = new URL("/login", request.url);
    login.searchParams.set("redirect", pathname);
    return NextResponse.redirect(login);
  }

  return null;
}
