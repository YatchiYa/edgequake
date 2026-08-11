# Lens 001 — Product Owner

## Problem in user language

Deleting or reprocessing a document fails on large workspaces with a database timeout. Knowledge stays half-cleaned: the document may leave the list while graph edges still cite it.

## Outcome

Operators and end users can delete/reprocess documents on large graphs without raising DB timeouts or filing support tickets.

## Acceptance (product)

1. Delete completes for documents that previously timed out on singular-edge discovery.
2. Reprocess retracts graph citations without the same timeout class.
3. Failure copy (if any) is understandable — not a raw Postgres cancellation.
4. No regression on correctness of Symptom F cleanup (orphan singular citations still removed).

## Priority

P0 reliability — blocks core document lifecycle on growing deployments.

## Non-goals

- Marketing claims of “instant delete” for multi-million-edge graphs without further batching work.
- Changing product meaning of delete (still fail-closed on post-proof residue).
