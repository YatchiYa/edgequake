# 09 — Acceptance

| # | Criterion | Proof |
|---|-----------|-------|
| A1 | Spec pack complete with cross-refs + ASCII | This folder |
| A2 | Reproduction classifies admit vs post-admit | [12-reproduction.md](12-reproduction.md) |
| A3 | Multi-PDF WebUI e2e: ≥2 PDFs admit + list presence | Playwright spec132 |
| A4 | Full wake channel does not hang HTTP/delivery forever | Rust e2e/unit |
| A5 | Docs do not route PDFs through `/upload/batch` | Grep docs green |
| A6 | #378 comment with root-cause + link to SPEC-122 | GitHub |
| A7 | #361 throughput not claimed fixed | Honest assessment |

## Done when

All A1–A6 true; A7 explicit in [11-honest-assessment.md](11-honest-assessment.md).

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
