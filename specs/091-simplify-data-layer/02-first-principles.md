# 02 — First Principles

> Method: reduce the problem to what a retrieval system *minimally requires*, state those requirements as axioms, derive enforceable laws (`LAW-D1..D8`), and anchor each law to (a) official documentation current as of July 2026 and (b) the code that today honors or violates it. Nothing in [05-target-specification.md](05-target-specification.md) or [06-implementation-plan.md](06-implementation-plan.md) is justified by taste; every decision traces to a law.
>
> **As-of banner:** the **Today** column below describes **published pin v0.22.0** (KV SSOT, empty `chunks` writers). After Waves A–D, several laws are **Honored / Partially** on HEAD — see the report card in [16-post-cutover-assessment.md](16-post-cutover-assessment.md). Do not rewrite this table; treat it as the pin-era baseline.

## Axioms (from 00-raw-needs, retained verbatim)

1. **A retrievable chunk is a tuple, not a row.** It is retrievable only if its text, its embedding, its routing attributes, and its graph links all exist and agree.
2. **A tuple spanning several stores needs one identity.** Without a single key every store agrees on, integrity can only be inferred.
3. **A tuple spanning several stores needs one commit boundary or one visibility fence.** If neither exists, partial states are observable by readers.
4. **A count is a projection of a state machine.** Where no state machine exists, every counter is an independent opinion.

## From axioms to enforceable laws

