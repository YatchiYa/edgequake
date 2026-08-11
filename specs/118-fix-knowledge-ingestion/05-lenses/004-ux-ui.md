# Lens 004 — UX / UI

## Surfaces

| Surface | v1 change |
|---------|-----------|
| Injection list / detail status | None required if backend completes |
| Failed injection error string | Already shows worker error; becomes success after fix |
| Onboarding / glossary wizards | Out of scope |

## Honesty

- Status must transition `processing → completed` (or honest `failed` for unrelated errors).
- Do not hide SPEC-058 vector-dimension failures behind empty success.
- If UI shows raw `invalid uuid 'injection::…'`, that is a backend bug symptom — fix backend, not copy.

## Copy (if needed later)

No new locked marketing copy in v1. Optional ops hint in docs only:

> Knowledge injection stores glossary text under a namespaced document id for citation exclusion; relational storage uses the injection UUID.
