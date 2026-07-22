# 072 — Lineage Label SSOT + Opaque Entity Hygiene

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** Identity ≠ presentation on every label surface; opaque machine IDs are never human labels

## Problem

Document-scoped Knowledge Graph (`/graph?document=…`) showed Concept/Org/Person nodes labeled with raw UUID/GUIDs (e.g. `84b69e27-E38b-…`).

[067](./067-opaque-entity-name-reject.md) soft-labeled the **workspace graph stream** via `graph_node_label`, but the document filter loads `GET /lineage/documents/:id`, which set `EntitySummaryResponse.name = node.id` and never called the label SSOT. The WebUI then did `label = formatEntityLabel(id)`, painting truncated UUIDs on Sigma.

Distinct from [066](./066-drawing-entity-display-name.md) (Drawing `im-…` vs VLM `display_name`).

## Law

| Concern | SSOT |
|---------|------|
| Write reject | `is_opaque_identifier` + `normalize_entity_name` → empty `EntityId` (067; hardened for prefixed UUID) |
| Graph / lineage / CRUD / query labels | `graph_node_label` (or equivalent soft-label for query crate) |
| Lineage DTO | `id` = graph node id; `label` = soft-label; `name` = bare name or soft-label (never raw UUID) |
| Multimodal keep | `im-…` / `IM-…` identities remain valid (066) |

## LightRAG

Adopt: provenance never becomes a name; drop incomplete extractions.  
Intentional divergence: keep opaque write-reject (LightRAG allows UUID entity names).

## Changes

1. `EntitySummaryResponse` — `id`, `label`, optional `description`; wire `entity_summary_from_node` through `graph_node_label`.
2. WebUI `documentLineageToKnowledgeGraph` — prefer API `label`; keep edge endpoints as node ids.
3. Soft-label remaining bypasses: `search_labels`, entity CRUD presentation, query `build_entity_from_node`; `create_entity` 400 on empty after normalize.
4. Harden `is_opaque_identifier` for `PREFIX_UUID` / `uuid:…` shapes; mirror WebUI.

## Ops

- Soft-label fixes display without re-ingest.
- Clean KG still needs re-ingest of UUID-heavy documents (067 note).
- Out of scope: full Wikidata-style opaque-id identity / SAME_AS ER.

## Verify

```bash
cargo test -p edgequake-storage --lib entity_id
cargo test -p edgequake-api --lib document_graph_lineage
cargo test -p edgequake-api --lib graph_label
cargo test -p edgequake-api --test contract_072_lineage_label_ssot
cargo test -p edgequake-query --lib helpers
cargo test -p edgequake-pipeline --test e2e_067_opaque_entity_names
cd edgequake_webui && pnpm exec vitest run src/lib/graph/label-utils.test.ts src/lib/graph/__tests__/document-lineage-to-graph.test.ts
cargo fmt --check
cargo clippy -p edgequake-storage -p edgequake-pipeline -p edgequake-api -p edgequake-query --all-targets -- -D warnings
```
