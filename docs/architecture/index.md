---
title: Architecture
description: Modular crate architecture and storage backends in EdgeQuake.
---

> **Product: v0.19.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Explore EdgeQuake's modular Rust architecture — **11 workspace crates** under `edgequake/crates/` (no separate `edgequake-llm` crate; LLM providers are composed at runtime via `edgequake-core`).

- **[Overview](/docs/architecture/overview/)** — System diagram, crate inventory, claim/lease task delivery, vision PDF convert → ingest.
- **[Data Flow](/docs/architecture/data-flow/)** — Admit → claim → PdfProcessing → Insert → query; WebSocket progress and cancel.
- **[Data Layer](/docs/deep-dives/data-layer/)** — Postgres ER, KV, AGE, pgvector, FTS, and how query modes read storage.
- **[Lineage Tracking](/docs/architecture/lineage-tracking/)** — Provenance chain including modality and mm-assets.
- **[Crate Reference](/docs/architecture/crates/)** — Per-crate notes.
- **[Ingestion cancel & fairness](/docs/ingestion-cancel-and-fairness.md)** — Operational SSOT for cancel, fairness, restart semantics.
