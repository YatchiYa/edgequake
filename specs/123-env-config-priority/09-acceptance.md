# 09 — Acceptance

## Status (honest)

| Slice | Status |
|-------|--------|
| PDF parser law (silent EdgeParse fix) | **Done** (code + e2e; partner repro advised) |
| Model SSOT (LLM / embedding / vision LLM) | **Done** — GET provenance, tenant update, sync+async resolve, pipeline/VLM, FE Resolves-to, e2e green |
| Partner / operator acceptance | **Open** |

Scorecard: [11-honest-assessment.md](11-honest-assessment.md).

## Must be true — PDF

- [x] LAW-123-1…4,6 documented and implemented for PDF
- [x] Server Default (Vision) never silently becomes EdgeParse
- [x] Auto is explicit and labeled honestly
- [x] Tenant layer in PDF cascade
- [x] V1–V8 addressed or test-gated
- [x] API e2e matrix (`e2e_spec123_parser_priority`)
- [x] WebUI unit + thin Playwright option check
- [ ] Operator repro in [10-reproduction.md](10-reproduction.md) on current build (partner)

## Must be true — Models

- [x] `model_resolution` + LAW-123-8
- [x] Vision not mutated into upload via `apply_workspace`
- [x] Vision inherit-paint stopped; metadata gate for LLM/embedding
- [x] Workspace GET exposes `resolved_*` + `*_resolution_source`
- [x] Tenant update applies vision/LLM/embedding defaults
- [x] Run paths load tenant into resolve (CRUD, async query, **sync→async query/chat/MCP**, pipeline embed, VLM try_*)
- [x] HTTP/unit e2e: model matrix + GET provenance (vision **and** LLM/embedding tenant) + upload vision options
- [x] FE vision settings + workspace card use effective resolve + “Resolves to”
- [x] LLM/embedding settings cards show Resolves-to via `WorkspaceModelConfigGrid` + `effective*FromWorkspace`
- [x] No invented “vision embedding”
- [x] Acc extract/keyword env-first pins unchanged

## Out of scope sign-off

SPEC-015V / SPEC-122 unchanged. Acc env-first extract/keyword unchanged unless product unlocks LAW-123-2 for those roles.
