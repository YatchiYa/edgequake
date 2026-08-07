# 07 — Cluster notes

## Cluster A — SPEC-091 migrate (362, 363, 364)

One partner investigation, one module family (`migration_engine`), one release train.

| Order | Why |
|-------|-----|
| Fix #363 first | Without real coverage, “ready to drop” is meaningless |
| Then #364 | Advisor must speak coverage language |
| Then #362 | Unblocks dry-run on large KV; patch with 125 parity |

Related: [SPEC-110](../110-migration-issue/) blocks some partners earlier (migration 118). Ship narrative: **v0.24.2** can carry 110 + 111 Cluster A.

## Cluster B — Documents (#366/#360, #361)

Clear All is a **live** dual-SSOT defect on v0.24.1 (#366), not merely historical. #361 remains capacity. Ship Clear All with Cluster A in **v0.24.2** CHANGELOG; keep PR scope separable.

## Similar historical issues

| Item | Overlap |
|------|---------|
| #309 durable wipe | Direct ancestor of #360 fix path |
| SPEC-090 | Explains #361-class latency |
| LAW-C3 advisor↔125 | Same parity demand as #364 advisor↔126 |
| Partner regenerate embeddings | Bridges #363 failure → #364 verify secondary |

## What we did **not** claim

- That `--confirm-drop` is impossible today when advisor shows RED for vectors — SQL guard may still pass. The defect is **truthfulness**, not necessarily a hard CLI block.
- That #360/#361 reproduce on v0.24.1 without a live repro session.
