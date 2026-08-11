# Lens 004 — UX / UI

## Stance

**No UI redesign.** Success is the absence of a failure state.

## As-is failure UX (bad)

Document / email shows Failed with raw storage constraint text mentioning `idx_*_legacy_vector_id`. Users cannot act on it except “retry later” or ask ops to lower concurrency.

## Target UX

| State | Copy / behavior |
|-------|-----------------|
| Happy concurrent ingest | Status progresses to Completed as today |
| Absorb path | Invisible — no toast, no Failed |
| Unrelated GraphMerge | Existing error UX unchanged |

## Guard

Do not add “legacy vector collision absorbed” banners. Ops may use logs / metrics only.
