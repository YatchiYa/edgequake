# 004 — Acceptance and edge-case tests

## Hard CI gates

| Gate | Assert |
|------|--------|
| G1 | No invented asset paths in assemble/inject |
| G2 | Full-page `page-NNNN.png` never Drawing-eligible |
| G3 | Region / chart crop area ≤ 55% of page |
| G4 | L0 proposals win over L1 for same element when both exist |
| G5 | Keyword chart gate is not the sole source of a figure/table crop |

## Edge-case matrix

| ID | Case | Expected |
|----|------|----------|
| E1 | ImageXObject figure | `fig` crop ≈ object pixels / bbox |
| E2 | Vector Form XObject + “Figure N” text | `fig` region, not full page, not invent |
| E3 | Ruled table + “Table N” | `table` crop ≤55% |
| E4 | Form + table on same page | both assets; chart skipped |
| E5 | Text-only page | no fig/table/chart |
| E6 | Near-full Form (~80% page) | rejected by area invariant |
| E7 | Tiny ornament Image (&lt;24px / &lt;2% area) | skipped |
| E8 | Overlapping Forms | single clustered region |
| E9 | Missing on-disk fig href in MD | stripped on inject |
| E10 | Chart residual with table present | no chart override |
| E11 | Empty PDF / zero pages | empty ok |
| E12 | Caption body mention without visual | no invented crop |
| E13 | StructTree Figure (tagged fixture) | L0 `RegionSource::StructTree`; G4 wins over L1 |
| E14 | Multi-page doc | per-page indices stable |
| E15 | Ideas arXiv PDF (`ideas_2607.08758v1.pdf`) | ≥10 figs; Figure 1–10; ObjectCluster; G3; ≥10 `-fig-`; **0** `-table-` |
| E16 | Hierar arXiv PDF (`hierar_2607.02980v1.pdf`) | ≥7 figs; Figure 1–7; G3; ≥7 `-fig-`; **0** `-table-` |
| E17 | LightRAG arXiv PDF (`lighrad_2410.05779v3.pdf`) | ≥5 figs; Figure 1,3–7; G3; ≥5 `-fig-`; **0** `-table-` (Fig 2 may be absent) |
| E18 | Page has ImageXObject + Form figure | Form crop still written (IoU merge; no any-embed skip) |
| E19 | Path-only cluster inside Form | suppressed (containment ≥ 0.8) |
| E20 | Chart residual propose | Pages without fig/table only; ink gates — not `text_suggests_chart` |

## Suites

- pdf2md: `visual::` unit + precision + E13 tagged L0 + `e14`/`e15`/`e16` corpus fixtures  
- edgequake-pdf: assemble/inject identity + `should_write_region_figure` + `chart_residual_*`  
- edgequake-api: `contract_spec049_*` + `e2e_spec049_visual_regions` (Ideas / Hierar / LightRAG + E20)  
- FE: existing mm-asset rewrite (no invent)
- Fixtures: `specs/048-improve-ux/e2e/{ideas,hierar,lighrad}_*.pdf`
- pdf2md **0.9.7**: StructTree L0 + placement-first L1 + Form-first precision + IoU dedup + P2 ink residual

