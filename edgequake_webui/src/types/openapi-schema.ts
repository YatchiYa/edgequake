/**
 * SPEC-083 X-26 — wire generated OpenAPI types into the webui type graph.
 *
 * Source of truth: `openapi/schema.d.ts` (produced by `pnpm codegen:api`).
 * Prefer these types for new API DTOs; migrate hand-written `@/types` gradually.
 */
export type { paths, components, operations } from "@/openapi/schema";
