# 08 — Test protocol

## Gates overview

```ascii
  G-class   classifier heuristics          unit
  G-profile DPI/max_px floors              contract
  G-prompt  MS vs print prompt select      contract
  G-edgeparse MS blocks Auto fast-path     contract
  G-passb   noise crop not specialized     contract / e2e mock
  G-persist modality+confidence columns    e2e / sql
  G-gold    CER/WER + table F1 + chart KV  offline / CI optional
  G-ui      modality chip visible          Playwright
  G-print   print Acc prompt unchanged     regression contract
  G-slicee  pixels forwarded, zero fig hrefs on empty/MS, EdgeParse veto, mixed stitch
```

## Naming

| ID | Artifact |
|----|----------|
| `E2E-134-01` | Mock VLM convert with forced `PAGE_MODALITY=manuscript` |
| `E2E-134-02` | Auto backend + MS fixture never EdgeParse-only |
| `E2E-134-03` | Pass-B noise crop suppressed |
| `E2E-134-04` | Hand-chart page: axis-tick + single-bar crops → zero Pass-B; MD has whole-graphic KV |
| `E2E-134-UI-chip` | Playwright chip + MD panel dominant |
| `contract_spec134_*` | Rust contracts in `edgequake-api` / `edgequake-pdf` |
| `contract_spec134_slice_e` | Assemble / pixels / EdgeParse / mixed-stitch behavior |

## Fixtures

Location: [`fixtures/`](fixtures/) — **synthetic only**.

| Fixture | Purpose |
|---------|---------|
| `print_simple.pdf` | Classifier → print; G-print |
| `ms_image_primary.pdf` | Full-page scan synthetic handwriting |
| `ms_implicit_table.gold.md` | Gold for table F1 |
| `ms_hand_chart.gold.md` | Gold for chart KV |
| `ms_noise_crop.png` | Tiny scribble for Pass-B gate |

Rules: no PII; no trigger content; ASCII/latin synthetic strings OK.

## Metrics (G-gold)

Normalize Unicode NFKC; collapse whitespace for WER optionally (document choice).

| Metric | Threshold (start) | Move only with corpus Δ |
|--------|-------------------|-------------------------|
| CER | ≤ 0.15 on synthetic clear ink | Yes |
| WER | ≤ 0.25 | Yes |
| Table cell F1 | ≥ 0.80 | Yes |
| Chart KV recall | ≥ 0.70 | Yes |
| Forced-English violations | 0 when gold language ≠ English | Yes |
| Crop theater count | 0 on MS fixture | Yes |
| Chart-fragment Pass-B count | 0 tick/bar crops specialized | Yes |

## Mock strategy

- Unit/contract: no live VLM.
- E2E: mock provider returns fixture MD; assert wiring (DPI, prompt id, modality persist).
- Live VLM: optional `LIVE-134` gated; not required for merge.

## Commands (intended)

```bash
cargo test -p edgequake-pdf --test contract_spec134_modality_profile
cargo test -p edgequake-api --test contract_spec134_manuscript_prompt
cargo test -p edgequake-api --test e2e_spec134_manuscript_convert
# FE
pnpm exec playwright test e2e/spec134-modality-chip.spec.ts
# optional
make spec134-proof
```

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- Fixtures: [fixtures/README.md](fixtures/README.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
