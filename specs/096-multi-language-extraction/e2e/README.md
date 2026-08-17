# SPEC-096 E2E proofs

## Artifacts

| Path | Purpose |
|------|---------|
| `run_spec096_proof.sh` | curl create/get/reject language round-trip |
| `artifacts/RUN_NOTES.md` | Last curl proof summary |
| `screenshots/*.png` | Playwright UI captures (S01–S05) |
| `screenshots/RUN_NOTES.md` | Per-scenario capture notes |
| `screenshots/ANALYSIS.md` | PASS/FAIL verdict matrix |

## WebUI Playwright

```bash
cd edgequake_webui
PLAYWRIGHT_SKIP_STACK_CHECK=1 pnpm exec playwright test \
  e2e/spec096-extraction-language.spec.ts --project=chromium
```

## Curl proof

```bash
# Requires healthy backend (make status / make dev-bg)
bash specs/096-multi-language-extraction/e2e/run_spec096_proof.sh
```

## Cargo gates

```bash
cd edgequake
cargo test -p edgequake-pipeline --lib spec096
cargo test -p edgequake-api --test contract_spec096_extraction_language
cargo test -p edgequake-api --features postgres --test e2e_spec096_extraction_language
```
