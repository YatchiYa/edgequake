# 07 — Residual Risks (post quality assessment)

> Assessed 2026-08-04 against SPEC-107 delivery + live HEAD. Partner E1–E4 classes are closed in code ≥0.24.0; this page tracks **what remains**.

## Quality verdict

| Area | Grade | Notes |
|------|-------|-------|
| Partner pack (00–06) | **A−** | DRY vs 104; broken reply table fixed; residual ops present |
| E1/E2/E4 code | **A** | Naming SSOT + tenant upsert; source contracts green |
| E3 monitor | **A** | Dual-read + LogOnly + `indexed\|completed` (SPEC-107 harden) |
| Residual completeness | **B+ → A−** after this page + INV-C fail-visible |

Overall SPEC-107 delivery: **A−** (was B+ before harden).

## Cleared (partner email classes)

| Risk | Status |
|------|--------|
| `workspaces.id` 42703 | Cleared — `workspace_id::text` |
| Hardcoded `edgequake."Node"` in inspector Default | Cleared — `for_namespace` |
| Bare tenant INSERT race → raw 23505 | Cleared — `ON CONFLICT (slug)` |
| INV-03 silent `_ => None` repair | Cleared — LogOnly |
| INV-03 `indexed`-only miss of `completed` orphans | Cleared — `IN ('indexed','completed')` |
| INV-C skip-on-error silent green | Cleared — `SchemaDriftIssue` Warning |

## Still open (ranked)

### R1 — High — Multi-workspace inspect silent green (SPEC-104 EC-05)

Boot + hourly + admin inspect use `InspectorConfig::for_namespace("default")` only.

- **Files:** `state/postgres.rs`, `handlers/admin.rs`
- **Effect:** INV-C / INV-01 KV / null-rate for non-default workspaces never evaluated; fleet can look healthy.
- **Mitigation now:** Ops inspect per workspace namespace manually; session agenda item 5.
- **Code follow-up:** loop namespaces or admin `?namespace=` (out of SPEC-107 thin scope).

### R2 — Medium — Node-count `57014` under load (SPEC-104 #5 / EC-13)

**Partially closed (SPEC-107 R2 harden):** INV-C now chunks prefixes with public `SOURCE_PREFIX_BATCH_LIMIT` (32) — same LAW-H1 budget as list analytics. List soft-fail warns tag `DATA-AGE-GRAPH-NODE-COUNTS-BY-SOURCE-PREFIXES`.

**Still open (capacity):** a single ≤32-prefix batch can still hit 300ms on huge graphs / contention. No timeout raise without EXPLAIN — see [08-r2-node-count-57014.md](08-r2-node-count-57014.md). Phase-2 denorm remains SPEC-089.

### R3 — Medium — Reverse-order / mixed-fleet upgrade

| Wrong order                                   | Effect                                                  |
| -----------------------------------------------| ---------------------------------------------------------|
| 0.24 API before migrate                       | Exit 78 fail-closed — OK                                |
| **0.22 API after mig 125** (`--confirm-drop`) | KV gone → old INV-03 KV-only → **false CRITICAL storm** |
| Mixed 0.22 writers + 0.24 readers             | Split-brain                                             |

See [04-residual-ops.md](04-residual-ops.md) upgrade checklist. Full runbook: [migrate-to-0.23.md](../../docs/operations/migrate-to-0.23.md).

### R4 — Low — LogOnly is `RepairTier::Safe`

`apply_repair(LogOnly)` only `info!` + `Ok(false)` — **no mutate**. Hourly may log “nothing to do”. Harmless; optional later: Manual tier for clarity.

### R5 — Low — Legacy scripts / docs still say graph `edgequake`

| Path | Risk |
|------|------|
| `migrations/support/040/apply.sql` | default graph name footgun |
| `docker/migrations/002_add_age_vertex_indexes.sql` | legacy create_graph |
| `docs/deep-dives/graph-storage.md` | tutorial cypher |

Not live inspector Default. Hygiene if those paths are re-run.

### R6 — Low — SAFE `DeleteOrphanedVectors` still mutates

Legacy INV-01 / INV-D path can DELETE from configured vector table. Mid-upgrade may fail or delete legacy rows only — not INV-03-shaped. Harden later (gate on table existence).

## Non-residual for this incident

- No remaining `graph_name: "edgequake"` in inspector Default (contract).
- No remaining `workspaces WHERE id` probe (contract).
- `documents WHERE id::text` elsewhere is **documents.id** PK — correct, not workspaces.

## Recommended next engineering (not blocking partner reply)

1. Multi-ws inspect loop / `?namespace=` (R1 / EC-05)
2. R2 capacity: EXPLAIN gate then optional Phase-2 denorm ([08](08-r2-node-count-57014.md)) — INV-C batching done
3. Optional LogOnly → Manual tier (R4)
