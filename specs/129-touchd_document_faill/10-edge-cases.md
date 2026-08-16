# 10 — Edge cases

| # | Case | Mitigation | Test |
|---|------|------------|------|
| E1 | `re_embedding` (#381) | → `processing` | T1/T2 |
| E2 | `queued` | → `pending` | T1/T2 |
| E3 | `merging` / `storing` / `gleaning` / `summarizing` / `uploading` / `converting` / `preprocessing` | → `processing` | T1 (+ sample T2) |
| E4 | `partial_success` | → `partial_failure` | T1 |
| E5 | `completed` | → `indexed` on write helper | T1/T2 |
| E6 | In-CHECK stages (`extracting`, `embedding`, …) | passthrough | T1 |
| E7 | `deleting` / `delete_failed` | passthrough (LAW-098-11) | T1/T2 |
| E8 | Empty / whitespace status | → `processing` | T1 |
| E9 | Unknown slug | → `processing` (default arm) | T1 |
| E10 | Missing documents row | non-fatal warn; no panic | existing touch behavior |
| E11 | Sidecar path without PdfStorage | same helper on SQL UPDATE | code wire |
| E12 | Stats refresh with rich stage | helper before bind | code wire |
| E13 | Concurrent resume + delete admit | lifecycle statuses not collapsed | T1 + SPEC-098 |
| E14 | Memory adapter parity | same helper / same mapping | unit memory touch if feasible |
| E15 | Case variants (`Re_Embedding`) | `to_ascii_lowercase` in normalize | T1 |

## Out of edge scope

- Durable `legacy_vector_id` collisions (#377) — separate spec/issue.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
