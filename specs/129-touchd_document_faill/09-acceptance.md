# 09 — Acceptance

| ID | Criterion | Proof |
|----|-----------|-------|
| AC-1 | `touch_document_status("re_embedding")` does not violate CHECK | T2 |
| AC-2 | Column becomes `processing` after that touch | T2 |
| AC-3 | Raw SQL `re_embedding` still rejected | T2 |
| AC-4 | KV resume path still writes `re_embedding` | T3 honesty |
| AC-5 | All listed writers use SSOT helper | T3 + code review |
| AC-6 | Lifecycle `deleting` / `delete_failed` passthrough | T1/T2 |
| AC-7 | `completed` touch → `indexed` | T1/T2 |
| AC-8 | No new migration for CHECK widen | git diff migrations |
| AC-9 | #381 WARN absent on happy-path resume (optional soak) | T4 |

## Done when

All AC-1…AC-8 green; SPEC pack merged; issue #381 comment posted with root cause + SPEC link.

## Cross-refs

- Tests: [08-test-protocol.md](08-test-protocol.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
