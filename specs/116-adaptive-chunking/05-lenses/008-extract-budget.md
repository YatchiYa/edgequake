# Lens — Extract Budget (Ops / Product)

> Companion to [`../12-extract-budget-first-principles.md`](../12-extract-budget-first-principles.md) and  
> [`../13-extract-budget-brainstorm.md`](../13-extract-budget-brainstorm.md).

## Job story

> As an ops/partner lead, I want a clear rule for **per-chunk entity/relation caps** so we stop timeouts and dual-SUT confounds without “tuning K to make the card look denser.”

## Answer in one line

**Keep per-chunk budgets (40/100 LR-parity). Fix geometry and selection before raising K.**

## Decision tree

```ascii
  Extract timeout / endless local output?
      yes → keep hard caps + check provider max_tokens
  Dual-SUT denser than LightRAG?
      → Acc-fair chunking first (SPEC-116), confirm K matched 40/100
  “Too few entities” on card?
      → Check M vs U; check %truncated; Acc-fair N; then gleaning / model — not K↑
  Dense medical chunk, high truncation rate on fair geometry?
      → gleaning continue OR modest K↑ with Acc remeasure
  Want better multi-hop?
      → better selection under K + schema (SPEC-114) + extract model (doc 10/11)
```

## Requirements

| ID | Requirement |
|----|-------------|
| EB-1 | Fleet default remains 40 ents / 100 rows (LR parity) |
| EB-2 | Soft prompt + hard truncate both required |
| EB-3 | Document that adaptive \(N\) × saturate-\(K\) inflates \(M\) |
| EB-4 | No UI “raise cap” without truncation metrics |
| EB-5 | Any default K change needs Acc honesty (SPEC-001/054 lesson) |

## Copy seeds

- “Each chunk may contribute at most 40 entities and 100 total extract rows — same as LightRAG.”  
- “More chunks still mean more total mentions even with that cap.”  
- “Raising the cap is a last resort after fair chunking and gleaning.”

## Success metric

Partner can explain density without proposing “set MAX_EXTRACTION_ENTITIES=200” as the first fix.
