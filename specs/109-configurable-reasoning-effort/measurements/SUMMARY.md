# SPEC-109 proof SUMMARY

Date: 2026-08-05

| Gate | Result | Evidence |
|------|--------|----------|
| E2E-109-01 OpenAI JSON field | PASS | `e2e109-openai-serialize.txt` / llm crate |
| E2E-109-02 clamp mini/nano | PASS | `e2e109-clamp-table.txt` |
| E2E-109-03 extract floor | PASS | `e2e109-contract-proof.txt` |
| E2E-109-04 query request field | PASS | contract `e2e109_04_*` |
| E2E-109-05 mistral omit | PASS | contract + clamp table |
| E2E-109-06 effective config | PASS | contract `e2e109_06_*` |
| E2E-109-07 OpenAPI | PASS | `e2e109-openapi.txt` (spec027 120 ok) + codegen refresh |
| E2E-109-08 Playwright | PASS | `make spec109-e2e` — 6/6; shots in [`e2e/screenshots/`](e2e/screenshots/README.md) |
| E2E-109-09 cache hash | PASS | `e2e109-cache-hash.txt` |
| E2E-109-10 live OpenAI | SKIP (optional) | not run |

## Deploy / cutover

| Item | Result |
|------|--------|
| `edgequake-llm` crates.io | **0.10.4** published (Anthropic `output_config.effort`, shared clamp, OpenRouter forward) |
| EdgeQuake `[patch.crates-io]` | **Removed** — resolves `edgequake-llm = "0.10.4"` from crates.io |
| Vision PDF effort | Injected via `ReasoningEffortInjectProvider` (pdf2md lacks effort field) |

## UX visual QC (2026-08-05)

| Surface | Pass? | Notes |
|---------|-------|-------|
| Query sheet effort | PASS | Control + filtered options; POST includes `reasoning_effort` |
| Settings fleet + by-role | PASS | Auto default; extract/query/vlm overrides |
| Explainability roles | PASS | Desired/effective/source; no illegal options shown for Mistral |
| Workspace role effort | PASS | Extract/Query effort readonly lines |
| Documents upload | PASS | Parser + **Vision effort** when Vision selected (`07-documents-vision-effort.png`) |
| Auto effective / best practice | PASS | Auto option + hint show effective (`omit` for query; lowest structured for extract/fleet) |

Aggregator: `make spec109-reasoning-effort-proof` → OK.  
Live UI: `make spec109-e2e` (needs healthy `dev-bg`; seed tenant via Playwright helper).
