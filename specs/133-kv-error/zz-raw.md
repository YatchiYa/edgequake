# zz-raw — Intake (not the contract)

## UI symptom (2026-08-16)

Document: `0001_Note_manuscrite.pdf` → status **Failed**.

```text
Knowledge graph persist failed: Graph error: 1 knowledge-graph merge error(s)
during persist: Storage error: Database error: SPEC-091: typed fleet mirror
resolved 995/1000 rows (relational entity/rel FK miss or name mismatch —
bare entities.name must match entity:NAME; ensure PostgresEntitySink wrote
the spine before fleet mirror; SPEC-098 misses: [
  "LEFT_MARGIN->LEFT_MARGIN_VALUE_1->_00_->_+:RELATED_TO",
  "SMALL_BOXED_LABEL_T.->LEFT_MARGIN_LABEL_1->_00_->_+:RELATED_TO",
  "FLOW_DIRECTION->ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET):RELATED_TO",
  "FLOW_DIRECTION->ARROW_2_(CIRCULAR_TARGET_->VERTICAL_PANEL):RELATED_TO",
  "LEFT_MARGIN_SEQUENCE->SEQUENCE_1->_00_->_+:RELATED_TO"
])
```

Evidence screenshot: [evidence/ui-failed-995-of-1000.png](evidence/ui-failed-995-of-1000.png)

## Operator notes

- Near-complete miss class (`995/1000`), not `0/N` spine-absent.
- Miss samples are **relationship** legacy keys with `->` inside endpoint names
  (handwriting / diagram extraction from a manuscript PDF).
- Reprocess alone does not fix while parse remains ambiguous.

## Pointers already in tree

- CHANGELOG: `rsplit_once("->")` fixed **source**-contains-arrow; residual **target**-contains-arrow.
- Ops: `docs/operations/spec098-entity-spine-ensure.md` § near-complete mirror misses.
- SPEC-130: sink RETURNING UUID map bypasses parse for in-session RelVectors when map is complete.
