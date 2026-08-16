# 00 — Why SPEC-133

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) + UI evidence
[`evidence/ui-failed-995-of-1000.png`](evidence/ui-failed-995-of-1000.png).

A manuscript PDF finishes extraction, then fails at knowledge-graph persist:

```text
SPEC-091: typed fleet mirror resolved 995/1000 rows
… SPEC-098 misses: [ … keys with "->" inside endpoint names … ]
```

## Product WHY

```ascii
  Operator: “Reprocess keeps failing at 99.5% mirror — is the DB corrupt?”
  Support:  “Miss samples look like real edges, not missing entities.”
       │
       ▼
  Today (bug):
       RelVectors id = SRC->TGT:TYPE   (delimiter inside TGT)
       parse uses last "->"            (invents wrong SRC/TGT)
       SELECT relationships by wrong endpoints → miss
       fail-closed → document Failed
              │
              ▼
  Blind spots:
       1. Prior fix only covered arrow-in-SOURCE
       2. SPEC-130 UUID map hides the bug until map miss / empty
       3. Ops runbook says “reprocess after upgrade” — residual still open
```

## Five WHYs

1. **Why does persist fail?** Typed fleet mirror requires `resolved == eligible`; five relationship rows miss.
2. **Why do those rows miss?** Name resolve looks up `(src, tgt, rel)` that do not exist as a pair.
3. **Why are `(src,tgt)` wrong?** `parse_relationship_legacy_key` takes the **last** `->` as the separator; when the **target** name itself contains `->`, that last arrow is inside the target.
4. **Why does the key contain arrows in names?** Vision/LLM extraction of diagrams emits names like `ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)` and `LEFT_MARGIN_VALUE_1->_00_->_+`.
5. **Root cause:** Legacy relationship vector ids are an **unescaped composite key** (`{src}->{tgt}:{rel}`). Format is not invertible when either side contains the delimiter — a delimiter collision ([Wikipedia](https://en.wikipedia.org/wiki/Delimiter_collision)).

## Job to be done

> When entity names contain `->` (or `:`) characters that collide with the legacy relationship key grammar, fleet mirror (in-session and iw2/backfill) still resolves the correct endpoints — or fails with an unambiguous, operator-actionable class — so handwritten/diagram PDFs complete KG persist.

## Success criteria

1. All five screenshot miss keys parse to the intended `(src,tgt,rel)` when both entities exist in the name index.
2. Source-contains-arrow keys (`27_->_25_STRENGTHENING->CLAIM_FRONTIER:STRENGTHENS`) still parse correctly (no regression).
3. SPEC-130 UUID map remains the primary in-session path (DRY: parse is fallback, not duplicate identity).
4. Fail-closed behavior preserved when neither split resolves (no silent wrong FK).
5. Unit + e2e gates prove the matrix in [10-edge-cases.md](10-edge-cases.md).

## Reproduction (lab)

```ascii
  format("FLOW_DIRECTION", "ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET)", "RELATED_TO")
    = FLOW_DIRECTION->ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET):RELATED_TO

  parse rsplit → (FLOW_DIRECTION->ARROW_1_(SHADED_BOX_ , CIRCULAR_TARGET) )  ✗
  parse index  → (FLOW_DIRECTION , ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET) )  ✓
                 when both names exist in EntityNameIndex
```

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
