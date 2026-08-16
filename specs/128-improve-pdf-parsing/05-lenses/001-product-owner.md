# Lens 001 — Product Owner

## Stake

Analysts trust GraphRAG only if indexed figures are real diagrams, not logos. They also need to **see** extraction on the original page — markdown crops alone do not prove alignment. Operators pay VLM cost for every proposed crop; an open classify loop burns money and pollutes the graph.

## Outcomes (v1)

1. Precision release: ingest prunes discarded figures; logos/stamps do not become Drawing assets.
2. Layout release: overlay toggle on the document PDF pane — figures, charts, tables, paragraphs, columns, noise — aligned under zoom.
3. Honest empty states: overlay disabled when layout was skipped/failed; never fake boxes.
4. No AGPL surprise in the Docker image; Apache default model, pinned hash.
5. Cost: geometry + layout gates reduce VLM calls/page vs today’s “keep everything”.

## Non-outcomes (v1)

- Replacing Pass-A page OCR with MinerU.
- Shipping DocLayout-YOLO in GHCR.
- Citation highlighting as PDF text quads (markdown line highlight stays SPEC-033).
- Filling unused SQL `chunks.page_start` columns.

## Acceptance narrative

> As an analyst, I upload a paper with a logo header and a real architecture diagram. After processing, the diagram is in markdown and on the overlay as **figure**; the logo is on the overlay as **noise** and is **not** a RAG figure. I toggle overlay, zoom to 150%, and the boxes stay on the diagram. I hide paragraphs and still see columns.

## Risks (product)

| Risk | Mitigation |
|------|------------|
| Overlay looks “AI guessed wrong” | Show confidence + source (L0/L1/L2); legend; fail-open L2 |
| Precision drop on small real charts | Start `MIN_IMAGE_AREA_FRAC=0.008`; G6 corpus lock |
| Layout slow on 200-page PDFs | Feature flag; CPU EP; persist L0/L1 without waiting if L2 times out (fail-open) |

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- UX: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
