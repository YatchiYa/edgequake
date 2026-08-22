# Lens 001 — Product Owner

## Stake

Operators upload **scanned notebooks**. Today the product can look “alive” (Vision
Analysis cards) while **failing the job**: facts in handwriting never enter RAG.

## JTBD

> Ingest handwritten / MFD technical pages so search and Q&A answer from the page’s
> real content — tables, numbers, labels — with honest confidence.

## Value

| Outcome | Metric |
|---------|--------|
| Page facts in index | Table cell F1 + Key-value recall on gold |
| Trust | Modality chip + confidence visible; no scribble-as-answer |
| Cost control | MS DPI only when classified; print path unchanged |
| Privacy | No trigger content in repo |

## Acceptance (PO)

1. Print Acc path unchanged when modality=print.
2. Manuscript demo (synthetic) shows page MD with table/numbers, not crop monologue.
3. Failure modes documented; no claim of archival HTR.
4. SPEC-133 remains green for arrow-heavy diagram extracts.
5. SOTA alignment documented ([12-sota-assessment.md](../12-sota-assessment.md)).

## Non-goals

Classical HTR product SKU; marketing “100% handwriting OCR.”

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
