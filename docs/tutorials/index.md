---
title: Tutorials
description: Step-by-step tutorials for common EdgeQuake workflows — from your first RAG app to advanced multi-tenant deployments.
---

<<<<<<< HEAD
> **Product: v0.19.0** · Contract: [`openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
=======
> **Product: v0.23.0** · Contract: [`openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

Step-by-step tutorials to get the most out of EdgeQuake. All examples follow the **OpenAPI snapshot** (SSOT): `DocumentSummary` uses `display_status` / `ui_phase` / `track_id` / `current_stage`; query returns `answer` + `sources`; progress uses `/ws/progress/{track_id}` (not legacy `/rag/*` routes).

- **[Building Your First RAG App](/docs/tutorials/first-rag-app/)** — Build a knowledge graph and run your first query from scratch.
- **[Document Ingestion Deep-Dive](/docs/tutorials/document-ingestion/)** — Explore the full document ingestion pipeline in detail.
- **[PDF Ingestion](/docs/tutorials/pdf-ingestion/)** — Ingest PDF files and extract structured knowledge.
- **[Query Optimization](/docs/tutorials/query-optimization/)** — Tune your queries for speed and quality.
- **[Multi-Tenant Setup](/docs/tutorials/multi-tenant/)** — Deploy EdgeQuake for multiple tenants with isolated workspaces.
- **[Migration from LightRAG Python](/docs/tutorials/migration-from-lightrag/)** — Migrate your LightRAG Python project to EdgeQuake Rust.
- **[Tracing Entity Sources](/docs/tutorials/tracing-entity-sources/)** — Debug and trace entity extraction sources through the pipeline.
- **[Knowledge Injection](/docs/tutorials/knowledge-injection/)** — Domain glossaries that enrich retrieval without polluting citations.
