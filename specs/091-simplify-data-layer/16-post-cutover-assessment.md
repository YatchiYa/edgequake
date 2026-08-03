# 16 — Post-cutover assessment (HEAD after Waves A–D)

> **As-of:** working tree after SPEC-091 Waves A–D + migration console (`dry-run` / `--confirm-drop`) + upgrade soak + **IW0–IW5** (see [19](19-improvement-plan.md) for the six-criteria closure program; this file remains the A–D wave audit — do not treat it as post-IW HEAD alone).
> **Product pin (published):** still **v0.22.0** — schema ≤ migration **105**, KV SSOT, GHCR `edgequake:0.22.0`.
> **This document is HEAD truth.** Pin-era assessments ([00](00-raw-needs.md), [02](02-first-principles.md) “Today”, [03](03-assessment.md)) remain frozen at the release tag; do not rewrite them — use this file for “what the data layer is now.”

---

## 1. Verdict (honest)

| Layer | Reality |
| --- | --- |
| **Published product** | v0.22.0 / migrations ≤105 / generic KV SSOT |
| **Working tree (HEAD)** | Waves **A–D** landed: typed SSOT for chunk text, dedup, quarantine, wsdoc, artifacts, checkpoints, injection, document shells, LLM cache; write-stop + `42P01` degrade; migration **125** drops `eq_*_kv`; console + **`migrate dry-run`** + `make spec091-upgrade-soak` GREEN |
| **Not done** | **W3** chunk-embedding cutover **implemented** but default still `legacy_tables` pending parity gate + flip; **W4 chunk-vector retirement implemented** (fleet backfill+verify, guarded migration 126, runtime-DDL retired for chunk-dedicated tables) but the physical drop stays human-gated behind `--confirm-drop`; entity/rel/report vectors still `eq_*_vectors` + **runtime DDL**; **W4** fence default off / 1M residue proofs open; **W5** scale stubs; many call sites still speak KV APIs that route under relational flags |
| **Release** | Unreleased until a version bump/tag ships migrations **106–125**. Confirm-drop is a **backup-gated contract**, not “data layer finished” |

Expand/contract practice (2026): EdgeQuake collapsed expand + family backfill + KV contract into **one unreleased train** for KV families. That is acceptable for a pinned upgrade with write-stop replicas + dry-run/guard, but it is **not** the multi-release textbook path still required for the remaining **vector** cutover (W3).

```ascii
 v0.22.0 pin (≤105, KV) ──▶ HEAD A–D (106–125 typed + drop)
                              │
                              ├─▶ ops: dry-run → confirm-drop → soak
                              └─▶ residual: W3 embeddings · W4 fence scale · W5 partition
```

---

## 2. Wave taxonomy map (A–D ↔ W0–W5)

Informal execution waves **A–D** (CHANGELOG / risk register) implemented the **KV-retirement slice** of the planned W0–W2 program as a single HEAD train. They do **not** close W3–W5.

| Informal | Planned wave(s) | What landed in HEAD |
| --- | --- | --- |
| **A** | W0 / W1 foundation | Migration engine ledger (106); typed tables (107); chunk authority + relational writer + dual-read flags |
| **B** | W1 / W2 families | Dedup / quarantine / shells / wsdoc / artifacts / checkpoints / injection cutovers + SQL backfills 117–122 |
| **C** | W2 + console | Family flags write-stop; advisor/console C0–C3; job control (123); LLM cache (124) |
| **D** | W2 contract | Migration **125** verified purge + durable-row guard + `DROP eq_*_kv`; 42P01 / fence JOIN / mm-asset fixes (R-27..R-30) |
| — | **W3** | Schema (108) **+ cutover now wired**: `VectorBackend` flag, `PgChunkEmbeddingIndex` port impl, ingest dual-write, query dual-read + fallback counter, engine `w3-chunk-embedding-backfill` + verify, console VECTOR posture — **chunks only**; default still `legacy_tables` |
| — | **W4 (vectors)** | **Chunk-vector retirement now wired**: fleet-wide engine backfill+verify across every `eq_%_vectors` relation, `VectorPosture.retirable` + gated `drop vector-legacy` advisor action, console `ReadyToRetire` + `--confirm-drop` surface, guarded migration **126** (coverage guard → delete covered chunk rows → drop chunk-dedicated tables, keep entity/rel/report vectors), runtime-DDL retired for chunk-dedicated tables (LD-03), boot-time `chunk_embeddings` validation — **chunks only**; physical drop human-gated behind `--confirm-drop` |
| — | **W4 (fence/scale)** | Fence module + `chunk_serving_state` / `outbox_events` (109); default **off**; scale proofs open |
| — | **W5** | `scale_gates.rs` stubs only |

