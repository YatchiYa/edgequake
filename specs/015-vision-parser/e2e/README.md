# SPEC-015V E2E — Vision extract toggles + prompts

## How to run

Backend must be the **current** binary (workspace PUT returns `vision_extract_*`). Ports come from `.edgequake-dev-ports.env` (not hard-coded `:8080`).

```bash
set -a && . .edgequake-dev-ports.env && set +a
# If PUT ignores vision fields, rebuild + restart:
#   (cd edgequake && cargo build -p edgequake --bin edgequake)
#   make backend-restart

cd edgequake_webui
E2E_LIVE_STACK=1 EQ_BACKEND_URL="$BACKEND_URL" PLAYWRIGHT_BASE_URL="$FRONTEND_URL" \
  pnpm exec playwright test spec015v-vision-extract --reporter=list
```

Asset-gate e2e (pdfium writers, no VLM required):

```bash
cargo test -p edgequake-pdf --test spec015v_asset_gates
```

Screenshots land in [`screenshots/`](screenshots/).

## UI polish (layout / progressive / minimal)

**Upload chrome:** `Parser` + one **Vision** panel trigger (popover form). Effort, modality toggles, and prompts all live inside the panel — no inline strip, no list jump when opening advanced options.

**Wizard:** same form body embedded (no popover).

**Scent:** trigger shows a dot + tooltip summary when modalities/prompts/effort differ from defaults.

**Prompts:** empty override shows the real built-in system prompt text in the textarea (Rust SSOT via `make codegen-vision-prompts`) with a Built-in / Custom badge; Reset restores SSOT; editing equal to default stores empty so future SSOT updates still apply.

## Screenshot analysis (panel UX)

| File | What we verify | Result |
|------|----------------|--------|
| `01-documents-default.png` | Dropzone: Parser + Vision panel when workspace default is Vision | Pass |
| `02-documents-vision-selected.png` | Explicit Vision keeps panel trigger | Pass |
| `03-documents-charts-off.png` | Panel open; Charts OFF | Pass |
| `04-documents-prompt-override.png` | Built-in prompt visible; custom override | Pass |
| `05-documents-edgeparse-hidden.png` | EdgeParse hides Vision trigger | Pass |
| `06-wizard-document-parsing.png` | Wizard Step 2 Vision extract form | Pass |
| `07-wizard-figures-toggled.png` | Figures OFF in wizard | Pass |
| `08-wizard-after-api-put.png` | After PUT persistence | Pass |

## Coverage (closed gaps)

| Gap | Closure |
|-----|---------|
| Asset writers when flags OFF | `VisionAssetWritePlan` + `spec015v_asset_gates` (G8–G10) |
| Pass B prompt override | `prompts::g5_*` |
| Multipart → snapshot | `g7_multipart_overlay_lands_in_snapshot` |
| FE↔Rust prompt drift | `spec015v_fe_prompt_mirror_matches_rust_ssot` + `make codegen-vision-prompts` |
| OpenAPI `vision_extract_*` | `make codegen-openapi-refresh` (G13) |
| Playwright UI/API | `spec015v-vision-extract.spec.ts` (G11–G12, G6) |

See also [`../04-e2e-test-matrix.md`](../04-e2e-test-matrix.md).

## Grade (honest)

| Area | Grade | Notes |
|------|-------|-------|
| Spec pack | A | Matrix G1–G14 tracked with status |
| Domain + writers | A | `VisionAssetWritePlan` SSOT; asset-gate e2e green |
| Prompts DRY | A | `make codegen-vision-prompts` + drift test |
| OpenAPI | A | `vision_extract_*` + prompt fields in snapshot + G13 |
| Playwright UI | A | Panel + PUT live |
| Full HTTP PDF→`GET /assets` | B | Writer-level e2e covers gates; optional live upload still nice-to-have |

## Last green run

- 2026-08-10 — Playwright 2/2; `spec015v_asset_gates` 7/7; prompt drift + G5/G7 green; OpenAPI includes `vision_extract_*`
