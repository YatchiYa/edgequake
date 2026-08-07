# issue-362 — KV residue advisor `::text` cast

**GH:** https://github.com/raphaelmansuy/edgequake/issues/362  
**Status:** Confirmed present on HEAD and **v0.24.1**  
**Severity:** P1 (migrate dry-run / readiness blocked)

## WHY

Drop readiness must complete on modest fleets. A cast that disables the PK index is a defect, not “scale”.

## Code law

```sql
-- BAD (residue.rs + migration 125 shells/artifacts)
d.id::text = substring(k.key from '…')

-- GOOD (already used for chunks in same query)
c.document_id = left(k.key, 36)::uuid
```

External confirmation: casting indexed columns defeats btree use ([dba.SE](https://dba.stackexchange.com/questions/277981/understanding-index-w-cast)).

## Fix (summary)

Cast substring → `::uuid`; patch advisor + 125 together (LAW-C3). Phase D in [04-fix-plan](04-fix-plan.md).

## Workaround (ops)

Temporarily raise `statement_timeout` for one advisor run — confirms logic, not a ship fix.
