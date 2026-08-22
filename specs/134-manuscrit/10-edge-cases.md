# 10 — Edge cases

Every row: mitigation + test ID. Implementers must not ship without covering **Must**.

| ID | Case | Risk | Mitigation | Test |
|----|------|------|------------|------|
| EC-01 | Mixed print + handwritten margins | Wrong profile | `mixed` → MS render floor + MS prompt | G-class |
| EC-02 | Multi-orientation pages | Crop/render skew | pdfium page rot; doc-majority gate | G-profile |
| EC-03 | Graph-paper high-frequency grid | Grid as “text” | MS prompt: grid is background; keep ticks | G-gold chart |
| EC-04 | Color-as-series histograms | Series collapse | LAW-134-5; prompt color series | G-gold |
| EC-05 | Implicit tables (no rules) | Lost numbers | Prompt alignment→GFM | G-gold table |
| EC-06 | Strikeouts / scribbles | Noise in index | Omit struck; keep page PNG | G-prompt |
| EC-07 | Bleed-through | Ghost text | Prompt ignore faint reverse; `[?]` | G-gold |
| EC-08 | Faint pencil | Missed content | DPI floor; low confidence | G-profile + persist |
| EC-09 | Rotated margin labels | Missed labels | Prompt read vertical text | G-gold |
| EC-10 | Scanner OCR layer non-empty | Auto EdgeParse | LAW-134-12 veto fast-path | E2E-134-02 |
| EC-11 | Empty VLM output | Silent success | EmptyOutput fail; confidence 0 | contract |
| EC-12 | Explicit Vision fail | Wrong fallback | Fail closed (existing); no EdgeParse on MS Auto either | contract |
| EC-13 | Large MS PDF vs DPI floor cost | Timeout / VRAM | concurrency≤2; stall SPEC-057; env opt-out | contract timeout |
| EC-14 | Diacritics / non-English | EN paraphrase | MS prompt no translate; forced-EN=0 | G-gold |
| EC-15 | Greek / math / subscripts | Lost symbols | Prompt preserve LaTeX/symbols | G-gold |
| EC-16 | Arrow/brace diagram names | KG delimiter | Depend SPEC-133; prefer bullet relations | cross e2e 133 |
| EC-17 | Figure filter signature discard | Drop hand marks | WP-4 MS policy | G-passb |
| EC-18 | Pass-B scribble theater | UX lie | Area/ink gate; accordion collapsed | E2E-134-03 / UI |
| EC-26 | Axis-tick digit crops Pass-B | Graphic atomization | LAW-134-16; tick-strip aspect gate | E2E-134-04 |
| EC-27 | Single histogram bar crop Pass-B | Geometric “frame” essay | Chart-band child suppress | E2E-134-04 |
| EC-28 | Multi-panel hand histograms | N charts → N atoms | Pass-A one section per panel; no per-bar crops | G-gold chart |
| EC-19 | Acc English vs MS | Break Acc | Print prompt untouched | G-print |
| EC-20 | `max_pixels` not raised with DPI | Soft DPI | Profile couples both floors | G-profile |
| EC-21 | Force modality env for tests | Leak to prod | Document; default unset | contract |
| EC-22 | Confidence null vs 0 | UX wrong | null hides badge; 0 shows Low | UI unit |
| EC-23 | Per-page vs doc-majority | Wrong page DPI | v1 majority; document limitation | honest assess |
| EC-24 | Tiny page count high ink plot | Classify print | Image-primary overrides ink | G-class |
| EC-25 | Operator sets DPI 96 globally | Crush MS | MS floor wins unless modality forced print | G-profile |
| EC-29 | Verify pass loops / cost | Latency | Single refine only; gate by confidence | contract |
| EC-30 | Consensus two-VLM cost | 2x cost | Default off; opt-in env | contract |
| EC-31 | Frontier VLM unavailable | Fallback | Document fallback chain; fail honest | honest assess |
| EC-32 | Uncalibrated confidence misleads | UX lie | v1 heuristic documented; v2 calibration | honest assess |

## Priority

- **Must for slice A:** EC-10, EC-11, EC-12, EC-17, EC-18, EC-19, EC-20, **EC-26, EC-27**
- **Must for slice B:** EC-22, UI for EC-18 / EC-26
- **Must for slice C:** EC-03…EC-09, EC-14, EC-15, **EC-28**
- **Must for slice D:** EC-29, EC-30, EC-31, EC-32

## Cross-refs

- Tests: [08-test-protocol.md](08-test-protocol.md)
- Laws: [01-first-principles.md](01-first-principles.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
