# 04 — E2E Test Matrix (SPEC-015V)

| Gate | Layer | Assert | Status |
|------|-------|--------|--------|
| G1 | Unit | `VisionExtractConfig::default` all true; empty prompt → None | ✅ `vision_extract` lib tests |
| G2 | Unit | resolve upload false overrides workspace true | ✅ |
| G3 | Unit | prompt cap >32KiB rejected | ✅ |
| G4 | Unit + e2e | `VisionAssetWritePlan` + writers skip when flags false | ✅ `spec015v_asset_gates` |
| G5 | Unit | Pass B messages use override system text | ✅ `prompts::g5_*` |
| G6 | API | PUT workspace round-trip metadata keys | ✅ Playwright live PUT |
| G7 | API | multipart overlay lands in snapshot / plan | ✅ `g7_multipart_overlay_lands_in_snapshot` |
| G8 | E2E | figures=false → no `-fig-`; figures=true writes figs | ✅ `spec015v_asset_gates` |
| G9 | E2E | charts=false → no `-chart` | ✅ |
| G10 | E2E | images=false → no page PNG; images=true writes page PNG | ✅ |
| G11 | Playwright | wizard shows Vision extract when Vision | ✅ `spec015v-vision-extract.spec.ts` |
| G12 | Playwright | upload Vision panel; hidden for EdgeParse | ✅ |
| G13 | Contract | OpenAPI schema includes `vision_extract_*` | ✅ `make codegen-openapi-refresh` |
| G14 | DRY | FE prompt mirror == Rust SSOT | ✅ `spec015v_vision_prompt_codegen` + `make codegen-vision-prompts` |

## How to run

```bash
# Rust gates + DRY drift
cargo test -p edgequake-pdf --test spec015v_asset_gates
cargo test -p edgequake-api --test e2e_spec015v_vision_extract
cargo test -p edgequake-api --test spec015v_vision_prompt_codegen
cargo test -p edgequake-api g5_ --lib

# Regenerate FE prompts / OpenAPI when Rust changes
make codegen-vision-prompts
make codegen-openapi-refresh

# Live WebUI
set -a && . .edgequake-dev-ports.env && set +a
cd edgequake_webui
E2E_LIVE_STACK=1 EQ_BACKEND_URL="$BACKEND_URL" PLAYWRIGHT_BASE_URL="$FRONTEND_URL" \
  pnpm exec playwright test spec015v-vision-extract --reporter=list
```
