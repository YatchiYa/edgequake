# 07 — Implementation plan

## Work packages

```ascii
  WP-0  Spec pack (this folder)
  WP-1  Reproduce + classify → 12-reproduction.md
  WP-2  Admit honesty: non-blocking / timed wake enqueue SSOT
  WP-3  WebUI: per-file timeout/error isolation
  WP-4  Docs/tutorial: PDF ≠ /upload/batch
  WP-5  Tests: multi-PDF Playwright + Rust admit-non-block
  WP-6  GitHub #378 comment + cross-link #361/#365
```

## WP-2 — Enqueue SSOT

Files:

- `edgequake/crates/edgequake-tasks/src/queue.rs` — `try_send` on `ChannelTaskQueue`
- Delivery path used by `TaskRuntime::enqueue` / `enqueue_with_delivery` — after persist, wake without unbounded await
- Hydrate already recovers Pending tasks (SPEC-057) — keep that invariant

**Exit:** unit/e2e proves full channel → enqueue returns Ok (or typed soft fail that still leaves durable row + 202 path).

## WP-3 — WebUI isolation

Files:

- `edgequake_webui/src/hooks/use-file-upload.ts`
- `edgequake_webui/src/lib/upload/bounded-file-upload.ts`
- `edgequake_webui/src/lib/upload/upload-timeout.ts` / multipart client

Ensure each `run()` callback always settles (success/error) so `finally` releases the slot; surface per-file error message on XHR timeout.

## WP-4 — Docs

Search/fix any tutorial that posts PDFs to `/documents/upload/batch`. Point to `/documents/pdf` or `/pdf/batch`.

## WP-5 — Tests

See [08-test-protocol.md](08-test-protocol.md).

## DRY / SOLID checklist

- [x] One wake delivery path for all task types (`enqueue_with_delivery` + `try_send`)
- [x] No WebUI `/pdf/batch` migration in v1
- [x] No unbounded vision raise
- [x] Progress still prefers `task_id`
- [x] Edge matrix EC-1..EC-12 mapped to tests (Playwright + Rust + unit + docs)

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
