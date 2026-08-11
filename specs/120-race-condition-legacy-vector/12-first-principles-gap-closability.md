# 12 — First Principles: Can We Close the Residual Gaps?

> Companion to [`11-honest-assessment.md`](11-honest-assessment.md).  
> Question: for each residual gap, is closure **necessary**, **possible without breaking LAW-120-2**, and **in-scope for SPEC-120**?

## Axiom split (do not confuse these products)

```ascii
  PRODUCT A — Live ingest (SPEC-120 / #374)
    Invariant: user document must not fail because of provenance bookkeeping
    Laws: LAW-120-1..3, LAW-120-6, LAW-120-7

  PRODUCT B — Identity completeness (SPEC-083)
    Invariant: one logical entity → one spine row (normalized)
    Laws: graph-identity cluster (normalize, merge, reconcile)

  PRODUCT C — Legacy fleet cutover (SPEC-111 / migration 131)
    Invariant: drop is honest — uncovered legacy rows must be visible
    Laws: LAW-111-2, LAW-C3 provenance coverage
```

**First Principles rule:** a gap is “closeable under SPEC-120” only if it is required for Product A **and** does not falsify Product B/C invariants.

---

## Causal diagram (why gaps remain)

```ascii
  WHY do residual gaps exist after P0 absorb?
    → P0 only arbitrates the SECOND unique index on fleet INSERT
  WHY not also merge aliases?
    → that is a different identity invariant (exact-name UNIQUE ≠ normalized UNIQUE)
  WHY not also soft-Ok the stamp job?
    → drop-GREEN would lie (Product C)
  WHY not HTTP e2e?
    → concurrency class already proven at merger; HTTP is soak cost, not necessity
```

---

## Gap-by-gap closability

### G1 — Alias spines (`JOHN_SMITH` vs `John Smith`)

| Question | Answer |
|----------|--------|
| Necessary for Product A (#374)? | **No** (LAW-120-5). Live sink already writes normalized bare names; UNIQUE is exact `(tenant, workspace, name)`. |
| Possible without breaking LAW-120-2? | **Yes** — merging aliases *reduces* dual-FK pressure. |
| Smallest correct close | (1) Spine/backfill always insert `normalize_entity_name(...)` never display; (2) merge migration for existing alias pairs; (3) only then UNIQUE on normalized key / expression ([Postgres unique checks](https://www.postgresql.org/docs/current/index-unique-checks.html) are atomic — expression UNIQUE is the durable arbiter, not check-then-insert). |
| Blast radius | `entities` schema, sink, AGE nodes, fleet resolve, Acc, backfill — SPEC-083 sized. |
| **Verdict** | **Can close — but not under SPEC-120.** Defer → SPEC-083. |

### G2 — Loser FK without typed embedding after absorb

| Question | Answer |
|----------|--------|
| Necessary for Product A? | **No.** ANN joins `entity_embeddings fe JOIN entities e ON e.id = fe.entity_id` — winner remains searchable; loser is ANN-invisible. Content path uses winner. |
| Possible without breaking LAW-120-2? | **Yes**, only by: merge loser→winner, **or** give loser an embedding with `legacy_vector_id = NULL` (usually wrong — second vector for one logical entity). Giving loser the **same** lid would violate LAW-120-2. |
| Smallest correct close | Prefer G1 merge; do not invent dual lids. |
| **Verdict** | **Can close only via identity merge (G1).** Accept as P0 residue until SPEC-083. |

### G3 — `fleet_provenance_stamp` fail-closed vs live absorb Ok

| Question | Answer |
|----------|--------|
| Necessary for Product A? | **No.** Different surface: migration/ops vs live ingest. |
| Unify outcomes without breaking Product C? | **No.** Stamp fail-closed keeps unstampable legacy rows **visible** so migration 131 / advisor cannot false-GREEN. Live absorb Ok means “skip losing **write**.” Soft-Ok stamp would mean “pretend coverage.” |
| Possible shared DRY? | Pure helper for “does this lid already belong to another FK?” — yes. Shared **success policy** — no. |
| **Verdict** | **Must not close by unification.** Keep split policies (LAW-120-2 ownership shared; outcomes diverge by product). |

### G4 — Historical unstamped / migration 131 readiness

| Question | Answer |
|----------|--------|
| Necessary for Product A? | **No.** Live ingest does not require `uncovered_fleet_rows == 0`. |
| Possible without breaking LAW-120-2? | **Yes** via ops: stamp stampable rows; resolve stalls (merge aliases or deliberate legacy delete); then `--confirm-drop`. |
| Smallest correct close | Ops runbook (SPEC-111), not SPEC-120 code. |
| **Verdict** | **Closeable as cutover work — defer SPEC-111.** |

### G5 — HTTP upload + worker dual-doc e2e

| Question | Answer |
|----------|--------|
| Necessary for Product A laws? | **No.** LAW-120-7 requires concurrency proof; merger path already hits sink → vectors → `mirror_legacy_batch` under typed authority. |
| Possible? | **Yes** — reuse AppState/worker harnesses (`e2e_spec118_*`, upload perf). Cost: flakes, LLM/mock, minutes. |
| Incremental signal vs merger e2e? | Low for this defect class (same mirror SQL). Higher for “whole product soak.” |
| **Verdict** | **Can add as soak — not required to close #374.** Defer nightly. |

---

## Closability matrix

| Gap | Product | Closeable? | Under SPEC-120? | Action |
|-----|---------|------------|-----------------|--------|
| G1 Alias merge | B | Yes | **No** | SPEC-083 |
| G2 Loser embedding | B (via G1) | Yes via merge | **No** | Accept residue |
| G3 Unify stamp+live | C vs A | Outcomes: **No** | **Never** | Keep split |
| G4 Historical / 131 | C | Yes (ops) | **No** | SPEC-111 |
| G5 HTTP dual-doc e2e | A (test) | Yes | Optional | Soak only |

```ascii
  SPEC-120 CLOSE BOX
  ┌─────────────────────────────────────┐
  │  LAW-120-1 Bookkeeping ≠ content    │  DONE (absorb)
  │  LAW-120-2 One lid owner            │  DONE (unique kept)
  │  LAW-120-3 Arbiter completeness     │  DONE (targetless DO NOTHING)
  │  LAW-120-7 Concurrency proof        │  DONE (contract + mirror + merger)
  └─────────────────────────────────────┘
           │
           │  NOT in the box
           ▼
  G1/G2 → SPEC-083     G3 keep split     G4 → SPEC-111     G5 soak
```

## Normative conclusion

1. **SPEC-120 is First-Principles-complete for Product A** once merged (P0 absorb + proofs). Residual “3/10 durable identity” is **not** a SPEC-120 blocker — it is Product B debt correctly fenced by LAW-120-5.
2. **Do not** close G3 by making stamp soft-Ok — that would break cutover honesty ([Postgres uniqueness is the arbiter](https://www.postgresql.org/docs/current/index-unique-checks.html); coverage must reflect reality).
3. **Do** close G1 eventually under SPEC-083 with normalize-at-write + merge + then UNIQUE on the normalized key (expression index / stored column) — the race-safe pattern, not check-then-insert.
4. Ship gate remains: land code + close #374; park identity/cutover under their specs.