SPEC-120 task migrations **112–115** ride the same 106–125 train (orphan WIP for `/operations` — see `specs/92-task-system/`). Ops blast radius is broader than “just KV drop.”

---

## 3. SSOT map post-125 (by family)

After migration **125** on a HEAD-migrated database, durable identity for retired KV families lives in typed tables. **`eq_*_kv` is gone.**

| Family / concern                        | Typed SSOT                                                                              | Migration                         | Notes                                                                                                                                   |
| -----------------------------------------| -----------------------------------------------------------------------------------------| -----------------------------------| -----------------------------------------------------------------------------------------------------------------------------------------|
| **CHUNK** text                          | `public.chunks` (+ `chunk_serving_state`)                                               | writer + 109 fence                | Authority: `EDGEQUAKE_CHUNK_TEXT_AUTHORITY` (default `relational`)                                                                      |
| **DOC_HASH / STAGING_HASH**             | `public.ingestion_dedup`                                                                | 107 create, 117 backfill          | Runtime routes **both** `doc:hash:` and `staging:hash:` through **DOC_HASH** — `EDGEQUAKE_KV_FAMILY_STAGING_HASH` is largely ornamental |
| **COMPENSATION_QUARANTINE**             | `public.compensation_quarantine`                                                        | 107                               | Transient; drain in `compensation_drain.rs`                                                                                             |
| **METADATA / content / staging shells** | `public.documents`                                                                      | 122 backfill; `document_shell.rs` | `_shell: "staging"` marker; columns dual-written when FK-safe (R-21)                                                                    |
| **WSDOC**                               | `documents.workspace_id` (+ JSONB fallback)                                             | 118                               | Membership index retired                                                                                                                |
| **ARTIFACT** (lineage / MM)             | `public.document_artifacts`                                                             | 116 + 119                         |                                                                                                                                         |
| **CHECKPOINT**                          | `public.pipeline_checkpoints`                                                           | 116; drain in 124                 | Transient by design                                                                                                                     |
| **INJECTION**                           | `public.documents` (`metadata.source_type=injection`)                                   | 121                               |                                                                                                                                         |
| **CACHE**                               | `public.llm_cache`                                                                      | 124                               | Recomputable; no KV backfill                                                                                                            |
| **AUTH** KV                             | purged                                                                                  | 120                               | Identity already PG-native                                                                                                              |
| **Migration engine**                    | `edgequake.edgequake_migration_job/_batch`, view `migration_progress`                   | 106; `cancelled` in 123           |                                                                                                                                         |
| **Serving fence / outbox**              | `chunk_serving_state`, `outbox_events`                                                  | 109                               | Fence default **off**                                                                                                                   |
| **W3 chunk embeddings**                 | `embedding_models`, `chunk_embeddings` (`halfvec(1536)`), `edgequake_schema_generation` | 108 + dual-write/read + backfill  | `PgChunkEmbeddingIndex` behind `EmbeddingIndex` port; `EDGEQUAKE_VECTOR_BACKEND` (default `legacy_tables`); dual-write on ingest, dual-read w/ fallback; engine backfill `w3-chunk-embedding-backfill` + `verify_chunk_embedding_backfill`; console VECTOR posture |
| **Provider budget (QW1)**               | `edgequake.provider_slot/_budget`, view `provider_inflight`                             | 110                               |                                                                                                                                         |
| **Live embeddings (still)**             | `public.eq_*_vectors` (+ stats / HNSW)                                                  | pre-091 + runtime DDL             | **W3 gap** — violates LD-03 for this family                                                                                             |

