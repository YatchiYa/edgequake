/**
 * SPEC-083 X-26 — wire generated OpenAPI types into the webui type graph.
 *
 * Source of truth: `openapi/schema.d.ts` (produced by `pnpm codegen:api`).
 * Prefer these types for new API DTOs; migrate hand-written `@/types` gradually.
 *
 * Use a relative import (not `@/openapi/schema`): Next.js path `@/*` → `src/*`
 * would otherwise shadow `@/openapi/*` in Docker production builds.
 */
export type { paths, components, operations } from "../../openapi/schema";
