# 06 — Edge Cases (SPEC-110)

| Case | Expected behavior |
|------|-------------------|
| Same `document_id` under 2+ workspaces in wsdoc | Patched 118 inserts **one** row; `workspace_id` = lexicographic min of candidate `ws_id`s that pass UUID + FK guards |
| Document already has non-NULL `workspace_id` | `COALESCE` keeps existing; proposed pick ignored |
| Document exists with NULL `workspace_id` | COALESCE fills from EXCLUDED (winning ws) |
| Workspace FK missing for one of two keys | That key skipped by `EXISTS`; other may win alone — no 21000 |
| Non-UUID segment in key | Skipped by regex; no insert |
| Empty `eq_*_kv` / no wsdoc keys | No-op; migration succeeds |
| Multiple `eq_*_kv` tenant tables | Loop applies per table; each statement must still obey LAW-M1 independently |
| Same doc id appears in two kv tables | Second table upsert: COALESCE / existing row rules apply; still ≤1 proposed id per statement per table |
| Injection id under two workspaces (121) | One row after `DISTINCT ON (inj_id)`; UPDATE overwrites typed fields from winning row |
| Injection only one workspace | Unchanged behavior vs pre-fix |
| `ON CONFLICT DO NOTHING` siblings (117) | Out of scope; duplicates OK under DO NOTHING |
| Partner stuck@117 re-runs patched image | 118 applies once; continues 119+ |
| Fleet already applied **old** 118 | Checksum mismatch → loud refuse; DEV_MODE or manual UPDATE then proceed (no re-exec of 118 body) |
| Fleet already applied **fixed** 118 | No repair; checksum matches |
| Concurrent migrate two pods | Advisory lock serializes; one winner |
| `--confirm-drop` with pending DROP OLD | Unrelated to 118; after 118 fixed, drop gates unchanged |
| Partial apply mid-118 before patch | sqlx transaction → no partial documents from failed 118 |
| Lexicographic UUID order ≠ “primary” workspace intent | Accepted LAW-M5; ops may re-scope document post-migrate if wrong WS won |
| Document content empty after 118 | Expected (wsdoc is membership only); shell/content backfill 122 fills later |

## Residual risk after patch

| Risk | Severity | Mitigation |
|------|----------|------------|
| Wrong workspace chosen for multi-member doc | Low–med | Deterministic pick + COALESCE; document in ops reply; KV remains until 125 for audit |
| Operator skips DEV_MODE repair on applied-old fleets | Med | Loud Protocol error with runbook ([09](09-ops-runbook.md)) |
| Image pin left on 0.24.1 | High | Partner reply + release notes |
