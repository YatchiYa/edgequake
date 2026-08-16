# 11 — Honest assessment

## What we know

- Reproduced delimiter collision on all five UI miss keys with current `rsplit` parse.
- Index-guided both-resolve recovers intended endpoints in lab when names exist.
- CHANGELOG / ops already admitted the residual — this spec closes that debt.

## What we do not claim

- That SPEC-130 alone made in-session immune (map miss / empty map still parse).
- That every multi-both-resolve pathological graph is uniquely decidable without escape encoding.
- That `:` inside names is fully solved (same composite-key family; follow-up).

## Residual risk

| Risk | Severity | Mitigation |
|------|----------|------------|
| Two real entity pairs format to one string | Low/rare | Escape keys (LAW-133-9); rightmost both-resolve interim |
| `:` in names breaks rel-type split | Low | Document; escape follow-up |
| Operators still re-run 139 | Ops | Doc update |

## Confidence

**High** that target-arrow parse is the root cause of this Failed document class.
**Medium** that production hit empty/incomplete known map vs map miss for those five rows — either way, index-guided parse is the correct fallback.

## Cross-refs

- Why: [00-why.md](00-why.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
