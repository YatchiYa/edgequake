# 00 — Why SPEC-111

## Trigger

Partner issues in this pack:

| # | Reporter | Reported version | Theme |
|---|----------|------------------|-------|
| [364](https://github.com/raphaelmansuy/edgequake/issues/364) | @ravimohta | 0.23.0 | Vector drop readiness paradox |
| [363](https://github.com/raphaelmansuy/edgequake/issues/363) | @ravimohta | 0.23.0 | iw2 backfill false GREEN |
| [362](https://github.com/raphaelmansuy/edgequake/issues/362) | @ravimohta | 0.23.0 | KV residue advisor timeout |
| [361](https://github.com/raphaelmansuy/edgequake/issues/361) | @ankursingh-devops | 0.12.11 | Bulk upload slow |
| [360](https://github.com/raphaelmansuy/edgequake/issues/360) | @ankursingh-devops | 0.12.11 → **clarified 0.24.1** | Clear All incomplete |
| [366](https://github.com/raphaelmansuy/edgequake/issues/366) | @ankursingh-devops | **0.24.1** | Clear All incomplete (correct pin) |

Issues **362–364** are one migrate-advisor cluster. **#360/#366** are the Clear All dual-SSOT defect on the published pin (durable wipe #309 is necessary but not sufficient). **#361** remains capacity.

## User impact

| Layer | Impact if ignored |
|-------|-------------------|
| Ops | Partners cannot trust `edgequake migrate dry-run` GREEN/RED; pause before irreversible `--confirm-drop` |
| Data | iw2 can “complete” while typed embeddings cover ≪ legacy fleet → silent query degradation after flip |
| Perf | Advisor timeout blocks drop-readiness on modest KV tables (~72k rows) |
| UX | Clear All ghosts on v0.24.1 (#366) break trust; bulk upload (#361) may be capacity |

## Why this pack (not five unrelated tickets)

1. **Code is law** — 362–364 share the migrate advisor / migration-engine module; one DRY fix plan.
2. **Partner trust** — 364 is correctly cautious: irreversible drop must not be forced past a misunderstood gate.
3. **Version honesty** — honor reporter corrections (#360→#366 on 0.24.1); do not close Clear All as “historical” without list⊆wipe proof.

## Non-goals

- Hot-patching partner containers without a release.
- Redesigning AGE→relational entity identity (beyond normalize + coverage reporting).
- Making bulk PDF+LLM ingest “instant” without measurement.

## Success condition

- Each GitHub issue has an evidence-based status comment.
- This pack names root cause, fix plan (SOLID/DRY), and e2e gates for every **still-present** defect.
- Engineering can implement Cluster A without inventing a second readiness model.
