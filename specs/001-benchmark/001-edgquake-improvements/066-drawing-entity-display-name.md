# 066 — Drawing Entity Display Name

**Status:** Implemented  
**Date:** 2026-07-21  
**Law:** Identity ≠ presentation (MegaRAG / MMKG 2026)

## Problem

KG UI listed Drawing nodes as opaque `im-{uuid}-page-…` ids. VLM `[Figure Name]` existed in mm chunks and association edges but never became Graph `label`.

## Law

| Concern | SSOT |
|---------|------|
| Identity | `im-…` / `IM-…` item id (stable, collision-resistant) |
| Presentation | `properties.display_name` + Graph API `label` |
| Fallback | VLM name → heading/caption → `Fig n · p.m` → optional doc short title |

## Changes

1. [`edgequake-pipeline/src/multimodal/display.rs`](../../../../edgequake/crates/edgequake-pipeline/src/multimodal/display.rs) — `resolve_mm_entity_display`, locus parser, placeholder filter, lazy read-path.
2. Inject + merger persist `display_name`, `page_num`, `figure_index`, `asset_id`, `mm_subtype`.
3. Graph API `graph_node_label` in all node response builders.
4. WebUI: DRAWING/TABLE/EQUATION colors; preserve human labels; detail Identity + thumbnail.
5. Query `build_entity_from_node` prefers `display_name` for mm types.

## Verify

```bash
cargo test -p edgequake-pipeline --lib multimodal
cargo test -p edgequake-api graph_label --lib
cargo test -p edgequake-api --test contract_066_drawing_display_name
cd edgequake_webui && pnpm exec vitest run src/lib/graph/label-utils.test.ts
```
