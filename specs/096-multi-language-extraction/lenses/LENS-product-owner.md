# LENS — Product Owner (SPEC-096)

> **Laws**: L1–L5 · **Findings**: F-352-01…15 · **Issue**: [GH-352](../issues/GH-352-extraction-language.md)

## Problem worth solving

Non-English customers cannot ship a coherent knowledge graph without forking Rust prompts. That is a **product gap**, not a model limitation. LightRAG already treats language as a first-class knob (`SUMMARY_LANGUAGE`); EdgeQuake advertised the same via dead `{language}` code.

## Acceptance (“done”)

1. **AC-PO-01** — Operator sets workspace extraction language to `Chinese` (or any allowlisted value) via UI **or** API without rebuilding the binary.  
2. **AC-PO-02** — Subsequent ingestions instruct the LLM (JSON path) to emit names/descriptions in that language.  
3. **AC-PO-03** — Server default can be set with `EDGEQUAKE_EXTRACTION_LANGUAGE` for fleets that skip per-workspace config.  
4. **AC-PO-04** — Changing language does **not** silently rewrite existing entities; UX tells the user to reprocess.  
5. **AC-PO-05** — Unsupported languages fail closed (400) with a clear allowlist message.  
6. **AC-PO-06** — Docs + OpenAPI describe the field; GH-352 can be closed with evidence.

## Non-goals

- Auto-detecting document language.
- One-click translation of an existing graph.
- Perfect LLM obedience (model quality is ops/model selection).
- Tenant-level language inheritance in v1.
- Changing UI chrome locale (separate i18n concern).

## Success metrics

| Metric | Signal |
|--------|--------|
| Configurability | Language settable in <2 minutes from workspace page |
| Correctness | Mock/e2e proves prompt contains target language |
| Safety | No migration; existing English workspaces unchanged when field omitted |
| Parity | Behavior analogous to LightRAG `SUMMARY_LANGUAGE` |

## Priority

**P0** for international / CJK adoption. Unblocks Chinese/Japanese/Korean deployments cited in GH-352.

## Release narrative

> “Workspaces can now pin the language used for entity and relationship extraction — matching LightRAG’s SUMMARY_LANGUAGE — from the workspace settings UI or API.”

## Dependencies / risks

| Risk | Mitigation |
|------|------------|
| LLM ignores instruction | Document model recommendations (Qwen for CJK); keep entity_types localization as complementary hint |
| FE/BE allowlist drift | Shared constant comment + e2e; prefer single OpenAPI enum if generated |
| Operators expect instant graph rewrite | LAW-L5 + toast (copy from entity types / LLM change) |

## Done means

Waves W1–W4 green per [roadmap](../03-implementation-roadmap.md); AC-PO-01…06 satisfied; issue #352 closable.
