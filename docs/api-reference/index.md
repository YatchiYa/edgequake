---
title: API Reference
description: Complete REST API documentation for EdgeQuake.
---

<<<<<<< HEAD
> **Product: v0.19.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Full reference for the EdgeQuake REST API.

**Authoritative contract:** [`edgequake_webui/openapi/openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) (version `0.19.0`). **Interactive Try-it-out:** [`/swagger-ui/`](http://localhost:8080/swagger-ui/) when the backend is running.

- **[REST API](/docs/api-reference/rest-api/)** — v0.19.0 guided overlay (progress, cancel, status fields).
- **[Extended API](/docs/api-reference/extended-api/)** — Tasks, pipeline queue-metrics, WebSocket progress, PDF cancel.
- **[Document Upload](/docs/api-reference/document-upload-quick-reference/)** — JSON vs file vs PDF vs batch decision tree.
- **[Lineage Endpoints](/docs/api-reference/lineage-endpoints/)** — Provenance, mm-assets, convert vs ingest.
=======
> **Product: v0.23.0** · Contract: OpenAPI · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

Full reference for the EdgeQuake REST API.

**Authoritative contract:** [`edgequake_webui/openapi/openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) (version `0.23.0`). **Interactive Try-it-out:** [`/swagger-ui/`](http://localhost:8080/swagger-ui/) when the backend is running.

- **[REST API](/docs/api-reference/rest-api/)** — v0.23.0 guided overlay (progress, cancel, status fields, **parse API**).
- **[Extended API](/docs/api-reference/extended-api/)** — Tasks, pipeline queue-metrics, WebSocket progress, PDF cancel.
- **[Document Upload](/docs/api-reference/document-upload-quick-reference/)** — JSON vs file vs PDF vs batch decision tree.
- **[Lineage Endpoints](/docs/api-reference/lineage-endpoints/)** — Provenance, mm-assets, convert vs ingest.
- **[Parse API (SPEC-094)](/docs/api-reference/rest-api/#parse-api-spec-094)** — stateless PDF→Markdown, no document residue.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
- **[Ingestion cancel & fairness](/docs/ingestion-cancel-and-fairness.md)** — Cancel SSOT, claim/lease, tenant fairness, store contention.
