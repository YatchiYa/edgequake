# SPEC-120 capacity / operations tests — WIP quarantine

These contract tests are **not** part of `tests/` discovery. They either **do not compile** against the live crate surface, or they assert SPEC-120 behaviours that are **not wired** (e.g. `/api/v1/operations/*` → 404).

Quarantined during SPEC-091 Wave completion so `cargo test` / clippy gates are not blocked by an unfinished feature. **Not SPEC-091 scope.** Status tracker: [`specs/92-task-system/README.md`](../../../../../specs/92-task-system/README.md).

## Files

| File | Why quarantined |
| --- | --- |
| `contract_spec120_operations.rs` | Routes not mounted → 404 (was failing in `tests/`) |
| `contract_spec120_task_presentation.rs` | Source asserts for unwired presentation / CancelDecision |
| `contract_spec120_capacity_wait.rs` | `CapacityLayer`, `stamp_capacity_block`, `mark_fairness_hold` missing |
| `contract_spec120_inv3_deletion_saga.rs` | `TaskStatus::DeadLetter` missing |
| `contract_spec120_document_stage_ssot.rs` | stage-mirror / `FenceEpoch` / `LeaseVerdict` / cancel-with-wake missing |
| `contract_spec120_inv1_multi_replica.rs` | `Cancelling`, `cancel_requested_at`, LISTEN path missing |
| `contract_spec120_fence.rs` | fence APIs unwired |
| `contract_progress_counts_ssot.rs` | progress-counts / stage-mirror unwired |

## Orphaned source (never wired into crate roots)

- `crates/edgequake-tasks/src/capacity_block.rs`
- `crates/edgequake-tasks/src/provider_capacity.rs`
- `crates/edgequake-api/src/handlers/operations.rs`
- `crates/edgequake-api/src/handlers/operation_presentation.rs`
- `crates/edgequake-api/src/services/cancel_notify.rs`
- `crates/edgequake-api/src/services/fenced_write.rs`
- `crates/edgequake-api/src/services/document_stage_mirror.rs`
- (+ related progress/queue helpers)

## To restore

1. Wire P0 modules into `edgequake-tasks` / `edgequake-api` crate roots + routes + OpenAPI.
2. Expand `TaskStatus` / `Task` fields / `LeaseVerdict` as required by Lens 1–3.
3. Move these files back to `tests/` and green them before claiming SPEC-120 shipped.
