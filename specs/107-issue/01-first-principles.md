# 01 — First Principles (SPEC-107)

> Method: map partner SQLSTATE / app signals to axioms. **Laws are SPEC-104 LAW-I1..I4** — this pack does not invent parallel law IDs. Deep text: [SPEC-104 01](../104-fix-datalayer/01-first-principles.md).

## Axioms (partner lens)

1. **A wrong name is a second universe.** Code that writes `workspaces.id` or `edgequake."Node"` while storage uses `workspace_id` / `eq_eq_default_graph` produces `42703` / `42P01` forever — Postgres is correct.
2. **Hourly monitors multiply bugs by fleet size.** One bad `EXISTS` per workspace table × 24 hours ≈ thousands of log lines without a single user request.
3. **An integrity alarm can be true while the monitor SQL is also buggy.** INV-03 on 0.22.0 is a real orphan set; fixing the monitor (dual-read) does not delete orphans.
4. **UNIQUE without idempotent write turns retries into errors.** Fresh UUID PK + same slug → `23505` under double-submit.

## Laws (cite SPEC-104)

| Law | Statement | Partner symptom |
|-----|-----------|-----------------|
| **LAW-I1** Schema-name SSOT | Inspector derives graph / PK names from the same helpers as storage | E1, E2 |
| **LAW-I2** Fail-visible monitors | Monitor errors and drift are visible; never silent green | E1 fail-open, E3 |
| **LAW-I3** Idempotent unique writes | Natural-key INSERT is get-or-create with HTTP 201/200/409 | E4 |
| **LAW-I4** Bounded lineage probes | Node counts use GIN + timeouts (not in this email; SPEC-104 #5) | — |

## SQLSTATE anchors ([PostgreSQL error codes](https://www.postgresql.org/docs/current/errcodes-appendix.html))

| Code | Condition | Partner |
|------|-----------|---------|
| `42703` | `undefined_column` | E1 |
| `42P01` | `undefined_table` | E2 |
| `23505` | `unique_violation` | E4 |

Apache AGE: each graph is a Postgres schema; label `"Node"` is a child table — joining a non-existent graph schema yields `42P01` ([AGE graphs](https://age.apache.org/age-manual/master/intro/graphs.html)).

## DRY / SOLID (already on HEAD)

```ascii
 LAW-I1 ─▶ InspectorConfig::for_namespace ─▶ age_graph_name_for_namespace
 LAW-I2 ─▶ INV-03 dual public.chunks | KV   ─▶ SPEC-107: LogOnly repair tip
 LAW-I3 ─▶ pg_create_tenant ON CONFLICT(slug) ─▶ Error::Conflict → HTTP 409
 SRP    ─▶ monitor reports; ops owns orphan requeue (no SAFE mutate)
```

## Root-cause classes

```ascii
 ┌────────────────────┐     ┌─────────────────────┐     ┌──────────────────┐
 │ SSOT violation     │     │ True SAGA residue   │     │ Non-idempotent   │
 │ (inspector names)  │────▶│ (INV-03 orphans)    │────▶│ tenant create    │
 │ E1 E2              │     │ E3                  │     │ E4               │
 └────────────────────┘     └─────────────────────┘     └──────────────────┘
        fix: ≥0.24.0              fix: upgrade + ops         fix: ≥0.24.0
```
