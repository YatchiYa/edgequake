# 00 — Why SPEC-115

## Trigger

Product observation: EdgeQuake extracts **too many chunks** and therefore **too many entities / relations** on PDF documents, relative to LightRAG.

Binding corpus: the LightRAG paper PDF  
`papers/light_rag_2410.05779v3.pdf` (same content as `zz_test_docs/academic_papers/lighrag_2410.05779v3.pdf`).

## Why this pack exists (not only SPEC-108)

| Pack | Answers | Missing for this trigger |
|------|---------|--------------------------|
| **SPEC-108** | Why a partner saw ~12k “entities” (M≠U + adaptive) | No **live Mistral Small** dual-SUT on this paper; geometry used heuristic stride |
| **SPEC-026** | Broader EQ↔LR feature/algorithm parity | Does not measure PDF paper chunk N / KG yield |
| **SPEC-115** | **Chunk size law + live KG yield** on this PDF/gold with Mistral Small | — |

## Non-goals

- Re-audit full SPEC-026 matrix
- Change Acc publication pins or SPEC-001 scorecards
- Ship product UI / adaptive default changes in this mission (reco only)
- Vision OCR variance as primary confound (use gold MD for extract arms)

## Success criteria

1. **WHY** chain: from first principles → code SSOT → measured N → measured M/U under identical LLM.
2. Real LightRAG execution with **mistral-small-latest** (+ mistral-embed).
3. Real EdgeQuake execution with the **same** model pins (product adaptive arm + fair 1200/100 arm).
4. Protocol reproducible from `experiments/`; results in `measurements/` + `05-execution-report.md`.
5. Recommendation: when to pin fair chunking vs keep adaptive; how to read counts (U not vanity M).

## Constraint: code is law

If docs and code disagree, **code wins**. Paths cited must exist in:

- EdgeQuake: `edgequake/crates/edgequake-pipeline/...`
- LightRAG: `/Users/raphaelmansuy/Github/03-working/LightRAG/lightrag/...`
