# Lens 006 — RAG Expert

## Stake

Retrieval rankers see **chunk text + embedding**. 70 underfilled chunks on a
26k-token paper dilute BM25/vector hits (many near-duplicate figure snippets)
and explode extract-then-graph noise. LightRAG **F** on the same text is 24
windows — a pack-to-budget baseline, not a strategy we must clone byte-for-byte.

## What we match / what we do not

| LightRAG (Aug 2026) | EdgeQuake after 135 |
|---------------------|---------------------|
| `chunking_by_token_size` 1200/100 **F** | Packer **fill** of 1200, structure-aware cuts |
| `_build_mm_chunks_from_sidecars` when F never saw VLM | Sidecar **only if** not already inlined |
| No PDF page IR | Page markers as attribution; span allowed |
| Wizard “Match LightRAG” = F window | Wizard still pins **size** 1200/100 (SPEC-116); 135 makes Pdf **use** that size |

Product PDF will **not** be byte-identical to F. It should be in the same
**N-band and fill-band** as F on the trigger class, plus structure (tables
stay atomic, ATX prefixes).

## Citation

RAG answers cite chunks. If `page_end > page_start`, show the span; retrieve
deeplink to `page_start`. Do not invent a mid-span page.

## Non-goals (RAG)

- Late chunking (embed full doc, pool token spans)
- Semantic-V
- Changing Acc F on non-PDF
- Cross-encoder rerank changes

## Retrieval regression watch

After WP-2, spot-check: unique probe strings still retrieve; figure-only
duplicate hits drop. Formal Acc PDF is [12](../12-honest-assessment.md).

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- Research: [../11-research-evidence-aug-2026.md](../11-research-evidence-aug-2026.md)
