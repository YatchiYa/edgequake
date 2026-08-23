# 10 — Edge Cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| E1 | Next unit is a new `#` H1 | Do **not** cross-page pack | unit + `U-135-SPAN` negative sibling |
| E2 | Next unit is oversize atomic (table/figure/fence) | Hard split; atomic stays whole | `U-135-NO-SPAN-OVERSIZE` |
| E3 | Language / script change across page (e.g. Latin → CJK body) | Do not span | unit |
| E4 | `EDGEQUAKE_PDF_CROSS_PAGE_PACK=0` | Hard page emit; `page_start == page_end` | `U-135-KILL` sibling |
| E5 | Table overflow | Repeat header+sep (LAW-125-5) | reuse SPEC-125 table unit |
| E6 | Manuscript `grounding:low` | Strip **before** pack (SPEC-134) | existing 134 + pack smoke |
| E7 | Empty page (marker then immediately next marker) | Skip empty segment | unit |
| E8 | CRLF vs LF | Normalize; same pack | unit |
| E9 | CJK / no spaces | tiktoken, not word-count | `U-135-TIKTOKEN` + CJK fixture slice |
| E10 | Comment-only `<!-- multimodal-chunks -->` | Never emit | `U-135-NO-COMMENT` |
| E11 | Lone `<!-- edgequake-page:N -->` | Control plane, not extract | `U-135-NO-COMMENT` |
| E12 | Inline VLM + sidecar same id | Index once | `U-135-MM-ONCE` |
| E13 | Sidecar only (VLM failed / not inlined) | Still append (LAW-047) | unit (negative of MM-ONCE) |
| E14 | `EDGEQUAKE_MM_CHUNKS=0` | No sidecar append at all | existing + 135 |
| E15 | Figure + caption + following prose ≤ budget | Same chunk | `U-135-PROBE` |
| E16 | Figure + prose **over** budget | Figure atomic; prose next chunk; no drop | unit |
| E17 | `min_chunk_size` > `chunk_size` | Floor = `min(min, size)` | reuse 125 |
| E18 | Last remainder under floor | Allowed (LAW-135-7) | unit |
| E19 | Single page 2500 tok | Split inside page; no span needed | `U-135-NO-SPAN-OVERSIZE` |
| E20 | Page1=80 tok + page2=80 tok, same section | One chunk, span 1–2 | `U-135-SPAN` |
| E21 | Content before first marker | Page 1 (existing page_aware) | existing + 135 |
| E22 | Acc PDF documents | Score may drift; pin `PDF_PACK=0` or re-score | [12](12-honest-assessment.md) |
| E23 | Acc R/F non-PDF | Unchanged | `U-135-ACC-R` |
| E24 | YAML frontmatter in converted MD | Pack with body (125) | reuse |
| E25 | Unclosed fence on a page | Remainder atomic | reuse 125 |
| E26 | ATX inside fence | Not a heading | reuse 125 |
| E27 | Rebuild vs future-only | Copy + no auto rebuild | Playwright future-only hint |
| E28 | Historical NULL `page_start` columns | No backfill v1; Rebuild KG | honest |
| E29 | Lineage KV still has JSON pages | Keep writing metadata JSON | E2E-135-01 both column + JSON |
| E30 | Trigger-class 16-page ~26k tok PDF | Gold closed N + fill_p50 | `U-135-FILL` |

## Cross-page pack blockers (normative)

```ascii
  MAY span page N → N+1 iff ALL of:
    combined tiktoken ≤ budget
    neither side is an oversize atomic that must split alone
    next unit is NOT a new `#` H1
    no language/script change heuristic (v1: CJK vs Latin letter-ratio flip)
    EDGEQUAKE_PDF_CROSS_PAGE_PACK != 0
    remainder of N is under floor OR continuation of the same section
```

## Residual risk

Packing across a weak section boundary can mix two topics. Kill switch
`CROSS_PAGE_PACK=0` restores hard pages. Acc PDF N will move (not a bug).