---

## 4. Database operations (who writes / when)

| Path | Behavior on HEAD |
| --- | --- |
| **Ingestion persist** | Relational chunk writer (`PostgresChunkRepository` / `relational_chunk_writer`) from `ingestion_persister`; KV chunk dual-write filtered by authority/family flags |
| **Shell / staging / admission** | Typed `document_shell` / `ingestion_dedup`; many API sites still call `kv.upsert` — adapter **write-stops** relational families and treats post-drop `42P01` as source-gone |
| **LLM cache** | `llm_cache` table when family relational |
| **Boot** | **LD-15: boot never applies versioned schema** — fail-closed verify, exit 78 + dry-run/migrate hint on pending, downgrade protection; `EDGEQUAKE_ALLOW_BOOT_MIGRATE` removed (warn-and-ignore shim); `make dev` runs `edgequake migrate` visibly first ([17-boot-migration-gating.md](17-boot-migration-gating.md)). Data movement **never** at boot (LD-08) |
| **Migration engine** | Default `EDGEQUAKE_MIGRATION_MODE=verify` (register + verify). `automatic` runs leased chunk-text backfill. Family SQL backfills **117–122** are **migration-time**, not engine jobs |
| **Vector path** | Chunk vectors: dual-write (legacy `eq_*_vectors` + typed `chunk_embeddings`) on ingest; query dual-read gated by `EDGEQUAKE_VECTOR_BACKEND` with logged fallback counter. **W4 retirement**: guarded migration 126 deletes covered chunk rows + drops chunk-dedicated tables behind `--confirm-drop`; runtime `create_table` skips chunk-dedicated legacy tables once retired (LD-03 honored for chunks). Entity/rel/report vectors unchanged — still on legacy `eq_*_vectors` + runtime DDL (LD-03 still **not** met for non-chunk embeddings, out of scope) |
| **Serving fence** | Opt-in JOIN on `public.chunk_serving_state` (R-28); keep off until query proof |

---

## 5. Operator surface maturity

| Surface | Maturity | Notes |
| --- | --- | --- |
| `edgequake migrate dry-run` | Strong | Preview pending + posture + checklist; exit 0 even when drop-readiness is RED; **no** schema advance |
| `migrate` (no confirm) | Strong | Applies expandable pending; **refuses 125** with guard + dry-run hint |
| `migrate --confirm-drop` | Strong | Irreversible gate; abort leaves DB pre-drop if guard fails; stdout: per-version applied + KV-drop message |
| `migrate console` / `plan` / `guard` / `family` | Good | Schema-derived advisor (LD-14); write verbs LD-07-gated; **VECTOR** row shows backend flag, backfill job state, verify result, legacy-vs-typed chunk row counts |
| Job pause/resume/cancel | Present | Needs ledger rows; meaningful when engine is `automatic` |
| `make spec091-upgrade-soak` | Good (synthetic) | GHCR 0.22.0 → dry-run assert → confirm-drop tee → HTTP/SQL gates |
| Ops runbook | Honest | [docs/operations/spec091-upgrade-from-v0.22.0.md](../../docs/operations/spec091-upgrade-from-v0.22.0.md) |

**Dry-run ≠ readiness.** GREEN drop-readiness still requires relational flags, zero durable residue, and completed backfills on large corpora.

---

## 6. Law / locked-decision report card (HEAD)

