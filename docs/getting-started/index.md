---
title: Getting Started
description: Install EdgeQuake, run your first pipeline, and understand the basics.
---

<<<<<<< HEAD
> **Product: v0.19.0** · Contract: [OpenAPI snapshot](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
=======
> **Product: v0.23.0** · Contract: [OpenAPI snapshot](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

Get up and running with EdgeQuake in minutes.

- **[Installation](installation.md)** — System requirements (Rust 1.95+, PostgreSQL 16–18), auth, and vision prerequisites.
- **[Quick Start](quick-start.md)** — Async upload with `track_id`, graph entities, and your first query.
<<<<<<< HEAD
- **[Docker quickstart](../operations/docker-quickstart.md)** — Prebuilt GHCR images (`0.19.0`) without a local Rust toolchain.
=======
- **[Docker quickstart](../operations/docker-quickstart.md)** — Prebuilt GHCR images (`0.23.0`) without a local Rust toolchain.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
- **[Runtime auth hardening](../operations/runtime-auth-hardening.md)** — Auth-on-by-default; use `EDGEQUAKE_DEV_MODE=true` only for local `make dev`.
- **[Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)** — Claim/lease, cancel → `Cancelled`, convert→ingest, multi-replica.
- **[PDF Ingestion](../tutorials/pdf-ingestion.md)** — Vision / EdgeParse PDF pipeline with progress WebSockets.
