# Lens 006 — Marketing and Growth

## Narrative (internal → external)

**Internal truth:** Large-graph delete was timing out because a correctness probe lacked indexes.

**External-safe claim (after ship):** Document lifecycle stays reliable as your knowledge graph grows — delete and reprocess clean citations without silent leftovers.

## Do not claim

- “Unlimited graph size with fixed 2s deletes” without measurement.
- That parent-table indexes were the fix (technical debt narrative stays internal).

## Growth angle

Reliability of delete/reprocess is a trust unlock for:

- Enterprise pilots with large corpora
- Re-ingestion / correction loops (reprocess)
- Support deflection (fewer “delete stuck / timeout” tickets)

## Release note snippet (when shipped)

> Fixed document delete/reprocess timeouts on large knowledge graphs by indexing singular edge citation fields used during cleanup (SPEC-119 / #375).
