# 067 — Opaque Entity Name Reject

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** Semantic name required; opaque machine IDs are not entity names (identity ≠ presentation for legacy soft-label)

## Problem

KG UI showed Organization/Concept nodes labeled with raw UUID/GUIDs (e.g. `84b69e27-E38b-…`). Root cause was **extraction + `EntityId` acceptance**, not display: the LLM extracted opaque resource IDs from API/agentic documents, and `normalize_entity_name` only rejected LightRAG short numerics (056). Display faithfully rendered the stored identity.

Distinct from [066](./066-drawing-entity-display-name.md) (`im-…` Drawing opacity was a `label: node.id` bug while VLM names existed).

## Law

| Concern | SSOT |
|---------|------|
| Write reject | `is_opaque_identifier` + `normalize_entity_name` → empty `EntityId` (skip write) |
| Prompt | Forbid UUID/GUID/hash/ARN as `entity_name`; prefer human referent |
| Legacy read | `graph_node_label` soft-label: description snippet or `Opaque ID · {type}` |
| Multimodal keep | `im-…` / `IM-…` identities remain valid (066) |

## Changes

1. [`edgequake-storage/src/entity_id.rs`](../../../../edgequake/crates/edgequake-storage/src/entity_id.rs) — `is_opaque_identifier` (UUID, ULID, ObjectId, hex digests, ARN); wired into normalizer.
2. SOTA + JSON extraction prompts — opaque ID hygiene (intentional LightRAG divergence).
3. Parser/merger — skip empty ids; log `metric = "opaque_entity_name_rejected"`.
4. [`graph_label.rs`](../../../../edgequake/crates/edgequake-api/src/handlers/graph/graph_label.rs) — legacy soft-label for opaque bare ids.
5. WebUI `isOpaqueIdentifier` + Identity row for opaque nodes.

## Ops / cleanup

- **Fresh ingest** after upgrade: opaque names are not written.
- **Legacy nodes**: readable via soft-label without re-ingest; for a clean graph, re-ingest UUID-heavy documents or prune low-degree nodes whose bare id matches `is_opaque_identifier`.
- **Out of scope v1:** full entity-resolution / SAME_AS dream pipeline.

## Verify

```bash
cargo test -p edgequake-storage --lib entity_id
cargo test -p edgequake-pipeline --lib opaque
cargo test -p edgequake-api --lib graph_label
cargo test -p edgequake-api --test contract_067_opaque_entity_names
cargo test -p edgequake-pipeline --test e2e_067_opaque_entity_names
cd edgequake_webui && pnpm exec vitest run src/lib/graph/label-utils.test.ts
cargo fmt --check
cargo clippy -p edgequake-storage -p edgequake-pipeline -p edgequake-api --all-targets -- -D warnings
```
