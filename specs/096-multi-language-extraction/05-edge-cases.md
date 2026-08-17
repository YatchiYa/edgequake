# SPEC-096 — Edge Cases

> **Cross-refs**: [Laws](00-first-principles.md) · [E2E matrix](04-e2e-test-matrix.md) · [PO lens](lenses/LENS-product-owner.md)

| ID | Case | Expected behavior | Mitigation / Law | Test |
|----|------|-------------------|------------------|------|
| EC-01 | Omit `extraction_language` on create | Resolve env → else `English` | L3 | `spec096_resolve_language_precedence` |
| EC-02 | Update with `null` / omit field | Leave existing metadata unchanged | L5 / PATCH semantics | API contract |
| EC-03 | Update with `""` or `"none"` | Remove metadata key; fall back to env/default | Mirror vision clear | `spec096_api_create_update_get_language` |
| EC-04 | Case variants `"chinese"`, `"CHINESE"` | Canonicalize to `"Chinese"` if allowlisted | L3 canonicalize | `spec096_canonicalize_language` |
| EC-05 | Unsupported value `"Klingon"` | HTTP 400; no metadata write | L1 allowlist | `spec096_api_rejects_unsupported_language` |
| EC-06 | Env set to unsupported value | Warn; treat as unset → `English` (no crash) | Ops safety L3 | `spec096_resolve_language_precedence` |
| EC-07 | Workspace `Chinese` + env `French` | Workspace wins | L3 | precedence unit |
| EC-08 | Mixed-language source document | Still emit target language; proper nouns may stay original | L1, prompt rule | doc + optional LLM e2e |
| EC-09 | Localized `entity_types` + matching language | Types + descriptions both target language; presets auto-remap (LAW-L6) | Complements F-352-15 | `spec096_ui_entity_types_follow_language` |
| EC-10 | Localized `entity_types` + language `English` | Types may be non-English labels; descriptions English-instructed | Document quirk; operator choice | doc |
| EC-11 | Strict entity types English + language Chinese | Allowed: operator may keep English type tokens while NL values are Chinese; presets default to remapping when selection matches catalog | L4 + L6 | prompt unit + UI |
| EC-23 | Language change with **custom/mixed** type list | Chips unchanged; optional muted hint that custom types are not auto-translated | L6 | catalog unit |
| EC-24 | Language clear / Server default after French General preset | Remap back to English General tokens when still preset-backed | L6 | S07 Playwright |
| EC-25 | Language change French→Chinese on General preset | Remap via catalog; no intermediate English flash required | L6 | catalog unit |
| EC-12 | Gleaning enabled | Gleaning pass uses same language | L2 | `spec096_gleaning_prompt_includes_language` |
| EC-13 | Language change after documents ingested | Existing nodes unchanged until reprocess/rebuild | L5 | UI hint + no migration |
| EC-14 | Concurrent ingest during language update | In-flight jobs keep language resolved at job start | Snapshot at pipeline build | doc |
| EC-15 | SOTA path re-enabled later | `with_language` + omit English few-shots | F-352-06/07 | SOTA unit |
| EC-16 | Whitespace-only language `"   "` | Treat as empty → clear or reject (prefer reject on API, fallthrough on resolve) | L3 | canonicalize unit |
| EC-17 | Max-length / injection string in language | Allowlist only — reject | Security | API 400 |
| EC-18 | Query UI / chat after Chinese extract | Query still works; answers may follow query language (out of scope) | Non-goal | smoke |
| EC-19 | Multilingual embedding mismatch | Retrieval quality may drop; docs advise multilingual embed model | Ops note | docs only |
| EC-20 | Tenant default vs workspace | v1: workspace + env only (no tenant-level language) | Scope | doc |
| EC-21 | OpenAPI client codegen | Field optional; older clients omit → English path | Compat | OpenAPI contract |
| EC-22 | `SUPPORTED_LANGUAGES` extended later | FE constant list must stay in sync (shared comment or codegen note) | OCP | checklist W3 |

---

## Explicit non-cases (do not implement as bugs)

| Temptation | Why out of scope |
|------------|------------------|
| Auto-detect language from PDF text | Fragments graph vocabulary (LAW-L1) |
| Translate all existing AGE nodes on save | Violates LAW-L5; expensive; lossy |
| Localize JSON keys | Breaks parsers (LAW-L4) |
| Per-chunk language | Same as auto-detect risk |

---

## Failure UX

| Failure | User-visible |
|---------|----------------|
| Unsupported language on save | Inline form error + toast; API 400 body message lists allowed values |
| Env misconfigured | Server starts; logs warn; workspaces without override get English |
| LLM ignores language instruction | Soft quality issue — document that model choice matters (Qwen for CJK, etc.) |
