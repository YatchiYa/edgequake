# 11 — Research Evidence (August 2026)

Grounding for pack-to-budget, MM-once, and “contextual prefix without an LLM.”
Fetched 2026-08-23.

## Claim → evidence → EdgeQuake

| ID | Claim | Evidence | EQ meaning |
|----|-------|----------|------------|
| R1 | LightRAG default **F** is 1200 / 100 tiktoken | HKUDS `chunking_by_token_size` / `chunk_token_size=1200`, `chunk_overlap_token_size=100` ([token_size.py](https://github.com/HKUDS/LightRAG/blob/main/lightrag/chunker/token_size.py); [ProgramingWithCore.md](https://github.com/HKUDS/LightRAG/blob/main/docs/ProgramingWithCore.md)) | Wizard “Match LightRAG” already pins **size**. 135 makes PDF **fill** that size. |
| R2 | F/R/V that ignore section structure operate on concatenated `content` | [LightRAGSidecarFormat.md](https://github.com/HKUDS/LightRAG/blob/main/docs/LightRAGSidecarFormat.md): concatenating block `content` must equal the document | LightRAG **F** never “sees” page IR. EQ has page markers — we pack **with** structure, not clone F bytes. |
| R3 | MM sidecars exist so analysis is indexed when the token window never saw VLM | LightRAG `_build_mm_chunks_from_sidecars` (pipeline); EQ port in `chunks.rs` `append_mm_chunks_to_text` ([SPEC-047](../047-rag-evaluation/), [SPEC-026 P4](../026-egdequake-vs-lightrag/009-improvement-plan/phase-4/004-multimodal-parity-implementation-plan.md)) | Follow **intent** (index MM once). Pass-A inlined VLM ⇒ skip sidecar. |
| R4 | Nested MM chunk schema + per-chunk entity injection | LightRAG PR [#3064](https://github.com/HKUDS/LightRAG/pull/3064) (2026 multimodal refactor) | Do not double-inject the same drawing into extract. |
| R5 | Header-orphan / heading-hard split is a known failure | SPEC-125 R1 (LangChain MarkdownHeaderTextSplitter); measured PDF p50=230 | LAW-135-1,2 pack-with-neighbor |
| R6 | Deterministic heading prefix ≈ contextual retrieval minus LLM cost | [Anthropic Contextual Retrieval](https://www.anthropic.com/engineering/contextual-retrieval) (2024); title-chain prefixes [arXiv:2608.00824](https://arxiv.org/html/2608.00824v1); [DEV heading-aware headers](https://dev.to/kartikeyraj/free-contextual-chunk-headers-heading-aware-chunking-for-hybrid-retrieval-560) | Already LAW-125-4 ATX prefix. **Non-goal:** Haiku “situate this chunk.” |
| R7 | Late chunking is a different layer | Jina [arXiv:2409.04701](https://arxiv.org/abs/2409.04701) | Non-goal v1 |
| R8 | Tables: never mid-row; repeat header | SPEC-125 R3 / LAW-125-5 | Packer reuse |
| R9 | SPEC-125 explicitly left PDF on Recursive | 125 E10/E30, 09-acceptance “PDF still page-aware recursive” | **Reversed** by LAW-135-3 |
| R10 | Page columns exist but writers fill JSONB only | Migration 066; SPEC-128 DB lens; live FreeToken `page_start` NULL | LAW-135-9 |

## Bibliography

1. HKUDS LightRAG — `chunking_by_token_size` — https://github.com/HKUDS/LightRAG/blob/main/lightrag/chunker/token_size.py
2. HKUDS LightRAG — Programming with Core (chunk_token_size 1200/100) — https://github.com/HKUDS/LightRAG/blob/main/docs/ProgramingWithCore.md
3. HKUDS LightRAG — Sidecar format (F/R/V on concatenated content) — https://github.com/HKUDS/LightRAG/blob/main/docs/LightRAGSidecarFormat.md
4. LightRAG PR 3064 — multimodal nested chunks — https://github.com/HKUDS/LightRAG/pull/3064
5. Anthropic — Contextual Retrieval — https://www.anthropic.com/engineering/contextual-retrieval
6. Structure-aware title-chain prefixes — https://arxiv.org/html/2608.00824v1
7. Jina late chunking — https://arxiv.org/abs/2409.04701
8. SPEC-125 research pack — [../125-better-chunking/11-research-evidence-aug-2026.md](../125-better-chunking/11-research-evidence-aug-2026.md)
9. SPEC-047 / 026 — MM append vs Acc double-index
10. SPEC-033 — page lineage; equality invariant to amend

## Causal synthesis

```ascii
  LightRAG F 1200/100 ──► N≈24 on 26k tok ──► fill is the point
  EQ Pdf Recursive+atomic emit+MM copy ──► N=70, p50=230 ──► underfill
  Packer (125) on same MD ──► N=29 ──► already the right inner
  Inline VLM + sidecar ──► double extract ──► skip sidecar (intent, not clone)
  ATX prefix (125) ──► free contextual header ──► do not add Haiku prefixes in 135
```

## What 135 will **not** copy from LightRAG

- Byte-identical F windows (no page IR, no ATX pack)
- Mechanical `_build_mm_chunks_from_sidecars` when VLM is already in the markdown
- Semantic-V
