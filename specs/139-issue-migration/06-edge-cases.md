# 06 — Edge cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC1 | Two surface names, one normalize | Last-write-wins; first provenance COALESCE | E2E-139-01 |
| EC2 | Rel type case fold | Same arbiter `relationship_id` | E2E-139-02 |
| EC3 | Report ids unique per table | Dedupe still applied (defense) | unit helper |
| EC4 | Unique `legacy_vector_id` 23505 | Existing: count as failed, do not crash | SPEC-111 |
| EC5 | Dual-legacy stalls after copy | Stamp job; 131 still fail-closed | SPEC-111 runbook |
| EC6 | W3 verify-failed after W1 adds spine | Reclaim + cursor reset | E2E-139-04 |
| EC7 | Equality mismatches, coverage OK | Default `passes()` ignores mismatches | unit + env=1 opt-in |
| EC8 | Guard before 106+ | 42P01 typed SSOT → migrate-first message | residue match + 1_guard.log |
| EC9 | Remainder on empty KV | estimate 0 / one empty batch | remainder estimate |
| EC10 | iw2 preflight after 21000 | Same sha, gen 1, retry with new binary | field + 01 |
| EC11 | `failed` from crash (no verify_failed) | Not auto-reclaimed | reclaim WHERE |
| EC12 | Many `eq_*_ws_*_vectors` tables | Fleet enum + SUM actual | E2E-139-03 |
| EC13 | Malformed `*-chunk-*` vector id | W3 `expected` uses UUID regex ≡ 126, not `LIKE '%-chunk-%'` | E2E-139-03 |
| EC14 | Remainder leftover orphans | Job `verify` is copy-complete; DROP/advisor stay RED | E2E-139-08 |
| EC15 | 4 field `doc_shells` | `wc-shell-remainder` replays 122 | E2E-139-07 |

## AGE note

Confirm-drop still must not `DROP SCHEMA` graph namespaces (LAW-137-7).
