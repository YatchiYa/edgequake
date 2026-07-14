# 021 — Lineage First Principles (Entity → Chunk → Document → Page)

**Status:** IMPLEMENTED (L-A1–L-A4 + e2e; L-B* deferred)  
**Re-assessment:** [022](./022-reassessment-2026-07-11.md) — lineage smoke Acc **0.427** (no re-ingest); next L-B2 telemetry  
**Peers:** [019](./019-query-first-principles-improvement-plan.md) · [020](./020-post-q1-first-principles-improvement-plan.md) · SPEC-031 · SPEC-032 · SPEC-045  
**Canvas:** [spec047-lineage-first-principles](/Users/raphaelmansuy/.cursor/projects/Users-raphaelmansuy-Github-03-working-edgequake/canvases/spec047-lineage-first-principles.canvas.tsx)  
**Research:** [TrustGraph explainability](https://docs.trustgraph.ai/overview/explainability.html) (PROV chain Document→Page→Chunk→Edge); W3C PROV-O

---

## 0. Direct answers

### Are you sure each Entity / Relation has lineage to Chunk → Document → Page?

**Today: only partially.**

| Link | Entity | Relationship | Chunk |
|------|--------|--------------|-------|
| → Chunk | **Yes** — `source_chunk_ids[]` merged across docs (capped KEEP≤200) | **Partial** — `source_chunk_ids[]` + singular `source_chunk_id` = first | **Self** (`id`) |
| → Document | **Weak** — singular `source_document_id` often **unset**; plural `source_document_ids[]` **never written** at ingest | Same weakness | **Strong** — `document_id` / metadata |
| → Page | **No** on entity/rel | **No** | **Yes** — `page_start` / `page_end` |

**Law:** Page is a property of a **chunk** (or page asset), not of an abstract entity. Correct chain is:

```text
Entity / Rel  ──mentions──▶  Chunk(s)  ──belongs──▶  Document
                                  │
                                  └──located──▶  Page(s)
```

You resolve page **through** chunk ids, never by inventing `entity.page`.

### If an entity belongs to several documents, must we keep the lineage?

**Yes — keep the union. Never first-doc-wins for scope.**

| What | Why |
|------|-----|
| Keep **all** `source_chunk_ids` (capped) | Citations + KG→chunk pick |
| Keep **all** `source_document_ids` (union) | Document-scoped query must see the entity when **any** of its docs is in scope |
| Do **not** collapse to singular `source_document_id` as the only truth | Causes silent drop or silent leak under `document_scope` |
| KEEP/FIFO caps | Bound memory/latency; prefer **doc-diverse** sample when capping (quality) |

One global entity **node** per normalized name is correct for GraphRAG. Multi-doc identity is expressed as **multi-parent lineage**, not duplicate nodes.

### How to use lineage in Query (first principles)

```text
1. RETRIEVE candidates (vector/graph) — may over-fetch
2. SCOPE by lineage — keep entity/rel iff lineage intersects allowed docs
     • Prefer source_document_ids[]
     • Else derive docs from source_chunk_ids prefixes / chunk_entity_links
     • Never "keep if unknown" when chunk ids imply foreign docs
3. GROUND via chunks — kg_chunk_pick only among chunk ids in scope
4. RESOLVE page — from retrieved chunks' page_start (entity cites [N] → page)
5. PROMPT — entities may show doc/chunk counts; answers cite chunk [N] + page=
6. FAIL-OPEN empty graph — no chunks after pick ⇒ prune entity/rel (020 A3)
```

**Quality:** answers cite real pages; no cross-doc leakage under scope.  
**Reliability:** scope filter is deterministic from stored lineage.  
**Speed:** filter early (SQL / chunk-id prefix) so local arm does not expand the whole graph then discard.

---

## 1. Master axioms

| ID | Principle | Operational meaning |
|----|-----------|---------------------|
| **L1** | Provenance is a chain, not a sticker | Entity → chunks → doc → page; page only via chunk |
| **L2** | Multi-doc entities keep a **set** of parents | Union of docs + chunks; never first-wins for filtering |
| **L3** | Scope is lineage intersection | `allowed_docs ∩ entity.docs ≠ ∅` |
| **L4** | Unknown provenance is unsafe under scope | Derive from chunk ids or **drop**; do not keep orphans |
| **L5** | Generation cites chunks, not entities | LLM sees `page=` on chunks; entities are navigation aids |
| **L6** | Caps must preserve diversity | When KEEP saturates, retain ≥1 chunk per contributing doc when possible |
| **L7** | Speed = filter before expand | Doc/chunk predicates in retrieval SQL before neighborhood walk |

**Corollary (020 + L):** Empty-arm prune and factual graph tax are Gen hygiene; **lineage completeness** is Scope hygiene. Both required for Acc under `document_scope`.

---

## 2. Current EdgeQuake physics (code is law)

```text
Ingest today
  Chunk ──stamp──▶ entity.source_chunk_ids  ──merge──▶ AGE node (multi-doc OK)
  entity.source_document_id  ──first-wins / often unset──▶ weak
  entity.source_document_ids[]  ──NEVER written──▶ query type expects it (SPEC-031)
  page  ──only on chunk vectors──▶ prompt page=

Query today
  Tier-1: vector MetadataFilter on document_id | source_document_id
  Tier-2: filter_context_by_document_ids
           chunks: strict
           entity/rel: plural → singular → KEEP IF EMPTY  ← leak / miss
  kg_chunk_pick: uses source_chunk_ids; no doc filter on ID list before fetch
  format_entity_line: no page / no doc list
```

**Symptom class:** under bench `document_scope`, local arm can inject entities whose **only** real parents are other PDFs (chunk ids present, doc fields empty → lenient keep), or hide entities whose first `source_document_id` is wrong while chunks from scoped PDF exist.

---

## 3. Target model (minimal, DRY)

### Ingest (write path)

1. Always stamp `source_document_id` **and** accumulate `source_document_ids[]` on entity/rel merge (union).  
2. Always stamp chunk ids (already).  
3. Optional denormalized `source_pages[]` **only** if cheap (resolve from chunk metadata at merge); otherwise resolve at query. Prefer query-time join for DRY.  
4. KEEP cap: when trimming `source_chunk_ids`, run **doc-diverse** retention (round-robin across docs).  
5. Refresh entity **vector metadata** from merged graph state (not last-batch snapshot).

### Query (read path)

1. **Strict scope:** `entity_or_rel_passes_filter`  
   - If `source_document_ids` non-empty → ANY match  
   - Else if singular set → match  
   - Else if `source_chunk_ids` non-empty → derive doc ids (`{uuid}-chunk-N` prefix / links table) → ANY match  
   - Else **drop** under active scope (L4)  
2. **kg_chunk_pick:** intersect candidate chunk ids with allowed docs **before** vector fetch.  
3. **Prompt:** entity line may include `docs=k chunks=n` for honesty; citations stay chunk `[N]` + `page=`.  
4. **Telemetry:** `lineage_unknown_drop_rate`, `entity_multi_doc_rate`, `kg_chunk_in_scope_rate`.

### What not to build

- Duplicate entity nodes per document (destroys graph connectivity).  
- Page fields as primary entity properties without chunk ids (lies when multi-page).  
- Full PROV RDF store (TrustGraph-class) before closing the write/read gap above.

---

## 4. Impact on Quality / Reliability / Speed

| Axis | Mechanism | Expected effect |
|------|-----------|-----------------|
| **Quality** | Scope-true entities + in-scope KG chunks only | Fewer wrong-doc facts; better Acc under `document_scope`; cleaner unanswerable |
| **Reliability** | Deterministic lineage; no lenient orphan keep | Reproducible scope; audit: entity → chunks → pages |
| **Speed** | Prefix/SQL filter before graph expand; fewer entities into truncation | Lower local/global latency; fewer tokens → cheaper Gen |

Trade-off: stricter drop of unknown-provenance entities may reduce graph recall until ingest stamps docs — **fail closed on scope**, measure `lineage_unknown_drop_rate`, backfill ingest.

---

## 5. Phased plan

| # | Ticket | Surface | Gate | Effort |
|---|--------|---------|------|--------|
| **L-A1** | Stamp + merge `source_document_ids[]` on ingest | `merger/entity.rs`, `relationship.rs`, `lineage.rs` | Contract: cross-doc entity has ≥2 doc ids | M |
| **L-A2** | Derive doc from chunk-id prefix when plural missing | `context_filter.rs`, helpers | Scope smoke: no foreign-doc entities in context | S |
| **L-A3** | kg_chunk_pick intersects allowed docs | `kg_chunk_pick.rs`, local/global | `kg_chunk_in_scope_rate` ≈ 1 under scope | S |
| **L-A4** | Doc-diverse KEEP when capping | `merge_limits.rs` | Cap does not wipe minority docs | S |
| **L-B1** | Prompt/citation: entity→chunk→page in sources API | `context_format`, `source_reference_builder` | UI can deep-link page from entity cite | M |
| **L-B2** | Telemetry in QueryStats / bench SUMMARY | query + bench047 | Printed every smoke | S |
| **L-C1** | Optional: refresh entity vectors from merged lineage | merger | Metadata matches graph | M |

**Order:** L-A2 (query fail-closed, immediate) → L-A1 (ingest truth) → L-A3 → L-A4 → L-B* → smoke Acc / scope fidelity.

**Do not** block 020 B3 Mix on this — orthogonal. Run L-A2 in parallel with B3.

---

## 6. Anti-patterns

| Anti-pattern | Violates |
|--------------|----------|
| Keep entity under scope when lineage empty | L4 |
| First-doc-wins as filter key | L2 |
| Put `page=` on entity without chunk | L1, L5 |
| Expand full neighborhood then truncate by tokens only | L7 |
| Split entity per document | Graph connectivity / DRY |

---

## 7. Definition of done

- [x] Cross-doc entity stores `source_document_ids` union (ingest contract)  
- [x] Under `document_scope`, context entities/rels all intersect allowed docs (derive from chunks if needed)  
- [x] kg_chunk_pick never fetches out-of-scope chunk ids when scope set  
- [ ] Bench SUMMARY shows lineage drop / multi-doc rates  
- [ ] Unanswerable Acc held ≥ 0.70; scoped Acc not regress vs post-A3  

---

## 8. One-screen law

```text
          keep UNION
  Entity ─────────────▶ {doc₁, doc₂, …} via chunks
     │
     │  query scope?
     ▼
  allowed ∩ docs ≠ ∅  ──yes──▶ keep → pick in-scope chunks → page= from chunks
     │
     no / unknown under scope
     ▼
  DROP (L4) — do not pollute Gen
```