| Law | Statement | Derived from | Official anchor (July 2026) | Today |
| --- | --- | --- | --- | --- |
| **LAW-D1 — Tuple integrity** | A chunk is served iff text ∧ embedding ∧ routing ∧ graph links are all present and agree; otherwise it is invisible (fail-closed). | Axiom 1 | pgvector filtered search semantics ([README § Filtering](https://github.com/pgvector/pgvector#filtering)) | **Violated** — no readiness notion exists; a partially written chunk is queryable (F-091-01) |
| **LAW-D2 — Single identity** | One chunk has exactly one identifier, of one type, in every store and every lineage structure. | Axiom 2 | PostgreSQL 18 `uuidv7()` — timestamp-ordered, index-friendly IDs ([release notes](https://www.postgresql.org/docs/18/release-18.html)) | **Violated** — `chunks.id uuid` (minted by no one) vs `{doc}-chunk-{n}` text (minted everywhere) (F-091-03) |
| **LAW-D3 — Commit or fence** | A multi-store tuple either commits atomically or is hidden behind a serving fence until all projections confirm. | Axiom 3 | PostgreSQL 18 transaction/WAL semantics; `RETURNING OLD/NEW` for outbox patterns ([release notes](https://www.postgresql.org/docs/18/release-18.html)) | **Violated** — three independent commits + after-the-fact compensation (F-091-01, F-091-05) |
| **LAW-D4 — Counts are projections** | No counter exists without a state machine it projects; derived counters are labeled, scheduled, and never trusted by the read path as truth. | Axiom 4 | PostgreSQL statistics/views discipline | **Violated** — `documents.chunk_count`, `eq_*_stats`, and live `COUNT(*)` over an empty table disagree (F-091-08, F-091-11) |
| **LAW-D5 — One schema owner** | All DDL is issued by numbered, digest-verified migrations; request-serving and boot code issue none. One inspectable schema generation exists at any time. | Axioms 2, 3 | PostgreSQL transactional DDL; pgvector sizing/limits ([README § Scaling](https://github.com/pgvector/pgvector#scaling)) | **Violated** — `ddl.rs` creates `eq_*_vectors` and applies six `ALTER TABLE`s with `.ok()` (`vector/ddl.rs:267-285`); `kv.rs` creates KV tables + indexes at runtime (F-091-04, F-091-10) |
| **LAW-D6 — One authoritative row per fact** | Every durable fact has exactly one authoritative row; every other copy is a labeled projection with a freshness contract. | Axioms 1, 2 | TOAST keeps large text out of the spine's way ([storage-toast](https://www.postgresql.org/docs/18/storage-toast.html)); stored generated columns keep indexes over the authoritative value ([ddl-generated-columns](https://www.postgresql.org/docs/18/ddl-generated-columns.html)) | **Violated** — declared authority (`chunks.content`) empty; de-facto authority (KV JSONB) unreachable by constraints (F-091-02) |
| **LAW-D7 — Batch-first boundaries** | No interface crossing a store boundary may require a round trip per row; every hot operation has a batch form. | Efficiency corollary of Axiom 1 | pgvector: "Use `COPY` for bulk loading" ([README § Loading](https://github.com/pgvector/pgvector#loading)); existing `unnest`-batch upserts (`vector/storage_impl.rs:225`) | **Mostly honored inside adapters, broken by design at the KV port** — string-key/opaque-value forces per-fact key derivation and suffix scans (F-091-10) |
| **LAW-D8 — Scale work off request paths** | Work proportional to total data (counts, backfills, index builds, reconciliation, fairness scans) runs asynchronously, from bounded windows, on the maintenance pool. | Axioms 3, 4 | PostgreSQL 18 async I/O accelerates scans/vacuum ([release notes](https://www.postgresql.org/docs/18/release-18.html)); keyset pagination; `FOR UPDATE SKIP LOCKED` | **Partially honored** (statement-level counters, CIC index builds) — **violated** by suffix scans for counts and boot-time relation discovery (F-091-08, F-091-10) |

## How the laws map to DRY, SOLID, SSOT

```ascii
 FIRST PRINCIPLE        DRY                       SOLID                       SSOT
 ┌─────────────┐  ┌──────────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐
 │ LAW-D2      │─▶│ one key-grammar,     │  │ DIP: app depends on  │─▶│ chunk UUID is the    │
 │ identity    │  │ derived once, not    │  │ typed ChunkId, not   │  │ single source; the   │
 │             │  │ re-derived per store │  │ on key strings       │  │ string key is retired│
 ├─────────────┤  ├──────────────────────┤  ├──────────────────────┤  ├──────────────────────┤
 │ LAW-D5      │─▶│ one chunks           │  │ OCP: schema evolves  │─▶│ migrations/ is the   │
 │ schema owner│  │ definition, not 3    │  │ by new migrations,   │  │ only schema narrator │
 │             │  │ (F-091-13)           │  │ not by patching code │  │                      │
 ├─────────────┤  ├──────────────────────┤  ├──────────────────────┤  ├──────────────────────┤
 │ LAW-D6      │─▶│ chunk text stored    │  │ SRP: chunks owns     │─▶│ content + generated  │
 │ authority   │  │ once; tsv generated  │  │ text; embeddings own │  │ tsv cannot drift —   │
 │             │  │ from the same value  │  │ vectors; AGE owns    │  │ one value, one truth │
 │             │  │                      │  │ traversal (SRP)      │  │                      │
 ├─────────────┤  ├──────────────────────┤  ├──────────────────────┤  ├──────────────────────┤
 │ LAW-D7      │─▶│ batch builders       │  │ ISP: narrow ports    │─▶│ conformance suite is │
 │ ports       │  │ shared via one       │  │ (Document/Chunk/     │  │ the single contract  │
 │             │  │ IngestionPersister   │  │ Embedding/Graph...)  │  │ every adapter meets  │
 ├─────────────┤  ├──────────────────────┤  ├──────────────────────┤  ├──────────────────────┤
 │ LAW-D1/D3   │─▶│ one fence state      │  │ LSP: adapters are    │─▶│ chunk_serving_state  │
 │ fence       │  │ machine, not N       │  │ substitutable only   │  │ is the one readiness │
 │             │  │ ad-hoc presence      │  │ when the suite says  │  │ truth for readers    │
 │             │  │ probes               │  │ so (LSP proven)      │  │                      │
 └─────────────┘  └──────────────────────┘  └──────────────────────┘  └──────────────────────┘
```

### SOLID, made concrete for this refactor

- **SRP** — one relation per fact family (`chunks` = text+lineage, `chunk_embeddings` = vectors, `chunk_serving_state` = lifecycle, AGE = traversal); one writer site for ingestion persistence (`ingestion_persister.rs`, F-091-15); one module per adapter.
- **OCP** — new providers arrive as new adapters behind existing ports; no application module changes (LD-05).
- **LSP** — substitutability is not asserted, it is *tested*: the port conformance suite runs against every registered adapter in CI, including the in-memory one; an adapter that fails is not shipped.
- **ISP** — ports stay narrow and fact-shaped, following the existing precedent (`GraphStorageReadOps` / `GraphStorageMutateOps` / `GraphStorageAnalyticsOps` in `edgequake/crates/edgequake-storage/src/traits/`).
- **DIP** — application modules depend on `DocumentRepository`, `ChunkRepository`, `EmbeddingIndex`, `GraphProjection`… never on SQL, relation names, `halfvec`, Cypher, or key strings. Enforced by a dependency lint in CI (Wave 1 exit gate, Table: acceptance).

## The one disagreement this spec settles by law, not by vote

SPEC-021 (storage study) concluded the current architecture should be repaired in place, largely to keep Cypher traversal. This spec reaches a different conclusion on a narrow point: **LD-04 keeps AGE as traversal authority**, so the study's central objection does not apply — while SPEC-021's own five invariants (currently unenforced) become enforceable precisely by LAW-D2/D3/D5 (foreign keys, fence, migration ownership). The two documents agree on the problem; they differ on whether invariants are enforced by convention or by the database. LAW-D5's anchor — transactional DDL and declarative constraints — is the tiebreaker: what PostgreSQL can declare, PostgreSQL will enforce; what convention declares, drift will eventually break (F-091-14 is the standing proof).
