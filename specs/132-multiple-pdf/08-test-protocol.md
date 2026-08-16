# 08 — Test protocol

## T1 — Unit (always)

| Test | Assert |
|------|--------|
| `ChannelTaskQueue` try_send when full | Returns err/full; does not hang |
| Delivery after persist with full wake | Returns without blocking forever; task durable |
| `bounded-file-upload` | Cap 3; overlapping selections queue |
| `perform-file-upload` | PDF → `/documents/pdf` |

## T2 — Rust e2e admit-non-block (`e2e_spec132_*`)

1. Create `ChannelTaskQueue` with capacity 1 (or 2).
2. Fill to capacity without draining.
3. Call delivery/enqueue path for a new task (or `try_send`).
4. Assert: completes within short timeout (e.g. 1s); no hang.
5. Assert: task still persisted when using full enqueue helper.

## T3 — Playwright multi-PDF (`spec132-multi-pdf-upload-webui.spec.ts`)

Live stack required (`skipUnlessLiveStack`).

1. Bootstrap deterministic UI context.
2. Upload 2 small PDFs via dropzone helper.
3. Assert both filenames appear in table within admit timeout.
4. Assert progress uses distinct tracking (no single frozen panel for both) when observable.
5. Do **not** require KG Completed.

## T4 — Regression

- `e2e_spec054_pdf_progress_identity` still green
- `issue-236-batch-upload-api` still green
- `/upload/batch` PDF rejection still green (SPEC-123)

## Commands

```bash
cargo test -p edgequake-tasks --lib queue
cargo test -p edgequake-tasks --test e2e_spec132_admit_wake_non_block
cd edgequake_webui && pnpm exec playwright test e2e/spec132-multi-pdf-upload-webui.spec.ts
```

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
