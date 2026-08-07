# 01 — First Principles (SPEC-110)

> **Cross-refs**: [WHY](00-why.md) · [Issue data](00-issue-data.md) · [RCA](03-root-cause.md) · [Fix](04-fix-plan.md)

## Axioms

1. **PostgreSQL INSERT … ON CONFLICT DO UPDATE is deterministic.** A single command must not affect any existing row more than once. Duplicate constrained values in the *proposed* set raise `21000` / U126 — not “last write wins”.
2. **`SELECT DISTINCT` dedupes whole rows, not conflict keys.** Distinct `(doc_id, ws_a)` and `(doc_id, ws_b)` both survive.
3. **Embedded migration SQL is the law of the released image.** Field operators run the binary’s `sqlx::migrate!` payload, not a mutable volume of `.sql` files.
4. **A migration that never records success can be re-run.** sqlx wraps applies in a transaction by default; failure at 118 leaves `latest_applied = 117`.
5. **A migration that already recorded success cannot silently change body.** sqlx verifies SHA checksums; mismatch → `VersionMismatch` unless an explicit repair path updates `_sqlx_migrations`.

## Causal diagram

```text
 Legacy KV membership index
   wsdoc:WS1:DOC
   wsdoc:WS2:DOC          ← same document_id, two workspaces (legitimate)
        │
        ▼
 SELECT DISTINCT (doc_id, ws_id, …)   ← keeps BOTH rows
        │
        ▼
 INSERT … ON CONFLICT (id) DO UPDATE  ← proposes id=DOC twice
        │
        ▼
 Postgres: cannot affect row a second time (21000)
        │
        ▼
 migrate aborts; 118 not recorded; partner stuck on 0.24.1 image
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-M1** | **Conflict-key cardinality** — Any `INSERT … ON CONFLICT (K) DO UPDATE` must propose **≤1 row per K** in one statement. |
| **LAW-M2** | **Dedup on the arbiter** — Collapse with `DISTINCT ON (conflict_cols)` + deterministic `ORDER BY`. Never rely on `DISTINCT` over a wider tuple. |
| **LAW-M3** | **Blocking migration edit** — When a released migration *blocks* fleets that have not applied it, edit that version in place, update `checksums.lock`, and ship M078-style checksum repair for fleets that already applied the old body. Append-only `NNN+1` cannot unblock a failing `NNN`. |
| **LAW-M4** | **Embedded SQL ⇒ patch release** — Fixing field migrate requires a new binary/image (target **v0.24.2**). Repo-only edits do not help GHCR `0.24.1`. |
| **LAW-M5** | **wsdoc collapse** — Relational SSOT is one `documents.workspace_id` per `id`. Multi-membership collapses to one row: pick lexicographic min `ws_id`; `COALESCE` never overwrites a non-NULL scope. Leftover KV keys remain until drop wave 125. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | Migration SQL owns backfill shape; bootstrap reconcile owns checksum repair; release process owns image cut. |
| **O** | New backfill migrations extend the train; LAW-M1/M2 are the open extension point for upsert safety. |
| **L** | Every `ON CONFLICT DO UPDATE` backfill must obey LAW-M1; `DO NOTHING` is a weaker but allowed sibling. |
| **I** | No mega “migrate fixer” CLI — small repair modules (`m118` / `m121`) mirroring `m078`. |
| **D** | Ops runbooks depend on published image tags + documented repair flags, not ad-hoc SQL dumps of embedded migrations. |
| **DRY** | Reuse M071/M078 DEV_MODE-gated checksum repair; do not invent a second immutability story. |

## Normative dedup sketch

```sql
INSERT INTO public.documents (id, workspace_id, content, status)
SELECT DISTINCT ON (doc_id) doc_id, ws_id, '', 'indexed'
FROM (
    SELECT split_part(kv.key, ':', 3)::uuid AS doc_id,
           split_part(kv.key, ':', 2)::uuid AS ws_id
    FROM eq_…_kv kv
    WHERE kv.key LIKE 'wsdoc:%'
      -- uuid + FK guards unchanged
) src
ORDER BY doc_id, ws_id
ON CONFLICT (id) DO UPDATE SET
    workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id);
```

## Postgres references (external law)

- [PostgreSQL INSERT — ON CONFLICT](https://www.postgresql.org/docs/current/sql-insert.html): “The command will not be allowed to affect any single existing row more than once.”
- [pganalyze U126](https://pganalyze.com/docs/log-insights/app-errors/U126): duplicate constrained values in the proposed set.
