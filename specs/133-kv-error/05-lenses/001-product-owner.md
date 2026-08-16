# Lens 001 — Product Owner

## Pain

Operators upload diagram / handwritten PDFs, watch merge reach ~100%, then see
**Failed** with a database-looking error. Support is told to “ensure spine /
re-run 139” — wrong class — wasting time and trust.

## Outcome

- Manuscript/diagram docs complete KG persist when entities were actually extracted.
- Error copy (when still failing) distinguishes “ambiguous legacy key” from “spine missing”.
- Reprocess after upgrade is a real fix, not a roulette spin.

## Acceptance (PO)

1. Reprocess of `0001_Note_manuscrite.pdf` class (arrow-in-target names) reaches Completed / indexed under typed embeddings.
2. Near-miss `995/1000` with zz-raw miss samples does not recur on that class.
3. No DDL / fleet drop migration required for this ship.

## Non-goals

- Changing extraction quality of handwriting OCR.
- Softening fail-closed mirror for true FK gaps.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
