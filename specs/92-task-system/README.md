# SPEC-120 — First-Class Task System

> **Status:** DESIGN + ORPHANED WIP — **not wired / not releasable**
> **Live production path today:** SPEC-057 cancel + fairness — [`docs/ingestion-cancel-and-fairness.md`](../../docs/ingestion-cancel-and-fairness.md) (`POST /api/v1/tasks/{id}/cancel`)
> **Hub:** [SPEC-120 hub](./SPEC-120%20%E2%80%94%20First-Class%20Task%20System%20for%20EdgeQuake%20(%20f2154512c0514e8e8d10cfbbc3f87c2b.md)

## Honest assessment (2026-07-29)

| Layer | Reality |
| --- | --- |
| Product lenses (1–8) | Design-complete |
| Migrations 112–115 | Present on disk (cancel fence, P1 states, P2 jobs/attempts, transition trigger) — **ahead of Rust enums** |
| API handlers (`operations.rs`, presentation, `cancel_notify`, `fenced_write`, stage mirror, …) | Drafted files exist; **not** declared in `handlers/mod.rs` / `routes.rs` / OpenAPI → `/api/v1/operations/*` is **404** |
| `edgequake-tasks` modules (`capacity_block`, `cancel_decision`, `job`, `p2_store`, …) | Orphaned — not all exported from `lib.rs` |
| Lens 1 INV-1..3 (cross-replica cancel, fence, delete saga) | **Not met** |
| Contract tests | Quarantined under `edgequake-api/tests-wip-spec120-capacity/` (do not compile or would 404) |

Lens 1 rule: **do not ship durable cancel without the fence** — that is worse than today's process-local cancel because it invites false trust.

## Roadmap checklist

### P0 — Trust (required before any `/operations` claim)

- [ ] Durable `cancel_requested_at` + `cancelling` status on task rows
- [ ] Cross-replica cancel delivery (`pg_notify` / LISTEN → registry) + `LeaseVerdict` heartbeat path
- [ ] Document fence epoch on writers; reject post-terminal side effects
- [ ] Delete saga: cancel dependents → fence → purge → verify (keep task rows until done)
- [ ] Wire `operations` routes + OpenAPI + presentation DTOs
- [ ] Restore INV-1 / INV-3 / fence contracts from quarantine

### P1 — Scale

- [ ] Richer states (`held`, `dead_letter`, `available_at`) live in Rust + claim SQL
- [ ] Indexed `document_id` lookups (INV-7)
- [ ] Named `document.reprocess` `TaskType`

### P2 — Fairness

- [ ] Durable tenant lane / vruntime ledger used by claim ranking
- [ ] Job graph (`jobs` / `attempts` / `task_events`) wired

### P3 — Transparency

- [ ] Queue position + global metrics surfaces
- [ ] Purge receipts
- [ ] UI Stopping affordance backed by stored cancel intent (INV-6)

## Overlap with SPEC-091

Risk **R-22**: `document_stage_mirror` must not become a second writer racing SPEC-091 relational shells. Reuse one typed CAS path when P0 lands.

## Quarantined tests

See [`edgequake/crates/edgequake-api/tests-wip-spec120-capacity/README.md`](../../edgequake/crates/edgequake-api/tests-wip-spec120-capacity/README.md).

### IW3 descope (2026-07-30)

SPEC-091 IW3 explicitly **does not** wire SPEC-120 P0. Each quarantined contract
remains out of CI until a dedicated SPEC-120 sprint lands the trust layer (Lens 1).

| Test file | IW3 disposition | Reason |
| --- | --- | --- |
| `contract_spec120_operations.rs` | **Descoped** | `/api/v1/operations/*` not mounted; `handlers/operations.rs` orphaned |
| `contract_spec120_task_presentation.rs` | **Descoped** | `cancel_decision.rs` references `TaskStatus::Cancelling` / `cancel_requested_at` not on live `Task` |
| `contract_spec120_capacity_wait.rs` | **Descoped** | `capacity_block.rs`, `CapacityLayer`, fairness hold APIs not exported from `edgequake-tasks` |
| `contract_spec120_inv3_deletion_saga.rs` | **Descoped** | `TaskStatus::DeadLetter` + delete saga fence unwired |
| `contract_spec120_document_stage_ssot.rs` | **Descoped** | `document_stage_mirror.rs`, `FenceEpoch`, `LeaseVerdict` cancel path unwired |
| `contract_spec120_inv1_multi_replica.rs` | **Descoped** | `cancel_notify.rs` not in `services/mod.rs`; LISTEN worker not spawned at boot |
| `contract_spec120_fence.rs` | **Descoped** | `fenced_write.rs` unwired; writer fence epoch not enforced |
| `contract_progress_counts_ssot.rs` | **Descoped** | `progress_counts.rs` + stage mirror not integrated into list/operations surfaces |

**Next owner:** SPEC-120 P0 checklist above — wire modules, expand enums/SQL, then move tests from `tests-wip-spec120-capacity/` → `tests/`.

## Related

- SPEC-091 upgrade (ships independently): [`docs/operations/spec091-upgrade-from-v0.22.0.md`](../../docs/operations/spec091-upgrade-from-v0.22.0.md)
- Live cancel/fairness: [`docs/ingestion-cancel-and-fairness.md`](../../docs/ingestion-cancel-and-fairness.md)
