---
title: 'EdgeQuake Architecture Overview'
---

# EdgeQuake Architecture Overview

<<<<<<< HEAD
> **Product: v0.19.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
=======
> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

> Understanding the system design through first principles

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
<<<<<<< HEAD
│ EdgeQuake v0.19.0                                                       │
=======
│ EdgeQuake v0.23.0                                                       │
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
│                                                                         │
│  Client: WebUI :3000  |  REST/WS :8080  |  SDKs                         │
│                      |                                                  │
│                      v                                                  │
│  edgequake-api (Axum, OpenAPI, auth, WS progress)                       │
│       |                  |                  |                           │
│       v                  v                  v                           │
│  edgequake-tasks    edgequake-core     edgequake-pdf                    │
│  claim/lease        orchestrate        vision convert                   │
│  cancel/fairness    insert / query    + mm-assets                       │
│       |                  |                  |                           │
│       |           +------+------+           |                           │
│       |           v             v           |                           │
│       |      pipeline        query          |                           │
│       |   chunk/extract    6 RAG modes      |                           │
│       |           +------+------+           |                           │
│       |                  v                  |                           │
│       +--------> edgequake-storage <--------+                           │
│                  KV | pgvector | AGE                                    │
│                         |                                               │
│                         v                                               │
│                   PostgreSQL 16-18                                      │
│            (required; no server in-memory mode)                         │
│                                                                         │
│  Cross-cutting: auth | audit | rate-limiter | observability             │
│  LLM providers composed in core (no edgequake-llm crate)                │
└─────────────────────────────────────────────────────────────────────────┘```

---

## Design Principles

### Why Rust?

| Factor          | Python (LightRAG) | Rust (EdgeQuake) | Impact          |
| --------------- | ----------------- | ---------------- | --------------- |
| **Performance** | ~100 docs/min     | ~1000 docs/min   | 10x throughput  |
| **Memory**      | 2-4GB typical     | 200-400MB        | 10x efficiency  |
| **Concurrency** | GIL limited       | True async       | Better scaling  |
| **Type Safety** | Runtime errors    | Compile-time     | Fewer prod bugs |
| **Deployment**  | Python env + deps | Single binary    | Simpler ops     |

### Why 11 Crates?

**Single Responsibility Principle** — workspace crates under `edgequake/crates/`:

```
edgequake-api           HTTP + WebSocket + OpenAPI
edgequake-core          Orchestration (EdgeQuake facade, LLM provider wiring)
edgequake-pipeline      Chunk · extract · embed · merge
edgequake-query         RAG query engine (6 modes)
edgequake-storage       PostgreSQL + pgvector + Apache AGE
edgequake-pdf           PDF → markdown (vision / EdgeParse) + mm-assets
edgequake-tasks         Task queue, workers, claim/lease, cancel, fairness
edgequake-auth          JWT, API keys, OIDC, tenant context
edgequake-audit         Compliance audit events
edgequake-rate-limiter  Tenant throttling
edgequake-observability Tracing, metrics, correlation
```

There is **no** `edgequake-llm` or `edgequake-graph` crate — LLM and graph logic live inside `core`, `pipeline`, `query`, and `storage`.

**Benefits**:

1. **Compile-time boundary enforcement** — Can't accidentally use internal types
2. **Parallel compilation** — Each crate compiles independently
3. **Selective testing** — Run tests for one crate only
4. **Clear dependency graph** — Easy to understand data flow
5. **Swappable implementations** — Change storage without touching query

### Why Trait-Based Abstraction?

```rust
// Core orchestrator — concrete LLM/storage wired at startup
pub struct EdgeQuake {
    // LLM + embedding: Arc<dyn …> from edgequake-core provider factory
    // Storage: Arc<dyn KVStorage>, VectorStorage, GraphStorage
}
```

**Advantages**:

- Production uses OpenAI, tests use Mock (zero code changes)
- Add new providers without modifying core
- Runtime provider switching (dev → prod)

---

## Crate Dependency Graph

```
┌─────────────────────────────────────────────────┐
│ Crate dependency (simplified)                   │
│                                                 │
│           edgequake-api                         │
│                 |                               │
│     +-----------+-----------+                   │
│     v           v           v                   │
│   core        tasks       auth                  │
│     |           |       audit/obs               │
│     +-----+-----+                               │
│     v     v                                     │
│ pipeline query                                  │
│     |     |                                     │
│     v     |                                     │
│    pdf    |                                     │
│     +--+--+                                     │
│        v                                        │
│     storage --> PostgreSQL                      │
└─────────────────────────────────────────────────┘
```

**Task delivery (SPEC-057):** API admits work → Postgres `Pending` row → worker `claim_next` + lease → `PdfProcessing` or `Insert` handlers. In-memory channel is wake-only.

---

## The 11 Crates Explained

| Crate | Purpose | Key types / notes |
| ----- | ------- | ----------------- |
| **edgequake-api** | HTTP REST, WebSocket progress, OpenAPI | `Router`, handlers, `/ws/progress/{track_id}` |
| **edgequake-core** | Central orchestration + LLM provider factory | `EdgeQuake`, `EdgeQuakeConfig`, provider wiring |
| **edgequake-pipeline** | Document processing | `Pipeline`, chunker, extractor, merge |
| **edgequake-query** | Search and retrieval | `QueryEngine`, `QueryMode` (6 modes) |
| **edgequake-storage** | Persistence | `KVStorage`, `VectorStorage`, `GraphStorage`, Postgres |
| **edgequake-pdf** | PDF → markdown, vision LLM, mm-assets | Convert phase for `TaskType::PdfProcessing` |
| **edgequake-tasks** | Background jobs | `claim_next`, lease, cancel registry, tenant fairness |
| **edgequake-auth** | Authentication | JWT, API keys, tenant/workspace context |
| **edgequake-audit** | Compliance logging | Audit events |
| **edgequake-rate-limiter** | Request throttling | Tenant quotas |
| **edgequake-observability** | Ops | Tracing, metrics, store contention signals |

See [Crate Reference](/docs/architecture/crates/) for per-crate detail.

---

## Key Architectural Patterns

### 1. Facade Pattern (EdgeQuake)

The `EdgeQuake` struct is a facade that coordinates all RAG operations:

```rust
// Simple interface hides complex internals
let eq = EdgeQuake::new(config)
    .with_providers(llm, embedder)
    .with_storage(kv, vector, graph)
    .initialize()
    .await?;

