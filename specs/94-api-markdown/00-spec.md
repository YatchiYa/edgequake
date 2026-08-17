# Standalone Document Parsing Endpoint (PDF to Markdown)

Captured: July 31, 2026
Core Concept: Expose EdgeQuake's existing PdfConverter as a direct, synchronous HTTP endpoint so PDF to Markdown parsing can be benchmarked and tuned without running the full ingestion pipeline, similar to LightOn's parsing API.
Domain: Architecture
Next Steps: 1) Confirm sync vs async default and payload size ceiling. 2) Add POST /v1/parse and /v1/parse/backends handlers in edgequake-api. 3) Wire OpenAPI path SSOT entries. 4) Build a golden-set benchmark harness under benches/ that calls the endpoint. 5) Publish a per-backend baseline scorecard.
Priority: 4
Stage: Refined
Tags: High Impact, Quick Win, Technical
Type: Tool

<aside>
📌

**Source:** [edgequake @ feat/version-023](https://github.com/raphaelmansuy/edgequake/tree/feat/version-023) (commit `e4fd0f6`). This specification is grounded in the actual crate layout of that branch, in particular `crates/edgequake-pdf` and `crates/edgequake-api`.

</aside>

## Problem

Today the only way to obtain parsed Markdown from a document in EdgeQuake is to submit it through the full ingestion pipeline. That path performs conversion, chunking, embedding, entity extraction, graph construction, and storage writes before anything is observable. Measuring the quality or the latency of the first stage alone therefore requires paying for every stage.

This has three consequences. Benchmark turnaround is slow and expensive, because every experiment pays large language model (LLM) and storage costs unrelated to parsing. Attribution of a regression is ambiguous, because a bad answer may originate in parsing, chunking, or retrieval. Comparison against external parsing services such as LightOn is not possible on equal terms, since those services expose parsing as a single request.

The request from the field is direct: expose the parsing step, the same one the pipeline uses, as a standalone endpoint that returns Markdown.

## Goals

- Provide a synchronous HTTP endpoint that accepts a document and returns Markdown, using the exact converter the pipeline invokes.
- Allow the parser backend, the vision provider, the model, the dots-per-inch (DPI) render resolution, and the concurrency to be selected per request.
- Return timing and cost metadata sufficient to build a benchmark scorecard without external instrumentation.
- Keep the endpoint stateless by default, so that repeated benchmark runs leave no documents, no assets, and no graph rows behind.

## Non-goals

- Replacing the ingestion pipeline or the existing PDF upload endpoints.
- Persisting parse results, creating documents, or emitting lineage records.
- Adding a new parser implementation. This specification exposes what exists.
- Chunking, embedding, or entity extraction. The response stops at Markdown.

## What already exists on this branch

The important architectural finding is that parsing is already isolated behind a trait. The coupling to the pipeline lives in the application programming interface (API) layer, not in the parsing crate, so the work is a thin handler rather than a refactor.

The conversion contract in `crates/edgequake-pdf/src/backend/mod.rs` is already a pure function from bytes and configuration to a Markdown string:

```rust
#[async_trait]
pub trait PdfConverter: Send + Sync {
    async fn convert(
        &self,
        pdf_bytes: &[u8],
        config: &PdfConversionConfig,
    ) -> Result<String, PdfConversionError>;

    fn backend_name(&self) -> &'static str;
}

pub fn create_pdf_converter(backend: PdfParserBackend) -> Arc<dyn PdfConverter>;
```

Because `convert` takes bytes and returns a string, it can be called from a request handler with no database, no task queue, and no workspace context.

Two backends are registered behind the `PdfParserBackend` enum. `Vision` is the default and drives a vision language model (VLM) over rendered page images. `EdgeParse` is the deterministic extraction path. Selection is currently environment-scoped through `EDGEQUAKE_PDF_PARSER_BACKEND`, which accepts the aliases `vision` or `llm`, and `edgeparse`, `edge-parse`, or `edge_parse`.

The table below maps the existing building blocks to the role each plays in the proposed endpoint.

| Existing component | Location | Role in the endpoint |
| --- | --- | --- |
| `PdfConverter` trait | `edgequake-pdf/src/backend/mod.rs` | The single call the handler makes |
| `PdfParserBackend` | `edgequake-pdf/src/backend/mod.rs` | Per-request backend selection |
| `PdfConversionConfig` | `edgequake-pdf/src/backend/mod.rs` | Carries filename, page count hint, table method |
| `VisionConversionConfig` | `edgequake-pdf/src/backend/mod.rs` | Carries provider, model, concurrency, DPI |
| `PageDrawingAssetsConfig` | `edgequake-pdf/src/backend/mod.rs` | Optional asset emission, disabled by default here |
| `should_fallback_to_edgeparse` | `edgequake-pdf/src/fallback.rs` | Controls the vision failure fallback |
| `resolve_pdf_page_count` | `edgequake-pdf/src/page_count.rs` | Populates the page count hint and per-page metrics |
| `multipart_upload.rs`, `file_validation.rs`, `safety_limits.rs` | `edgequake-api/src` | Reused for intake, validation, and ceilings |
| `openapi_path_ssot.rs` | `edgequake-api/src` | Registers the new paths in the generated schema |

Everything in that table already ships on the branch, which is why this feature is scoped as a handler plus schema work.

## Proposed API surface

Three endpoints are proposed. The first performs the parse, the second describes what the server can do, and the third supports long documents.

### Parsing a document synchronously

`POST /v1/parse` accepts either `multipart/form-data` with a `file` part, or `application/pdf` with the raw bytes and an optional `X-Filename` header. Options are supplied as a JSON `options` part in the multipart case, or as query parameters in the raw case.

The following request illustrates a vision parse against a specific model, with page timings requested:

```bash
curl -X POST https://host/v1/parse \
  -H "Authorization: Bearer $EDGEQUAKE_TOKEN" \
  -F "file=@invoice.pdf" \
  -F 'options={"backend":"vision","provider":"ollama","model":"qwen2.5vl:7b","dpi":200,"concurrency":4,"include_page_timings":true}'
```

Each option maps onto a field that the converter already understands, so no new configuration surface is introduced.

| Option | Type | Default | Maps to |
| --- | --- | --- | --- |
| `backend` | `"vision"` or `"edgeparse"` | server default | `PdfParserBackend` |
| `provider` | string | server default | `VisionConversionConfig.provider_name` |
| `model` | string | provider default | `VisionConversionConfig.model` |
| `dpi` | integer, 72 to 400 | 150 | `VisionConversionConfig.dpi` |
| `concurrency` | integer, 1 to 16 | server default | `VisionConversionConfig.concurrency` |
| `pages` | range string, such as `"1-10"` | all pages | Page selection before render |
| `table_method` | string | server default | `PdfConversionConfig.table_method` |
| `emit_assets` | boolean | `false` | `PageDrawingAssetsConfig` |
| `allow_fallback` | boolean | `true` | `should_fallback_to_edgeparse` |
| `include_page_timings` | boolean | `false` | Metrics detail level |

Because `emit_assets` defaults to `false`, the default benchmark path writes nothing to disk and the figure filter pass is skipped entirely.

### Response shape

The response carries the Markdown alongside the metadata a benchmark harness needs, so that a single request is self-describing:

```json
{
  "markdown": "# Invoice 2026-114\n\n...",
  "backend": "vision",
  "backend_effective": "vision",
  "fallback_applied": false,
  "page_count": 12,
  "metrics": {
    "total_ms": 8420,
    "render_ms": 610,
    "ocr_ms": 7580,
    "assemble_ms": 230,
    "pages_per_second": 1.42,
    "prompt_tokens": 18240,
    "completion_tokens": 6110,
    "estimated_cost_usd": 0.0413
  },
  "page_timings": [{ "page": 1, "ms": 612, "chars": 1840 }],
  "warnings": [],
  "request_id": "pr_01JZQ..."
}
```

The `backend_effective` field matters for honest measurement. When a vision parse fails and the fallback engages, the reported backend differs from the requested one, and a scorecard that ignores this will attribute EdgeParse output to the vision model.

### Describing server capabilities

`GET /v1/parse/backends` returns the available backends, the vision providers currently reachable, the models each provider advertises, and the configured limits. A benchmark harness calls this once and then enumerates the matrix it is allowed to run, rather than hard-coding model names that may not be installed.

### Handling long documents

A synchronous request is the right default for benchmark work, where documents are typically small and the caller wants a single round trip. Long documents need a different shape. When `Prefer: respond-async` is sent, or when the resolved page count exceeds the configured synchronous ceiling, the server responds `202 Accepted` with a job identifier. The caller then polls `GET /v1/parse/jobs/{id}` and receives the same response body once the job completes. This reuses the existing operation and status patterns already present in `handlers/pdf_upload`.

## Errors

Errors follow the existing API error type in `edgequake-api/src/error.rs`. The mapping below keeps parsing failures distinguishable from transport and policy failures, which is necessary when a benchmark run reports a non-zero failure rate.

| Condition | Status | Code |
| --- | --- | --- |
| Missing or unreadable file part | 400 | `parse.invalid_request` |
| Unsupported media type | 415 | `parse.unsupported_media_type` |
| Encrypted or malformed document | 422 | `parse.document_unreadable` |
| Payload or page count above the ceiling | 413 | `parse.too_large` |
| Vision provider unreachable, fallback disabled | 502 | `parse.backend_unavailable` |
| Conversion exceeded the deadline | 504 | `parse.timeout` |

When `allow_fallback` is `true`, a vision failure that `should_fallback_to_edgeparse` classifies as recoverable produces a `200` response with `fallback_applied` set and a warning entry, rather than an error.

## Safety, limits, and access control

The endpoint accepts arbitrary uploads and drives model inference, so it inherits the existing controls rather than bypassing them. Authentication uses the same middleware as the rest of the v1 surface. File validation reuses `file_validation.rs`, and payload ceilings reuse `safety_limits.rs`. A dedicated rate limit and a dedicated concurrency semaphore are added, because an unbounded benchmark loop against a shared vision provider would otherwise starve ingestion traffic.

Uploaded bytes are held in memory or in a temporary file for the duration of the request and are removed when the response is written. No workspace scope is required, since nothing is persisted.

## Observability

The handler emits a `parse_request` span carrying the backend, the provider, the model, the DPI, the page count, and the outcome. Counters track requests by backend and by outcome, and histograms track total duration, render duration, and per-page duration. These are the same dimensions the response returns, so dashboards and scorecards agree.

## Implementation plan

1. Add `handlers/parse/` in `edgequake-api` with `mod.rs`, `types.rs`, and `handler.rs`, following the structure already used by `handlers/pdf_upload/`.
2. Build `PdfConversionConfig` and `VisionConversionConfig` from the validated request options, then call `create_pdf_converter(backend).convert(...)`.
3. Instrument the conversion with the existing `VisionStatusHook` to capture render and optical character recognition (OCR) phase boundaries for the metrics block.
4. Register the routes in `routes.rs` and the schemas through `openapi_path_ssot.rs` and `openapi_examples.rs`.
5. Add the asynchronous job variant on top of the existing operation store.
6. Add a benchmark harness under `benches/` that reads a golden set, sweeps the backend and model matrix returned by `/v1/parse/backends`, and writes a comparison report.
7. Extend the software development kits under `sdks/` with a `parse` method.

## Acceptance criteria

- A single `POST /v1/parse` request returns Markdown for a twelve-page PDF with no database write, no queued task, and no document row created.
- Markdown produced by the endpoint is byte-identical to the Markdown the pipeline produces for the same document, the same backend, and the same model, with assets disabled.
- `GET /v1/parse/backends` lists both backends and reflects provider availability at call time.
- Requesting an unavailable model returns `502` with `parse.backend_unavailable` rather than a silent fallback when `allow_fallback` is `false`.
- The benchmark harness completes a full matrix sweep over a fifty-document golden set and emits a per-backend scorecard.
- Repeated runs leave no residue on disk, verified by comparing the temporary directory before and after a sweep.

## Open questions

- Should the synchronous ceiling be expressed in pages, in megabytes, or in both? A page-based ceiling is more predictable for vision parsing, where cost scales with rendered pages.
- Should the endpoint expose a `format` option for future targets such as HyperText Markup Language (HTML) or structured JavaScript Object Notation (JSON), or stay Markdown-only until a second consumer exists?
- Should an optional structured layout block, listing detected figures and tables per page, be returned when `emit_assets` is `true`? This would make figure detection quality measurable, which the Markdown alone does not expose.
- Does the benchmark scorecard need a reference-comparison metric, such as normalized edit distance against a human-corrected Markdown target, or is human review of the golden set sufficient for the first iteration?