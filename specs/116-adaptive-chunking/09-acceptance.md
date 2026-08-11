# 09 — Acceptance

| # | Criterion | Pass? |
|---|-----------|-------|
| 1 | Docs pack complete with WHY → acceptance + lenses | |
| 2 | `ChunkingPolicy` resolve DRY in pipeline | |
| 3 | Workspace metadata + API round-trip | |
| 4 | Worker uses workspace policy before doc options | |
| 5 | OpenAPI / WebUI types include fields | |
| 6 | `WorkspaceChunkingCard` + Acc-fair chip | |
| 7 | Future-only hint + rebuild messaging | |
| 8 | Validation overlap &lt; size | |
| 9 | Default Inherit preserves env Acc behavior | |
| 10 | Unit + contract + e2e/Playwright evidence | |

## Sign-off

Implementation complete when all rows checked and `cargo test` / webui tests for touched surfaces pass.
