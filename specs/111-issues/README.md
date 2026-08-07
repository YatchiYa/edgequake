# SPEC-111 — Partner issues #360–#364 (cross-ref pack)

> **Trigger:** Partner bugs #360/#366 Clear All, #361 bulk upload, #362 KV-residue advisor, #363 iw2 join miss, #364 vector drop readiness.  
> **Method:** First principles — **code is law** — source proof on current HEAD + last published tag **v0.24.1**; no speculative RCAs.  
> **Audience:** Engineering (fix train) + partners (honest status on each thread).  
> **Ship vehicle:** **v0.24.2 shipped** (Cluster A #362–364 + Clear All LAW-111-9 #366/#360); #361 measure-only. Follow-on pool harden is **v0.24.3** ([SPEC-112](../112-connection-pool/)), not Cluster A.

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
│  RELEASE — v0.24.2 SHIPPED (ship-with-runbook; not blind upgrade)            │
│    Also: SPEC-091/098 fleet-mirror parse for `->` in entity names            │
│    Gates: measurements/e2e111-v0242-publish-verify.txt                       │
│    Ops: 09-ops-runbook.md + docs/operations/upgrade-to-0.24.2.md             │
│  FOLLOW-ON — v0.24.3 = SPEC-112 connection-pool (identity/budget/close)      │
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

**Ship vehicle:** **Shipped on v0.24.2** (with SPEC-110). Pool / shared-PG harden ships on **v0.24.3** ([SPEC-112](../112-connection-pool/)).

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

## CHANGELOG

### [0.24.2] — 2026-08-07

**Shipped** (GH Release + multi-arch GHCR). Cluster A (#362–364) + Clear All LAW-111-9 (#366/#360) + SPEC-110 migrate 118/121 + SPEC-109 reasoning effort + fleet-mirror `->` parse. Ship-with-runbook — not blind upgrade.

| Proof | Link |
|-------|------|
| Partner cutover | [`11-release-partner-notes.md`](11-release-partner-notes.md) |
| Ops runbook | [`09-ops-runbook.md`](09-ops-runbook.md) |
| Upgrade guide | [`docs/operations/upgrade-to-0.24.2.md`](../../docs/operations/upgrade-to-0.24.2.md) |
| Release-safety gates | [`measurements/e2e111-release-safety-gates.txt`](measurements/e2e111-release-safety-gates.txt) |
| GHCR + Acc verify | [`measurements/e2e111-v0242-publish-verify.txt`](measurements/e2e111-v0242-publish-verify.txt) |
| CD workflow log | [`measurements/e2e111-v0242-ghcr-run.txt`](measurements/e2e111-v0242-ghcr-run.txt) |
| Root CHANGELOG | [`CHANGELOG.md`](../../CHANGELOG.md) `[0.24.2]` |

**GHCR (verified multi-arch `linux/amd64` + `linux/arm64`):**

- `ghcr.io/raphaelmansuy/edgequake:0.24.2` — index digest `sha256:678d6c8e1f18274585d1d3018550aa161ae0f9874392bff7b8ef99dc65c1d17c`
- `ghcr.io/raphaelmansuy/edgequake-frontend:0.24.2` — index digest `sha256:796ba2b99634402c132b4936e004921329be0381bb8690e2b948846fe946f80b`
- `ghcr.io/raphaelmansuy/edgequake-postgres:0.24.2` (+ `-pg16` / `-pg17` / `-pg18`) — index digest `sha256:b5b1678cdb03a875d87a8eccedc7432f0df93f90b9bc91421013f7f95290cc78`

Release: <https://github.com/raphaelmansuy/edgequake/releases/tag/v0.24.2>

### [0.24.3] — 2026-08-07

**Shipped** (GH Release + multi-arch GHCR). SPEC-112 connection-pool identity / budget / graceful close + UTF-8 truncate SSOT. Not a Cluster A re-ship — partners on shared PG should upgrade for attribution + slot budget.

| Proof | Link |
|-------|------|
| SPEC-112 pack | [`../112-connection-pool/`](../112-connection-pool/) |
| Upgrade guide | [`docs/operations/upgrade-to-0.24.3.md`](../../docs/operations/upgrade-to-0.24.3.md) |
| Multi-pool gates | [`../112-connection-pool/measurements/e2e112-gates.txt`](../112-connection-pool/measurements/e2e112-gates.txt) |
| Acc attest | [`../112-connection-pool/measurements/e2e112-acc-attest.txt`](../112-connection-pool/measurements/e2e112-acc-attest.txt) |
| GHCR verify | [`measurements/e2e111-v0243-publish-verify.txt`](measurements/e2e111-v0243-publish-verify.txt) |
| Root CHANGELOG | [`CHANGELOG.md`](../../CHANGELOG.md) `[0.24.3]` |

**GHCR (verified multi-arch `linux/amd64` + `linux/arm64`):**

- `ghcr.io/raphaelmansuy/edgequake:0.24.3` — index digest `sha256:b4f8f40d8398eb6e4884a684fb604f5d85e2daeec392667a1f33c6bfe7005e24`
- `ghcr.io/raphaelmansuy/edgequake-frontend:0.24.3` — index digest `sha256:621ea204de9e1d153a568400808254031887ab014df280a9b610bb13274a2a23`
- `ghcr.io/raphaelmansuy/edgequake-postgres:0.24.3` (+ `-pg16` / `-pg17` / `-pg18`) — index digest `sha256:f4a6b1542c5251dedfc857e2b59b74a219223f97898bed601c30eef0be055992`

Release: <https://github.com/raphaelmansuy/edgequake/releases/tag/v0.24.3>  
CD: <https://github.com/raphaelmansuy/edgequake/actions/runs/31192856924>
