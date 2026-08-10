# 05 — Edge Cases (SPEC-015V)

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-015V-1 | All three OFF | Pass A OCR still runs; no crops; no Pass B visual; no broken asset hrefs | G4/G8–10 |
| EC-015V-2 | Charts ON, Figures OFF | Chart residual without figure_map; skip fig-as-chart promotion | unit |
| EC-015V-3 | Figures ON, Charts OFF | No chart crops / chart mm chunks | G9 |
| EC-015V-4 | Images OFF, Figures/Charts ON | Skip page PNGs; fig/chart still write; no invented page PNG hrefs | G10 |
| EC-015V-5 | EdgeParse + toggles sent | Ignore flags; soft warn OK | unit |
| EC-015V-6 | Huge prompt override | Cap 32 KiB → 400 | G3 |
| EC-015V-7 | Empty string override | Clear → SSOT | G1 |
| EC-015V-8 | Upload overrides workspace | Per-field upload wins | G2 |
| EC-015V-9 | process_options vs extract | extract_images false ⇒ no image analyze even if `i` | unit |
| EC-015V-10 | Reprocess | Prefer ingest snapshot on doc metadata | G7 |
| EC-015V-11 | Chart+figure same page both ON | Unchanged heuristics | regression SPEC-047 |
| EC-015V-12 | Prompt injection / XSS | Trusted-admin; store as text; UI escape | review |
