# 09 — Acceptance

## Ship criteria (docs pack — this PR)

- [x] Full cross-ref pack under `specs/134-manuscrit/`
- [x] WHY + LAW-134-* + architecture + lenses + plan + tests + edges + honest assessment
- [x] No trigger filename or content quotes
- [x] Fixtures rubric present

## Ship criteria (implementation — follow-up)

### Must

1. Forced `EDGEQUAKE_PDF_PAGE_MODALITY=manuscript` applies DPI floor + MS prompt.
2. Print modality leaves Acc English Pass-A prompt unchanged (contract).
3. MS modality skips EdgeParse Auto fast-path (**must classify first**; honor `skip_edgeparse_fastpath`).
4. Noise crops below threshold do not get Pass-B specialize.
5. Hand-chart pages: axis-tick / single-bar fragment crops get **zero** Pass-B specialize;
   Pass-A MD carries whole-graphic series / Key values (`E2E-134-04`, LAW-134-16).
6. `document_pages.page_modality` persisted; API exposes it.
7. UI shows modality chip on MS docs (`data-testid="page-modality-chip"`).
8. Edge matrix in [10-edge-cases.md](10-edge-cases.md) has test IDs mapped.
9. SPEC-133 arrow cases still pass (no regress).
15. Pass-A `ConversionConfig.max_rendered_pixels` is **3600** for manuscript groups (not viewer-only).
16. Empty Pass-A manuscript (or empty print) page markdown has **zero** `fig-` / chart hrefs (LAW-134-20).
17. Mixed documents: print pages do not receive the MS prompt (Acc English pin preserved).
18. ImageGuard must not downscale manuscript Pass-A below 2000px long side.

### Should

10. Synthetic gold CER/WER gates in CI or `make spec134-proof`.
11. Confidence badge when non-null.

### Must not

12. Claim archival HTR accuracy in release notes.
13. Ship AGPL HTR weights.
14. Commit private trigger scans.

## Sign-off

| Role | Signs |
|------|-------|
| PO | JTBD demo on synthetic MS |
| Fullstack | Contracts green |
| DB | Migration expand-contract reviewed |
| UX | Chip hierarchy review |
| OCR/AI | Prompt + metrics review |

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
