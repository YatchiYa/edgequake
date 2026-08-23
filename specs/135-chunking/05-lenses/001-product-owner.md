# Lens 001 — Product Owner

## Stake

Partners upload PDFs (papers, reports, scans). They pin **Match LightRAG /
Fixed 1200/100** and still see 70 extract jobs on a 16-page paper, empty-looking
chunks, and citations that cannot deep-link. That is a **trust failure**: the
product looks like it ignored the size they set.

## Outcomes (v1)

1. A 16-page ~26k-token technical PDF at 1200/100 lands in the gold **N** range
   (~24–32) with **fill p50 ≥ 55%**, not 70 chunks at 19% fill.
2. Each figure is indexed **once** (no duplicate chart chunks in lineage).
3. Citations show `p.3` or `p.3–4`; click opens the **start** page.
4. Kill switches restore old geometry without a code revert.
5. Rebuild remains explicit (future ingestions only).
6. Acc score on **PDF** corpora may move; we say so (no silent claim of
   Acc-neutral PDF). Non-PDF Acc R/F stay pinned.

## Non-outcomes (v1)

Late chunking, LLM-written context prefixes, semantic-V, auto-rebuild,
changing Acc Recursive word-count on plain text, vendoring partner papers
into git.

## Acceptance narrative

> As a partner, I upload a 16-page PDF with figures at workspace 1200/100.
> Lineage shows packed chunks (not one chunk per heading/figure/page). Charts
> appear once. A chunk that continues onto the next page shows `p.3–4` and
> opens page 3. If fill looks wrong, support can read `fill_p50` in Langfuse
> without opening the file. If we must freeze Acc PDF, ops sets
> `EDGEQUAKE_PDF_PACK=0`.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- Honest Acc: [../12-honest-assessment.md](../12-honest-assessment.md)
- UX: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
