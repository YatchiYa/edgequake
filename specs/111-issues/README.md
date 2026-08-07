# SPEC-111 — Partner issues #360–#364 (cross-ref pack)

> **Trigger:** Partner bugs #360/#366 Clear All, #361 bulk upload, #362 KV-residue advisor, #363 iw2 join miss, #364 vector drop readiness.  
> **Method:** First principles — **code is law** — source proof on current HEAD + last published tag **v0.24.1**; no speculative RCAs.  
> **Audience:** Engineering (fix train) + partners (honest status on each thread).  
> **Ship vehicle (proposed):** **v0.24.2** for Cluster A (#362–364) + Clear All LAW-111-9 (#366/#360); #361 measure-only.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  CLUSTER A — SPEC-091 migrate advisor / backfill                             │
│    #362  FIXED on HEAD — residue/125 cast → uuid (Index Cond)                │
│    #363  FIXED on HEAD — iw2 normalize join + honest failed_count            │
│    #364  FIXED on HEAD — retirable = uncovered==0; fleet = provenance-only   │
│                                                                              │
│  CLUSTER B — Documents UX                                                    │
│    #366/#360  FIXED on HEAD — LAW-111-9 authoritative empty + wipe KV purge  │
│    #361  OUT OF SCOPE — capacity/LLM; measure only (no concurrency code)     │
│                                                                              │
│  RELEASE — v0.24.2 = ship-with-runbook (not blind upgrade)                   │
│    Also: SPEC-091/098 fleet-mirror parse for `->` in entity names            │
│    Gates: measurements/e2e111-release-safety-gates.txt                       │
│    Ops: 09-ops-runbook.md + docs/operations/upgrade-to-0.24.2.md             │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| Issue | Title (short) | Present on HEAD? | Present on v0.24.1? | Severity | Fix needed? |
|-------|---------------|------------------|---------------------|----------|-------------|
| [#364](https://github.com/raphaelmansuy/edgequake/issues/364) | Vector drop readiness = empty legacy | **Fixed on HEAD** (coverage/`uncovered_*`) | **Yes** (bug) | P0 UX / gate drift | **Done** — advisor ≡ 126/131 coverage |
| [#363](https://github.com/raphaelmansuy/edgequake/issues/363) | iw2 join miss + false success | **Fixed on HEAD** (normalize + `failed_count`) | **Yes** (bug) | P0 data loss risk pre-drop | **Done** — normalize join + provenance |
| [#362](https://github.com/raphaelmansuy/edgequake/issues/362) | KV residue `::text` cast timeout | **Fixed on HEAD** (cast → uuid) | **Yes** (bug) | P1 migrate blocked | **Done** — residue + 125 + wave_d |
| [#366](https://github.com/raphaelmansuy/edgequake/issues/366) | Clear All leaves docs (0.24.1) | **Fixed on HEAD** (LAW-111-9) | **Yes** (bug) | P1 UX | **Done** — authoritative empty + KV purge |
| [#361](https://github.com/raphaelmansuy/edgequake/issues/361) | Bulk upload slow | Expected load | Older report | P3 / capacity | Measure only (no concurrency code) |
| [#360](https://github.com/raphaelmansuy/edgequake/issues/360) | Clear All leaves docs | Same as #366 | **Yes** (clarified) | P1 UX | **Done** — duplicate of #366 |

**Ship vehicle:** Unreleased / **v0.24.2** (with SPEC-110).

**Residual harden (LAW-C3):** fleet drop = **provenance-only** (`legacy_vector_id`); see [`09-ops-runbook.md`](09-ops-runbook.md) and [`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md).

## Document map

```ascii
 00-why / 00-issue-data
   → 01-first-principles (LAW-111-*)
   → 02-cross-ref-matrix
   → 03-root-cause (per issue)
   → 04-fix-plan (DRY / SOLID)
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-cluster-notes
   → 08-partner-comments (posted to GH)
   → 09-ops-runbook
   → 10-migration-immutability (LAW-MIG — never edit applied SQL)
   → 11-release-partner-notes (v0.24.2 partner cutover)
   → issue-360 / issue-366 … issue-364 (deep dives)
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Truth source | Current tree under `edgequake/crates/edgequake-storage/src/migration_engine/**` + migrations 125/126/131 |
| #364 readiness predicate | **Coverage** (legacy row has typed counterpart), **not** `COUNT(*) == 0` on pre-drop table |
| #363 join | Reuse `normalize_entity_name` (single SSOT); count unresolved as `failed` / coverage shortfall |
| #362 cast | Cast substring → `::uuid`; keep indexed column bare — mirror chunk predicates already correct in same SQL |
| #362 DRY | Patch **both** `residue.rs` advisor SQL **and** migration `125` guard (LAW-C3 parity) |
| #366 / #360 | LAW-111-9: authoritative empty membership terminal for reads; wipe purges residual KV list surfaces; e2e plants raw KV ghosts |
| #361 | Treat as SPEC-090 / capacity; require measurement before code change |
| Confirm-drop | Physical safety stays in SQL guards; advisor must not lie about readiness |

## Start here

1. [00-why.md](00-why.md)  
2. [03-root-cause.md](03-root-cause.md)  
3. [04-fix-plan.md](04-fix-plan.md)  
4. [05-e2e-test-matrix.md](05-e2e-test-matrix.md)  
5. Per-issue: [issue-366](issue-366-clear-all.md) · [issue-360](issue-360-clear-all.md) · [issue-364](issue-364-vector-retirable.md) · [issue-363](issue-363-iw2-join.md) · [issue-362](issue-362-kv-cast.md) · [issue-361](issue-361-bulk-upload.md)  
6. Partner cutover: [11-release-partner-notes.md](11-release-partner-notes.md) · [09-ops-runbook.md](09-ops-runbook.md) · [measurements/BRUTAL-HONESTY.md](measurements/BRUTAL-HONESTY.md)

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-091](../091-simplify-data-layer/) | KV/vector cutover, migrate console, drops 125/126/131 |
| [SPEC-110](../110-migration-issue/) | Partner migrate train; v0.24.2 vehicle |
| [SPEC-105](../105-fix-legacy/) | Post-drop assert 142 |
| [SPEC-090](../090-performance/) | Ingest throughput / pool contention (#361) |
| [SPEC-050](../050-pipeline-and-delete/) | Bulk delete progress / Clear All |
| Issue #309 / CHANGELOG durable wipe | Supersedes much of #360 on ≥ mid-0.2x |

## Out of scope (this pack as living docs)

- Re-running partner production DBs
- Auto `--confirm-drop` / auto dual-legacy residue delete
- #361 concurrency code (measure-only)
- Full SPEC-120 mark-and-supersede delete saga (cancel/purge soft-fail is scoped)
