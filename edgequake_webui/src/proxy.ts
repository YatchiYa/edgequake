/**
 * SPEC-144 — single Next.js network boundary (`proxy` convention).
 *
 * Composes:
 * - Swagger trailing-slash redirect (blank UI without `/swagger-ui/`)
 * - SPEC-083 X-27 coarse session guard (cookie mirror; not crypto auth)
 *
 * Next.js 16 supports one `proxy.ts` per project (root or `src/`).
 * Helpers live in `./lib/server/proxy-guards` for DRY/SOLID + unit tests.
 */
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";
import { applyAuthGuard, applySwaggerSlashRedirect } from "./lib/server/proxy-guards";

export function proxy(request: NextRequest) {
  const swagger = applySwaggerSlashRedirect(request);
  if (swagger) return swagger;

  const auth = applyAuthGuard(request);
  if (auth) return auth;

  return NextResponse.next();
}

export const config = {
  matcher: [
    /*
     * App routes + swagger slash; skip static assets (same as former middleware).
     */
    "/((?!_next/static|_next/image|.*\\.(?:png|jpg|jpeg|gif|svg|ico|webp|css|js|map)$).*)",
  ],
};
