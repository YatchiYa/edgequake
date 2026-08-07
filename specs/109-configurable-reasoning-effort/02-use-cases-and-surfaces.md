# SPEC-109 — Use Cases and Surfaces

> **Laws**: LAW-R5 (role defaults) · LAW-R6 (hierarchy) · LAW-R8 (surface parity)  
> **UX rule**: Never offer an effort value the capability API marks unsupported for the selected model.

## 1. Use-case inventory

| UC-ID | Role | When it runs | Why effort matters | Default | Overridable at |
|-------|------|--------------|--------------------|---------|----------------|
| **UC-EXTRACT** | `extract` | Ingest entity/relation NER (LLM / SOTA / gleaning) | Structured JSON needs output tokens | Lowest supported | Env · server · tenant · workspace role · (future: reprocess options) |
| **UC-SUMMARY** | `summary` | Entity/relation description merge | Same as extract — schema-ish text | Lowest supported | Env · server · tenant · workspace role |
| **UC-KEYWORD** | `keyword` | Dual-level hl/ll keyword extract at query | High volume, short JSON | Lowest supported | Env · server · tenant · workspace role · (optional query inherit) |
| **UC-VLM** | `vlm` | PDF page / table / figure vision | High page volume; captions starve under CoT | Lowest supported | Env · server · tenant · workspace vision · PDF upload `vision_reasoning_effort` |
| **UC-QUERY** | `query` | RAG answer (`/query`, `/query/stream`) | Quality vs latency/cost | **Auto** (omit) | Env · server · tenant · workspace role · **`QueryRequest.reasoning_effort`** |
| **UC-CHAT** | `query` (same policy) | `/chat/completions` OpenAI-compat | Same as query | Auto | Same as query (+ chat body field if exposed) |

### Non-use-cases (v1)

| Item | Reason |
|------|--------|
| Embeddings | No reasoning_effort on embed APIs |
| Rerank | Separate model path; out of scope |
| Mock provider | Accept and ignore; tests assert options still set |

## 2. Configuration surfaces (product)

```text
┌──────────────┐   seed    ┌──────────────┐   llm_roles.*.reasoning_effort
│   Tenant     │ ────────► │  Workspace   │ ─────────────────────────────┐
│ defaults     │           │  + default_  │                             │
└──────────────┘           │  reasoning_  │                             ▼
                           │  effort      │                    ┌─────────────────┐
┌──────────────┐           └──────────────┘                    │ Role completion │
│ Server       │  fleet defaults / reasoning_by_role ─────────►│ + clamp         │
│ /settings    │                                               └────────┬────────┘
└──────────────┘                                                        │
┌──────────────┐   EDGEQUAKE_*_REASONING_EFFORT                         │
│ Env / Acc    │ ───────────────────────────────────────────────────────┤
└──────────────┘                                                        │
┌──────────────┐   QueryRequest / PdfUploadOptions                      │
│ Request      │ ───────────────────────────────────────────────────────┘
└──────────────┘
```

### 2.1 Tenant

| Field (conceptual) | Behavior |
|--------------------|----------|
| `default_reasoning_effort` | Optional single seed applied to new workspaces as `default_reasoning_effort` |
| `default_llm_roles.*.reasoning_effort` | Optional per-role seed into workspace `metadata.llm_roles` |

**UI**: Tenant create / edit / onboarding wizard ([SPEC-101](../101-wizard-mode-tenant-workspace/)) — advanced section under LLM defaults, same visual language as vision LLM seed.

### 2.2 Server (fleet)

| Field | API |
|-------|-----|
| `reasoning_effort` | `GET/PATCH /api/v1/settings/llm-defaults` |
| `reasoning_by_role: { extract?, query?, … }` | Same payload |

**UI**: `/settings` LLM defaults card — primary “Default reasoning effort” + expandable “Per role”.

### 2.3 Workspace

| Field | Storage |
|-------|---------|
| `llm_roles.{extract,query,summary,vlm,keyword}.reasoning_effort` | Existing metadata object; extend `RoleLlmConfig` |
| `default_reasoning_effort` | Optional workspace-level fallback when role omits |

**UI**: `/workspace` and `/w/[slug]/workspace` — beside each role’s provider/model selector, an effort `<Select>`:

