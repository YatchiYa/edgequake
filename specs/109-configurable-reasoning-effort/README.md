# SPEC-109 — Configurable Reasoning Effort

> **Trigger:** GPT-5-family models (vision + entity extraction) burn completion budget on internal reasoning, starving structured output. Reasoning effort is not configurable today.  
> **Method:** First principles + provider capability matrix + role-scoped config hierarchy + UX surfaces + e2e gates.  
> **Phase:** Spec pack (SSOT for implementation). Code waves tracked in [06-implementation-roadmap.md](06-implementation-roadmap.md).

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  reasoning_effort is a first-class, role-scoped knob.                        │
│  Hierarchy: compiled → env → server → tenant → workspace → request → clamp.  │
│  Capability SSOT = edgequake-llm::reasoning_capabilities (never send 400s).  │
│  Native OpenAI must forward the field (today it drops it).                   │
│  Extract/vlm/summary/keyword default = lowest supported; query = Auto.       │
│  Surfaces: tenant seed · workspace roles · query sheet · PDF upload · /settings │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | Native OpenAI drops `reasoning_effort` | **Confirmed gap** | [04](04-finding-register.md), edgequake-llm `openai.rs` |
| F2 | Extract hardcodes `"none"` (unsafe for gpt-5-mini) | **Confirmed gap** | [04](04-finding-register.md), `completion_options.rs` |
| F3 | No tenant/workspace/query UX | **Confirmed gap** | [02](02-use-cases-and-surfaces.md) |
| F4 | No capability clamp registry | **Confirmed gap** | [03](03-provider-capability-matrix.md) |
| H1 | Configurable effort restores output budget | **Design locked** | [01](01-first-principles.md) LAW-R1..R8 |
| E2E | Contract + provider + Playwright gates | **Specified** | [07](07-e2e-test-matrix.md) |

## Document map

```ascii
 00-why / 00-issue-data
   → 01-first-principles (LAW-R1..R8)
   → 02-use-cases-and-surfaces
   → 03-provider-capability-matrix
   → 04-finding-register
   → 05-cross-ref-matrix
   → 06-implementation-roadmap
   → 07-e2e-test-matrix
   → 08-edge-cases
   → measurements/
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Unified wire field | `CompletionOptions.reasoning_effort: Option<String>` (no parallel knobs in v1) |
| Capability SSOT | `edgequake-llm` module `reasoning_capabilities` |
| Role scope | `extract` · `summary` · `keyword` · `vlm` · `query` (+ chat = query) |
| Config storage | Extend `RoleLlmConfig` + server/tenant seeds — no second metadata tree |
| Structured-output default | Lowest supported effort (`none` clamped to `minimal` when model rejects `none`) |
| Query default | Omit (provider default); UI label **Auto** |
| OpenAI wire (v1) | Chat Completions top-level `reasoning_effort` (not Responses API yet) |
| Proof target | `make spec109-reasoning-effort-proof` |

## Role default intent

| Role | Path | Default |
|------|------|---------|
| **extract** | Entity/relation NER (ingest) | Lowest supported |
| **summary** | Description merge | Lowest supported |
| **keyword** | Dual-level keywords | Lowest supported |
| **vlm** | PDF vision / page understand | Lowest supported |
| **query** / **chat** | RAG answer + chat completions | Auto (omit unless set) |

## Config hierarchy (normative)

```text
Compiled role defaults
  → Env: EDGEQUAKE_REASONING_EFFORT
         EDGEQUAKE_{EXTRACT|QUERY|SUMMARY|VLM|KEYWORD}_REASONING_EFFORT
  → Server DB: server_config.llm_defaults (+ reasoning_by_role)
  → Tenant defaults (seed new workspaces)
  → Workspace: llm_roles.{role}.reasoning_effort
               fallback: workspace.default_reasoning_effort
  → Request: QueryRequest.reasoning_effort
             PdfUploadOptions.vision_reasoning_effort
  → Clamp via ModelReasoningCapabilities
```

## UX surfaces (summary)

| Surface | Route / API | Control |
|---------|-------------|---------|
| Server fleet | `/settings` | Fleet default + per-role advanced |
| Tenant | Wizard / tenant edit | Seed for new workspaces |
| Workspace | `/workspace`, `/w/[slug]/workspace` | Per-role effort select |
| Query | `query-settings-sheet` | Per-query override (Auto = inherit) |
| Ingest | PDF upload advanced | Vision effort override |
| Explainability | `GET /api/v1/config/effective` | Resolved effort + source layer |

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-047 extract options](../047-rag-evaluation/) / pipeline `completion_options` | Mistral Large omit; current `"none"` hardcode |
| [SPEC-103 LLM cache](../103-llm-cache/) | Cache keys must include resolved effort when set |
| [SPEC-096 extraction language](../096-multi-language-extraction/) | Peer role-scoped workspace config pattern |
| [SPEC-101 wizard](../101-wizard-mode-tenant-workspace/) | Tenant/workspace seed UX |
| [SPEC-043 edgequake-llm](../043-update-edgequake-llm/) | Catalog / provider sync |
| [SPEC-108 density](../108-extraction-compared-light-rag/) | Adjacent ingest quality; not a fork |

## DRY rule

**Capability truth** (which efforts a model accepts, how to clamp, how to wire) lives in **`edgequake-llm`**. EdgeQuake **resolves** desired effort and **surfaces** it in API/UI; it must not fork per-provider effort lists. If packs disagree on API wire shape, **provider docs + edgequake-llm win**.

## Out of scope (v1)

- Responses API / `reasoning.mode=pro`
- Exposing thinking traces in WebUI
- Per-chunk dynamic effort
- Acc protocol rewrite (document Acc pins only)

## Start here

1. [00-why.md](00-why.md)  
2. [00-issue-data.md](00-issue-data.md)  
3. [01-first-principles.md](01-first-principles.md)  
4. [02-use-cases-and-surfaces.md](02-use-cases-and-surfaces.md)  
5. [03-provider-capability-matrix.md](03-provider-capability-matrix.md)  
6. [06-implementation-roadmap.md](06-implementation-roadmap.md)  
7. [07-e2e-test-matrix.md](07-e2e-test-matrix.md)