// User doesn't know about Pipeline, QueryEngine, etc.
let result = eq.insert("Document content").await?;
let response = eq.query("What is X?").await?;
```

### 2. Strategy Pattern (Query Modes)

Six different query strategies, selected at runtime:

```
            ┌─────────────────┐
            │   QueryEngine   │
            └────────┬────────┘
                     │
     ┌───────────────┼───────────────┐
     │               │               │
     ▼               ▼               ▼
┌─────────┐    ┌─────────┐    ┌─────────┐
│  Naive  │    │  Local  │    │ Global  │
│ (vector)│    │ (entity)│    │(commun.)│
└─────────┘    └─────────┘    └─────────┘
     │               │               │
     └───────────────┴───────────────┘
                     │
     ┌───────────────┼───────────────┐
     │               │               │
     ▼               ▼               ▼
┌─────────┐    ┌─────────┐    ┌─────────┐
│ Hybrid  │    │   Mix   │    │ Bypass  │
│(L+G)    │    │(weighted)│   │(no RAG) │
└─────────┘    └─────────┘    └─────────┘
```

### 3. Pipeline Pattern (Document Processing)

PDF and async text admission use a **two-phase** task pipeline (SPEC-057 P2):

```
┌───────────────────────────────────────────────────────┐
│ Convert then ingest (SPEC-057)                        │
│                                                       │
│  POST /documents/pdf  -->  admit task_id              │
│              |                                        │
│              v                                        │
│  [1] PdfProcessing (convert only)                     │
│      vision / edgeparse --> markdown                  │
│      PDF row --> Completed (artifact)                 │
│              |                                        │
│              v  markdown barrier                      │
│  [2] Insert (KG ingest, new lease)                    │
│      chunk --> extract --> embed --> store            │
│              |                                        │
│              v                                        │
│  document display_status = completed                  │
└───────────────────────────────────────────────────────┘
```

Cancel, fairness park, and lease refresh are handled in `edgequake-tasks`. Operational detail: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md).

### 4. Adapter Pattern (Storage)

Multiple backends behind unified traits:

```
┌─────────────────────────────────────────────────────────┐
│                   Storage Traits                         │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │
│  │  KVStorage   │ │VectorStorage │ │ GraphStorage │     │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘     │
└─────────┼────────────────┼────────────────┼─────────────┘
          │                │                │
    ┌─────┴─────┐    ┌─────┴─────┐    ┌─────┴─────┐
    │           │    │           │    │           │
    ▼           ▼    ▼           ▼    ▼           ▼
┌───────┐  ┌───────┐ ┌───────┐  ┌───────┐ ┌───────┐  ┌───────┐
│Memory │  │Postgres││Memory │  │pgvector││Memory │  │  AGE  │
└───────┘  └───────┘ └───────┘  └───────┘ └───────┘  └───────┘
```

---

## Multi-Tenancy Architecture

EdgeQuake supports multi-tenant isolation via `tenant_id` and `workspace_id`:

```
┌─────────────────────────────────────────────────────────┐
│                    Request Flow                         │
│                                                         │
│  Request ──▶ [Middleware] ──▶ [Handler] ──▶ [Storage]   │
│               │                    │             │      │
│               ▼                    ▼             ▼      │
│         Extract tenant       Validate        Filter by  │
│         from header          permissions     namespace  │
│                                                         │
└─────────────────────────────────────────────────────────┘

Isolation enforced at storage layer:
- Tenant A cannot see Tenant B's documents
- Workspace 1 cannot see Workspace 2's entities
```

---

## Next Steps

- **[Data Flow](/docs/architecture/data-flow/)** — Admit → claim → convert → Insert → query
- **[Crate Details](/docs/architecture/crates/)** — Deep dive into each crate
- **[Ingestion cancel & fairness](/docs/ingestion-cancel-and-fairness.md)** — Cancel SSOT, claim/lease
- **[API Reference](/docs/api-reference/rest-api/)** — REST endpoint documentation

---

## Code References

| Component        | File                               | Lines |
| ---------------- | ---------------------------------- | ----- |
| EdgeQuake struct | edgequake-core/src/orchestrator.rs | 1-300 |
| QueryMode enum   | edgequake-core/src/types/query.rs  | -     |
| Pipeline struct  | edgequake-pipeline/src/pipeline.rs | 1-100 |
| Storage traits   | edgequake-storage/src/traits/      | -     |
| API routes       | edgequake-api/src/routes.rs        | -     |
