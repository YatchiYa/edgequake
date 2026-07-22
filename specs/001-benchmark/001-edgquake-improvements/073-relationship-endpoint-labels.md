# 073 — Relationship Endpoint Labels + Full-App Opaque Display Sweep

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** Identity ≠ presentation on relationship endpoints; opaque machine IDs are never human labels

## Problem

Query page **Connections** showed truncated UUIDs (`84b69e27-e38… → has theme → …`) while **Key Topics** showed human names. [072](./072-lineage-label-ssot-opaque.md) soft-labeled entities and document-scoped KG nodes; relationship `source`/`target` remained raw graph node ids.

Also remaining bypasses: traversal `label: n.id`, neighborhood expand without `label`, chunk/provenance lineage raw names, LLM `format_relationship_line` embedding UUIDs.

## Law

| Concern | SSOT |
|---------|------|
| Endpoint identity | `RetrievedRelationship.source` / `.target` (graph node id) |
| Endpoint presentation | `source_label` / `target_label` via `resolve_entity_display_label` |
| Graph node responses | `graph_node_label` (never `label: *.id`) |
| WebUI display | Prefer `*_label`; clicks keep identity |

## Changes

1. Query: `source_label`/`target_label` + batch resolve after edges collected (local/global).
2. API: ContextRelationship passthrough; traversal/neighborhood/chunk_detail/provenance soft-label.
3. WebUI: Connections show labels; `displayEntityLabel` defense-in-depth.
4. Contracts forbid `label: n.id` and require endpoint label wiring.

## Ops

Soft-label fixes display without re-ingest. Clean KG still needs re-ingest of UUID-heavy docs (067/072).

## Verify

```bash
cargo test -p edgequake-query --lib helpers
cargo test -p edgequake-query --lib context_format
cargo test -p edgequake-api --test contract_073_relationship_endpoint_labels
cd edgequake_webui && pnpm exec vitest run src/lib/graph/label-utils.test.ts src/lib/utils/
```
