# SPEC-110 — Migration 118 wsdoc ON CONFLICT

> **Trigger:** Partner PPD `edgequake migrate --confirm-drop` on `ghcr.io/raphaelmansuy/edgequake:0.24.1` fails at migration **118** with Postgres `21000`: `ON CONFLICT DO UPDATE command cannot affect row a second time`.  
> **Method:** First principles (code is law) + Docker reproduce + checksum-aware patch + e2e gates.  
> **Broken through:** **v0.24.1** (and any binary embedding unpatched 118).  
> **Target cut:** **v0.24.2**.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Migration 118 backfills documents from wsdoc:{ws}:{doc} with                │
│  SELECT DISTINCT + ON CONFLICT (id) DO UPDATE.                               │
│  Same document_id under multiple workspaces → two proposed rows per id       │
│  → Postgres cardinality_violation (deterministic INSERT rule).               │
│  Fix: DISTINCT ON (doc_id) + ORDER BY doc_id, ws_id; harden 121 the same.    │
│  Ship: patch 118/121 in place + checksums.lock + M078-style checksum repair  │
│        + new GHCR image (append-only 143 cannot unblock stuck@117).          │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | 118 fails on multi-ws same `document_id` | **Closed (proven)** | [00-issue-data](00-issue-data.md), E2E-110-01 |
| F2 | `SELECT DISTINCT` does not protect conflict key | **Closed** | [01](01-first-principles.md); patched SQL |
| F3 | 121 shares structural risk | **Closed (hardened)** | [07](07-similar-issues.md), E2E-110-04 |
| F4 | Append-only migration cannot unblock stuck@117 | **Locked / honored** | In-place 118 edit |
| F5 | Already-applied old 118 needs checksum repair | **Implemented** | `m118.rs` / `m121.rs` (Path B DB UPDATE not e2e'd) |
| H1 | Patch release required (embedded SQL) | **Code ready; tag deferred** | CHANGELOG Unreleased → ship as **v0.24.2** |
| E2E | Docker/local repro + contract + repair gates | **Local proof green** | [measurements/SUMMARY.md](measurements/SUMMARY.md) |

## Document map

```ascii
 00-why / 00-issue-data
   → 01-first-principles (LAW-M1..M5)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 04-fix-plan
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-similar-issues
   → 08-partner-reply
   → 09-ops-runbook
   → measurements/
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Primary fix site | Edit [`118_spec091_wsdoc_backfill.sql`](../../edgequake/migrations/118_spec091_wsdoc_backfill.sql) in place |
| Sibling harden | Edit [`121_spec091_injection_backfill.sql`](../../edgequake/migrations/121_spec091_injection_backfill.sql) the same way |
| Dedup idiom | `SELECT DISTINCT ON (conflict_id) … ORDER BY conflict_id, ws_id` |
| Workspace pick | Lexicographic min `ws_id`; `COALESCE` only fills NULL scope (LAW-M5) |
| Lockfile | Update [`checksums.lock`](../../edgequake/migrations/checksums.lock) SHA-384 for 118 and 121 |
| Already-applied fleets | M078-style pre-sqlx checksum repair (DEV_MODE-gated; loud refuse otherwise) |
| Product cut | **v0.24.2** per [release-and-cd](../../docs/operations/release-and-cd.md) |
| Proof target | `make spec110-migrate-118-proof` |

## Partner one-liner

Oui — il faut une **nouvelle version** (cible **0.24.2**). L’image `0.24.1` embarque le SQL défectueux de la migration 118 ; corriger le repo sans republier l’image ne débloque pas PPD. Détail opérateur : [08-partner-reply.md](08-partner-reply.md) · [09-ops-runbook.md](09-ops-runbook.md).

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|-----------|
| [SPEC-091](../091-simplify-data-layer/) | wsdoc → `documents.workspace_id`; backfills 117–124; drop 125 |
| [SPEC-105](../105-fix-legacy/) | Legacy cutover assert 142 after confirm-drop |
| [SPEC-041](../041-fix-migration/) | L1 checksum repair pattern (M078) |
| [SPEC-083 X-02](../083-improvements/) | No silent checksum rewrite in production |
| [SPEC-106](../106-kg-persist-bug/) | Bug-fix pack template |
| [SPEC-107](../107-issue/) | Partner-facing reply template |
| [migrate NOTES](../../edgequake/migrations/NOTES.md) | Immutability rule (exception = LAW-M3 blocking bug) |
| [spec091-upgrade-from-v0.22.0](../../docs/operations/spec091-upgrade-from-v0.22.0.md) | Operator upgrade path |

## DRY rule

Conflict-key dedup law (LAW-M1/M2) and checksum-repair pattern (LAW-M3 / M078) are the SSOT. Do not invent a parallel migrate CLI, a one-off “skip 118” flag, or a second membership model. If this pack and NOTES.md disagree on editing applied migrations, **LAW-M3 wins for blocking field failures**; NOTES still forbids casual edits.

## Out of scope (v1)

- Multi-workspace membership join table (relational model stays one `workspace_id` per `documents.id`)
- Changing `--confirm-drop` / SAFE SCHEMA vs DROP OLD semantics
- Silent production checksum rewrite without `EDGEQUAKE_DEV_MODE`

## Start here

1. [00-why.md](00-why.md)  
2. [00-issue-data.md](00-issue-data.md)  
3. [01-first-principles.md](01-first-principles.md)  
4. [03-root-cause.md](03-root-cause.md)  
5. [04-fix-plan.md](04-fix-plan.md)  
6. [05-e2e-test-matrix.md](05-e2e-test-matrix.md)  
7. [09-ops-runbook.md](09-ops-runbook.md)