- Options = `Auto (inherit)` + supported efforts from models catalog for **that** role’s model
- Disabled / hidden when model has empty supported list (non-reasoning)
- Helper text: “Lower effort preserves tokens for structured output”

### 2.4 Query (request)

| Field | API |
|-------|-----|
| `reasoning_effort: Option<String>` | `POST /api/v1/query`, `/query/stream` |
| Echo on stats | `QueryStats.reasoning_effort` + stream meta (optional but preferred) |

**UI**: `query-settings-sheet.tsx` — control next to provider/model:

- Label: **Reasoning effort**
- Values: `Auto` | `none` | `minimal` | `low` | `medium` | `high` | `xhigh` | `max` (filtered by selected model)
- `Auto` = do not send field (inherit workspace query role)

### 2.5 Ingest / vision (request)

| Field | API |
|-------|-----|
| `vision_reasoning_effort` | PDF upload options / multipart advanced fields |

**UI**: Document upload advanced panel — only when vision backend uses an LLM VLM path.

### 2.6 Explainability

`GET /api/v1/config/effective` must report per role:

```json
{
  "roles": {
    "extract": {
      "provider": "openai",
      "model": "gpt-5-mini",
      "reasoning_effort": {
        "desired": "none",
        "effective": "minimal",
        "source": "compiled_default",
        "clamped": true
      }
    }
  }
}
```

**UI**: Settings effective-config panel shows desired → effective + source badge.

## 3. Env var catalog (normative)

| Variable | Scope |
|----------|-------|
| `EDGEQUAKE_REASONING_EFFORT` | Fleet fallback for all roles when role-specific unset |
| `EDGEQUAKE_EXTRACT_REASONING_EFFORT` | Extract |
| `EDGEQUAKE_QUERY_REASONING_EFFORT` | Query / chat |
| `EDGEQUAKE_SUMMARY_REASONING_EFFORT` | Summary |
| `EDGEQUAKE_VLM_REASONING_EFFORT` | Vision |
| `EDGEQUAKE_KEYWORD_REASONING_EFFORT` | Keyword |

Values: effort strings or empty. Invalid strings: warn + treat as unset (do not crash process start). Acc / cold-bench: pin structured roles to lowest (`none` or `minimal` per model).

## 4. Models catalog / API for UI

Extend models list / health payload so each LLM model card can include:

```json
{
  "id": "gpt-5-mini",
  "reasoning_effort": {
    "supported": ["minimal", "low", "medium", "high"],
    "default": "medium",
    "lowest_structured": "minimal"
  }
}
```

Source: `edgequake-llm` registry (LAW-R4), exposed via EdgeQuake `/api/v1/models` (or sibling). UI **must not** hardcode effort enums per slug.

## 5. Wireframe checklist (implementation)

| # | Screen | Acceptance |
|---|--------|------------|
| W1 | Workspace LLM roles | Save/reload extract + query effort independently |
| W2 | Server settings | Patch fleet default; visible in effective config |
| W3 | Query sheet | Override appears in network request body |
| W4 | PDF upload advanced | Vision effort only when VLM path active |
| W5 | Tenant wizard | New workspace inherits tenant seed roles |
| W6 | Unsupported model | Control shows “Not applicable” / hidden |

## 6. Mapping to existing code (targets)

| Concern | File / area |
|---------|-------------|
| Roles | `edgequake-core/src/llm_roles.rs` (`RoleLlmConfig`) |
| Server defaults | `edgequake-core/src/server_config_overrides.rs` |
| Workspace / tenant types | `types/multitenancy/{workspace,tenant,requests}.rs` |
| Query body | `edgequake-api/src/handlers/query_types.rs` |
| PDF options | `edgequake-api/src/handlers/pdf_upload/types.rs` |
| Effective config | `edgequake-api/src/config_resolution.rs` |
| Extract options | `edgequake-pipeline/.../completion_options.rs` |
| WebUI workspace | `edgequake_webui` workspace settings + vision card |
| WebUI query | `query-settings-sheet.tsx`, `types/settings.ts` |
| WebUI settings | `settings/page.tsx`, LLM defaults cards |
