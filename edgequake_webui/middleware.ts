/**
 * SPEC-083 X-27 — coarse server-side session guard.
 *
 * Tokens live in localStorage (client) and are mirrored to the
 * `edgequake_access_token` cookie on login so Edge middleware can redirect
 * unauthenticated deep links away from protected routes.
 *
 * This is not a cryptographic authorization boundary — API handlers still
 * enforce JWT/API-key auth. Middleware only prevents obvious unauthenticated
 * HTML navigation when NEXT_PUBLIC_AUTH_ENABLED / DISABLE_DEMO_LOGIN is on.
 */
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

const AUTH_COOKIE = "edgequake_access_token";

const PUBLIC_PREFIXES = [
  "/login",
  "/api",
  "/_next",
  "/favicon",
  "/e2e-fixtures",
];

function authRequired(): boolean {
  const authEnabled = process.env.NEXT_PUBLIC_AUTH_ENABLED === "true";
  const disableDemo = process.env.NEXT_PUBLIC_DISABLE_DEMO_LOGIN === "true";
  return authEnabled || disableDemo;
}

function isPublicPath(pathname: string): boolean {
  if (pathname === "/") return false;
  return PUBLIC_PREFIXES.some(
    (p) => pathname === p || pathname.startsWith(`${p}/`),
  );
}

export function middleware(request: NextRequest) {
  if (!authRequired()) {
    return NextResponse.next();
  }

  const { pathname } = request.nextUrl;
  if (isPublicPath(pathname)) {
    return NextResponse.next();
  }

  const token = request.cookies.get(AUTH_COOKIE)?.value;
  if (!token) {
    const login = new URL("/login", request.url);
    login.searchParams.set("redirect", pathname);
    return NextResponse.redirect(login);
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    /*
     * Protect app routes; skip static assets.
     */
    "/((?!_next/static|_next/image|.*\\.(?:png|jpg|jpeg|gif|svg|ico|webp|css|js|map)$).*)",
  ],
};
