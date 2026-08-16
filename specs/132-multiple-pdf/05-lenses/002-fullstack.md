# Lens 002 — Full Stack Developer

## Stake

One enqueue SSOT; one WebUI upload stack; tests that prove Plane A.

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| SRP | Delivery helper owns wake semantics; handlers own multipart/PDF validation |
| OCP | New task types inherit non-blocking wake without PDF forks |
| DRY | No second “PDF-only enqueue”; no duplicate timeout math outside `upload-timeout.ts` |
| ISP | Progress tracking consumes `task_id` only |
| DIP | Handlers depend on `enqueue_task` abstraction |

## Concrete changes

1. `ChannelTaskQueue`: expose `try_send` / timed send; delivery uses it after persist.
2. WebUI: guarantee executor `finally` release + per-file error on XHR timeout.
3. Playwright: `e2e/spec132-multi-pdf-upload-webui.spec.ts` (PDF, not MD).
4. Rust: `e2e_spec132_admit_wake_non_block` (or tasks-crate unit) filling channel then admit.

## Anti-patterns

- Migrating WebUI to `/pdf/batch` “to fix concurrency” (creates body-sum 50 MiB trap).
- Raising `MAX_CONCURRENT_FILE_UPLOADS` without server fairness story.
- Special-casing PDF in `task_runtime` only.

## Cross-refs

- As-is: [../03-code-as-is.md](../03-code-as-is.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
