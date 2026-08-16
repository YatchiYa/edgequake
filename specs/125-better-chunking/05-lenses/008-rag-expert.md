# Lens 008 — RAG Expert

## Retrieval geometry

Heading-hard split maximizes **recall of titles** and destroys **precision of facts**. Packed chunks:

- Improve dense retrieval (more signal per vector)
- Improve FTS (heading + body co-located)
- Reduce extract \(N\) (cost and merge noise)

Tradeoff: a packed chunk mixing three `###` topics is less surgical for “only the first child.” Overlap and parent ATX still allow a query that names the parent plus a child topic to match. For heading-dense notes the mix is correct; for 50-page manuals the budget (600–1200) still splits oversized sections.

## Table RAG

A row without a header is not a record. Header repetition is mandatory on overflow (industry default 2026). Do not keep a 10k-token table as one chunk (embedder window / extract cap) — C-16 still splits; we only add header repeat.

## GraphRAG interaction

Fewer, denser chunks → fewer extract calls → \(M\) down, unique \(U\) not necessarily down (SPEC-115: \(M\) tracks \(N\)). Packing heading-dense notes is **anti-vanity-\(M\)**. Acc Recursive path unchanged so dual-SUT stays fair.

## Observability

Support must see `token_min` near 0 to catch residual orphans. Heading-dense packed: min ≈ max ≈ full-note tokens; `orphan_heading_chunks=0`.
