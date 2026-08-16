# 09 — Acceptance

| ID | Criterion | Proof |
|----|-----------|-------|
| AC-1 | Typed RelGraph completes before RelVectors (invariant retained) | T1 |
| AC-2 | In-session mirror uses sink relationship UUIDs | T2, T3 |
| AC-3 | Duplicate-name / oldest-vs-last class no longer fails with map | T2 |
| AC-4 | Fail-closed hint names relationship identity (not entity spine only) | T4 |
| AC-5 | Coverage / offline name resolve still works | T5 |
| AC-6 | `->` in source name still supported | T6 / SPEC-091 contracts |
| AC-7 | #380 has maintainer comment linking SPEC-130 | C1 |
| AC-8 | DRY: one legacy-key + one identity producer | WP checklist + review |
| AC-9 | No sleep/retry as primary fix | Code review / LAW-130-8 |

## Done when

- Doc pack complete (D0–D1).
- C1 comment posted.
- After implementation WPs: T1–T6 green and AC-1…AC-9 checked.

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- PO: [05-lenses/001-product-owner.md](05-lenses/001-product-owner.md)
