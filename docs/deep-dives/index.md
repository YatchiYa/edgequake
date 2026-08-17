---
title: Deep Dives
description: In-depth technical explorations of EdgeQuake internals.
---

<<<<<<< HEAD
> **Product: v0.19.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Detailed technical deep dives into EdgeQuake's implementation.

**Recently rewritten for v0.19.0:** [Pipeline Progress](/docs/deep-dives/pipeline-progress/) and [PDF Processing](/docs/deep-dives/pdf-processing/) reflect SPEC-047 vision ingest, mm-assets, convert→ingest split, and SPEC-057 cancel/status SSOT.
=======
> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Detailed technical deep dives into EdgeQuake's implementation.

**Recently rewritten for v0.23.0:** [Pipeline Progress](/docs/deep-dives/pipeline-progress/) and [PDF Processing](/docs/deep-dives/pdf-processing/) reflect SPEC-047 vision ingest, mm-assets, convert→ingest split, and SPEC-057 cancel/status SSOT.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

- **[Data Layer](/docs/deep-dives/data-layer/)** — PostgreSQL ER, KV SSOT, AGE, pgvector, FTS, and query×store matrix (code is law).
- **[LightRAG Algorithm](/docs/deep-dives/lightrag-algorithm/)** — The algorithm behind entity extraction and graph construction.
- **[Entity Extraction](/docs/deep-dives/entity-extraction/)** — How LLMs extract entities from text.
- **[Entity Normalization](/docs/deep-dives/entity-normalization/)** — Deduplication and canonicalization of entities.
- **[Chunking Strategies](/docs/deep-dives/chunking-strategies/)** — Document splitting approaches and trade-offs.
- **[Query Modes](/docs/deep-dives/query-modes/)** — The 6 retrieval modes explained.
- **[Graph Storage](/docs/deep-dives/graph-storage/)** — PostgreSQL AGE integration for graph operations.
- **[Vector Storage](/docs/deep-dives/vector-storage/)** — pgvector for embedding-based retrieval.
- **[Embedding Models](/docs/deep-dives/embedding-models/)** — Supported embedding providers and configuration.
- **[Community Detection](/docs/deep-dives/community-detection/)** — Graph clustering for global queries.
- **[PDF Processing](/docs/deep-dives/pdf-processing/)** — Vision LLM PDF conversion, mm-assets, convert vs ingest.
- **[Gleaning](/docs/deep-dives/gleaning/)** — Multi-pass entity extraction for higher recall.
- **[Cost Tracking](/docs/deep-dives/cost-tracking/)** — Monitor and control LLM API costs.
- **[Pipeline Progress](/docs/deep-dives/pipeline-progress/)** — Real-time processing status and progress tracking.
