# Lens 001 — Product Owner

## Job

Restore partner trust that knowledge injection (glossary / enrichment) works on the default PostgreSQL product path after SPEC-091 Wave D.

## Decisions

| Decision | Rationale |
|----------|-----------|
| Map injection UUID for relational FK | Completes typed chunk SSOT; parent row already exists |
| Keep composite IDs for citations | Do not regress “enrich without citing glossary” |
| Ship CI e2e under relational authority | Prevent silent regression (#376 CI blind spot) |

## Risks

| Risk | Mitigation |
|------|------------|
| Skip-only hotfix ships “green” but empty chunks | Acceptance requires `chunks` rows |
| Partners still blocked by unrelated vector dim mismatch (SPEC-058) | Document separately; not SPEC-118 scope |
| Handler ignores path workspace_id | Note as follow-up; not blocking identity fix |

## Success (PO)

- Injection status reaches `completed` on Docker/latest + Postgres + default authority
- No user-facing API change required
- Issue #376 closed with linked acceptance evidence
