# Lens 001 — Product Owner

## Pain / Stake

Operators with dense extraction corpora (invoices, remittance) see large Failed cohorts (reporter: **199 / 9825**). Documents show entities and relationships in SQL, yet every reprocess fail-closes on typed fleet mirror `0/N`. Trust erodes: “the data is there — why is the product red?”

## Outcome

- Dense documents complete KG persist when the relational spine was written in the same merge.
- Reprocess of previously-failed docs succeeds without “retry lottery.”
- Error copy stops sending engineers down a false race / entity-spine rabbit hole.

## Acceptance (PO)

1. New ingest of multi-relationship documents does not systematically land Failed on `typed fleet mirror resolved 0/N` for relationships that the sink wrote.
2. Reprocess of a failed doc from the #380 class succeeds after the fix (or fails with a **true** integrity reason).
3. Issue #380 closed or marked fixed with link to SPEC-130 proofs.

## Non-goals

- Guaranteeing 100% extraction quality / entity density caps product decisions.
- Hiding failures by softening fail-closed.
- UI redesign of the Documents list.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- Honest assessment: [../11-honest-assessment.md](../11-honest-assessment.md)
