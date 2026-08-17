# 00 — First Principles (SPEC-098)

## Axioms

1. **Identity is relational for typed fleet.** UUID FKs on `entity_embeddings` / `relationship_embeddings` require a prior spine row.  
2. **Projections never invent parents.** Fleet mirror resolves; it does not create entities.  
3. **KEEP is a graph merge policy, not a CQRS policy.** Saturation must not erase identity ensure.  
4. **One normalizer per label.** Relation type casing is part of identity.  
5. **Evidence beats vibes.** Failures name misses; tests prove the invariant in CI.  
6. **Upserts are deterministic.** Postgres forbids one `ON CONFLICT DO UPDATE` from affecting a row twice; batches must be unique on the arbiter after BEFORE INSERT triggers.  
7. **One arbiter per table.** Dual UNIQUE indexes on AGE `EDGE`/`Node` cause non-arbiter unique violations or cardinality under multigraph.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-098-1** | Spine before projection — no typed fleet write without a resolvable relational FK. |
| **LAW-098-2** | KEEP is AGE-only — saturation skips graph mutation, never spine ensure. |
| **LAW-098-3** | One identity SSOT — bare `entities.name` ↔ `entity:NAME`; uppercase `relation_type` everywhere. |
| **LAW-098-4** | Fail closed with evidence — typed mirror fails when `resolved < eligible`; error samples misses. |
| **LAW-098-5** | Version via capability — PG16/17/18 via `capabilities.rs`; migrations use portable SQL. |
| **LAW-098-6** | CI is proof — unwired tests are documentation, not gates. |
| **LAW-098-7** | Single EDGE arbiter `(eq_source_id, eq_target_id, eq_rel_type)`; dual UNIQUEs forbidden; boot reconcile always drops legacy. |
| **LAW-098-8** | Every `ON CONFLICT DO UPDATE` batch writer dedupes to its arbiter key (AGE edges, entity sink, relationship sink, vectors) via shared normalizers. |
| **LAW-098-9** | Lifecycle admit is dual-written — KV metadata **and** `public.documents.status` set to `deleting` after durable job enqueue; list merge treats `deleting` as inflight vs stale relational success; `delete_failed` is a terminal failure. |
| **LAW-098-10** | Delete UI has one lifecycle SSOT — deletion sessions drive feedback + table dimming; pins block terminal poll overwrite until document absence or `delete_failed`; batch task completion is not per-doc success without absence proof. |
| **LAW-098-11** | Lifecycle statuses pass through shell/SQL writers unchanged (`deleting` / `delete_failed` are not pipeline `cancelled` / `failed`); batch failures carry per-id reasons; UI verbs match lifecycle (Retry delete ≠ reprocess). |
| **LAW-098-12** | Subtractive cascade writes must not use ingest union merge — shared-entity prune uses property Replace (`EXCLUDED.properties`); `eq_merge_graph_properties` remains ingest-only. |
| **LAW-098-13** | Document provenance ≠ graph topology — never treat edge `source_id`/`target_id` (`workspace::ENTITY`) as provenance; cascade identity is `(src,tgt,rel)`; exclusive delete binds **raw** `properties.relation_type` and matches via the **trigger formula** `UPPER(COALESCE(NULLIF(TRIM(…), ''), 'RELATED_TO'))` (SQL SSOT — not Rust ASCII upper / dual heuristics); singular citation fields stay coherent with arrays. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | `normalize_relation_type_str` / `dedupe_edges_by_endpoints` shared by sink, vector ids, fleet bind, graph upsert; spine ensure reuses `EntitySinkRow` + `upsert_entities_batch`; `admit_documents_deleting` shared by single + batch delete; shell status normalizer is the single CHECK allowlist. |
| **SRP** | Sink writes spine; fleet resolves FKs; merger orchestrates KEEP vs ensure; graph adapter owns AGE arbiters; admit helper owns lifecycle status dual-write; cascade owns purge; batch worker reports per-id failure reasons. |
| **OCP/DIP** | `FleetEmbeddingIndex::mirror_legacy_batch` returns a report; callers decide fail-closed. |
| **LSP** | All adapters implement the report contract (no-op adapters: eligible=0). |
