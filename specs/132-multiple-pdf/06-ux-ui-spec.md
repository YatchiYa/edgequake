# 06 — UX / UI spec

## Surfaces

1. **Dropzone** — multi-select PDFs (existing).
2. **UploadProgressList** — per-file rows for transfer/admit.
3. **Documents table** — admit presence (`file_source` / title).
4. **Ingestion banner** — post-admit queue / fairness (SPEC-122 P0).

## Behavioral requirements

| ID | Requirement |
|----|-------------|
| UX-132-1 | Each selected PDF gets its own progress row |
| UX-132-2 | Timeout/error marks that row failed; other rows continue |
| UX-132-3 | After all admits settle, header must not imply infinite “Transferring” |
| UX-132-4 | Post-admit stages use processing vocabulary (Queued/Converting) |
| UX-132-5 | Duplicate dialog still works for multi-PDF duplicates |

## Copy snippets

- Waiting: `Waiting for upload slot…`
- Admit: `Saving to workspace…`
- Fail: `Upload failed. You can retry this file.`
- Done transfer: `Transfer complete — processing in background`

## Cross-refs

- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
