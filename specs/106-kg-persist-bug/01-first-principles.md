# 01 — First Principles (SPEC-106)

## Axioms

1. **PostgreSQL only executes operators that exist in the catalog.** If AGE does not register `=` for `graphid`, any SQL that compares two `graphid` values fails at parse/plan time — even on empty tables.
2. **A fixed sibling does not fix an unfixed call site.** Issue #214 proved the law; one leftover JOIN reopens the incident class.
3. **Persist reads and viz reads share the same type system.** Relationship merge and degree queries must obey one comparison policy.

## Laws

| Law | Statement | Honored pre-106? |
|-----|-----------|------------------|
| **LAW-G1 — Graphid compare SSOT** | Joins/filters on `Node.id` / `EDGE.start_id` / `EDGE.end_id` cast via `::text`, or avoid graphid via property/`eq_*` text keys. Never `graphid = graphid`. | Violated (`pg_get_edges_for_nodes_batch`) |
| **LAW-G2 — AGE E2E for graphid SQL** | Any SQL that touches graphid adjacency has a Postgres AGE regression (not Memory-only). | Violated (#214 tests were Memory) |

## DRY / SOLID

```ascii
 LAW-G1 ─▶ one ::text join idiom (degrees + edges-batch + scan + search)
        ─▶ SRP: storage owns AGE type quirks; merger only calls GraphStorage
        ─▶ OCP: M072 text indexes already serve the cast predicates
```
