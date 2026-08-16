# 11 — Honest assessment

## What this pack fixes (when implemented)

- Correct diagnosis of #380 vs unordered-race narrative.
- In-session relationship identity retained sink → mirror.
- Deterministic duplicate-name / re-lookup miss class for RelVectors.
- Operator-facing error hint accuracy.
- Spec + e2e contract for order + UUID map.

## What this pack does **not** fix

- Legacy (non-typed) RelVectors-before-RelGraph ordering (product default is typed).
- Placeholder endpoints without entity spine.
- Extraction quality, caps, or LLM cost.
- Historical Failed rows auto-heal without reprocess (reprocess after fix should work).
- Visibility races in exotic multi-pool setups if someone bypasses the map (map is the contract).

## Residual risk

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Trait return-type churn across mocks | Medium | WP-1 mechanical; compile gates |
| RETURNING shape under ON CONFLICT | Low | Explicit SQL test T3 |
| Target names containing `->` | Low | Document; prefer UUID path |
| Operators still believe “race” | Medium | #380 comment + README verdict |

## Confidence

| Claim | Confidence |
|-------|------------|
| Typed RelGraph → RelVectors exists | High (source) |
| Pure timing race is not dominant | High (order + compensation + identical retries) |
| Identity discard is the first-principles gap | High |
| Duplicate-name oldest/last is a concrete miss class | Medium–High (code paths proven; live corpus not required) |
| UUID map eliminates in-session miss class | High (by construction) |

## Cross-refs

- Why: [00-why.md](00-why.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