| ID | Grade | Notes |
| --- | --- | --- |
| **LAW-D5 / LD-03** | Partial | **Honored for KV** (no recreate after 125) **and for chunk vectors** (runtime `create_table` skips chunk-dedicated tables once migration 126 retires them). **Violated for entity/rel/report vectors** (runtime DDL remains, out of W4 scope) |
| **LAW-D6 / LD-01** | Mostly honored | Chunk text authority defaults relational; pin-era empty-`chunks` defect fixed in-tree |
| **LD-02** | Partial | Relational chunks use UUID; vector PK still TEXT `content_ref` world until W3 |
| **LD-05** | Partial | Domain ports exist for chunks; many callers still KV-shaped and rely on adapter routing |
| **LD-07** | Honored (KV drop) | One irreversible op (125) behind `--confirm-drop` / env; dry-run + soak |
| **LD-08** | Honored | Engine descriptors; boot does not move data |
| **LD-09** | Partial | Fence module exists; default **off**; W3 FKs not enforced → fence remains advisory at scale |
| **LD-14** | Honored | Console/advisor schema-derived; illegal `kv` post-drop refused |
| **LD-06 / W3** | Partial (chunks) | Chunk vectors have a typed index path (`PgChunkEmbeddingIndex`); entity/rel/report fleet + single HNSW policy not converged |

---

## 7. Residuals and open risks (ops-relevant)

| ID | Status | Residual |
| --- | --- | --- |
| R-23 / R-26 / R-30 | Mitigated in-tree | Drop is restore-only; verified purge + typed SSOT guard; soak exercises confirm-drop |
| R-27 | Residual-ops | Roll **all** replicas to write-stop binary before/with 125 |
| R-28 | Mitigated | Fence JOIN must use `public.chunk_serving_state` |
| R-29 | Mitigated | mm-asset existence via typed `documents`; KV reads 42P01-tolerant |
| R-01..R-04 | Historical for fresh HEAD installs; still relevant on v0.22.0→HEAD upgrades with large KV corpora | |
| **W3** | Mitigated (chunks) | `EDGEQUAKE_VECTOR_BACKEND` implemented; chunk embeddings cut over behind flag with dual-write/read + backfill + verify + console posture + recall parity e2e. Flip to `chunk_embeddings` default is operator-gated after parity |
| **W4 (vector retire)** | Mitigated (chunks) | Fleet-wide engine backfill+verify, gated `drop vector-legacy` advisor action, guarded migration **126** (chunk-only), runtime-DDL retirement for chunk-dedicated tables (LD-03), boot-time `chunk_embeddings` validation. Physical drop stays human-gated behind `--confirm-drop`; entity/rel/report vectors remain open |
| **W4 / W5** | Open | 1M delete residue / partitioning evidence gates |

---

## 8. What “done” means

| Claim | Meaning |
| --- | --- |
| **KV-retirement DoD** | Waves A–D infrastructure + soak GREEN → **met in-tree, unreleased** |
| **Spec-complete DoD** | W3–W5 exit gates + full test matrix in [11](11-e2e-test-matrix.md) → **not met** |
| **Published product** | Tag ships 106–125 → **not yet** |

Do not read “DoD checkboxes green” in [06](06-implementation-plan.md) as “ship SPEC-091 complete.” Read them as **KV retirement ready for a release tag**, with W3–W5 residual explicit.

---

## 9. Evidence commands (exists today)

See [11 — Existing vs planned](11-e2e-test-matrix.md#exists-today-run-these) and [README Verification](README.md#verification). Primary proof:

```bash
cargo test -p edgequake-storage --features postgres --test e2e_spec091_wave_d
cargo test -p edgequake-storage --features postgres --test e2e_spec091_console
cargo test -p edgequake --features postgres --test cli_migrate_console
make spec091-upgrade-soak

# W3 chunk-embedding cutover
cargo test -p edgequake-storage --features postgres --test e2e_spec091_chunk_embeddings
cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_backfill
cargo test -p edgequake-storage --features postgres --test e2e_spec091_recall_parity
cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_backend_dual

# W4 chunk-vector retirement (fleet backfill → retirable → guarded drop 126)
cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_retire
```

---

## Related

- Ops: [spec091-upgrade-from-v0.22.0.md](../../docs/operations/spec091-upgrade-from-v0.22.0.md)
- Risks: [09-risk-register.md](09-risk-register.md) (R-21..R-30)
- Console: [15-migration-console-cli.md](15-migration-console-cli.md) (§7.0 dry-run)
- Pin study (immutable): [00-raw-needs.md](00-raw-needs.md) · [03-assessment.md](03-assessment.md)
