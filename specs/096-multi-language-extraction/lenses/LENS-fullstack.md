# LENS — Full Stack (SPEC-096)

> **Laws**: L1–L5 · **Findings**: F-352-01…14 · **Flow**: end-to-end wire

## Target path

```
WebUI WorkspaceExtractionLanguageCard
    → updateWorkspace({ extraction_language: "Chinese" })
        → PUT/PATCH /api/v1/tenants/{tid}/workspaces/{wid}
            → validate allowlist (400 if bad)          ← LAW-L1
            → apply_extraction_language_metadata       ← JSONB
            → response includes extraction_language

Ingest document in workspace
    → load workspace metadata
    → resolve_extraction_language(meta, env)           ← LAW-L3
        → IngestionPipelineOptions.extraction_language
            → build_ingestion_pipeline(...)
                → LLMExtractor.with_language(...)
                → GleaningExtractor (same language)
                    → json_extraction_prompt(..., language)  ← LAW-L2/L4
                    → LLM completion
                    → parse JSON (English keys)
                    → graph write
```

## Breakpoints (before → after)

| Layer | Before | After |
|-------|--------|-------|
| Prompt | English-only JSON | Language instruction SSOT |
| Extractor | No language field | `with_language` |
| Factory | Hardcoded schema only | Schema + language |
| Resolve | N/A | workspace → env → English |
| API | No field | Create/Update/Response |
| DB | No key | `metadata.extraction_language` |
| UI | Entity types only | Language card |

## Ownership (SRP)

| Module | Owns |
|--------|------|
| `prompts/json_prompts.rs` | Wording of language instruction + prompt assembly |
| `resolve_extraction_language` | Precedence + canonicalize |
| `apply_extraction_language_metadata` | JSONB mutate/clear |
| API handlers | HTTP validate + map DTO |
| Orchestrator ingestion | Pass resolved language into pipeline options |
| WebUI card | Presentation + edit state |

## Contract rules

1. **Orthogonal to entity schema** — do not add language into `EntityExtractionSchema` (ISP).  
2. **Snapshot at pipeline build** — in-flight jobs do not flip mid-extract when UI saves (EC-14).  
3. **Clear semantics** — `""` / `"none"` removes override (vision model precedent).  
4. **Env invalid** — warn + English; **API invalid** — 400.

## Observability

Log at pipeline build (info):

```
extraction_language = Chinese
extraction_language_source = workspace | env | default
```

Aids support without new metrics backend.

## Test spine

Unit prompts → resolve unit → API contract → optional mock ingest e2e → Playwright UI. See [04-e2e-test-matrix.md](../04-e2e-test-matrix.md).

## Laws

All L1–L5 apply on this path.
