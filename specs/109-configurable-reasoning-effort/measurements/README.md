# SPEC-109 — Measurements

> Placeholder for post-implementation proof artifacts. Do not invent numbers here.

## Purpose

Store logs and captures that close findings F1–F8 and gates E2E-109-01…10.

## Expected artifacts (after Wave 4)

| File | Contents |
|------|----------|
| `SUMMARY.md` | Pass/fail table for gates |
| `e2e109-openai-serialize.txt` | Unit output proving JSON field present |
| `e2e109-clamp-table.txt` | Registry test output (mini/nano/large) |
| `e2e109-extract-options.txt` | Mock capture of CompletionOptions |
| `e2e109-effective-config.json` | Sample `/config/effective` payload |
| `e2e109-playwright.md` | UI run notes + screenshot paths |
| `e2e109-live-openai.txt` | Optional live run (redact keys) |

## How to regenerate

```bash
make spec109-reasoning-effort-proof
# optional:
OPENAI_API_KEY=… cargo test -p … e2e109_live -- --ignored
```

## Status

| Gate | Status |
|------|--------|
| Pack authored | **Done** (SPEC-109 docs) |
| Wave 0–4 code | Pending |
| Proof files | Pending |
