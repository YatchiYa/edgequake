# 02 — Cross-Reference Matrix

| Claim | Authority |
|-------|-----------|
| Adaptive thresholds 1200/800/600 | [`adaptive_chunking.rs`](../../edgequake/crates/edgequake-pipeline/src/adaptive_chunking.rs) |
| Acc fair env pin | SPEC-001 / `tools/bench001` `acc_env.py` |
| M vs U vanity | [SPEC-108](../108-extraction-compared-light-rag/) |
| Live Mistral density | [SPEC-115](../115-extraction-chunk-kg/) |
| Adaptive SSOT history | SPEC-025 |
| Extraction language metadata pattern | [SPEC-096](../096-multi-language-extraction/), `apply_extraction_language_metadata` |
| Wizard reconfigure | SPEC-101, `reconfigure-workspace-wizard.tsx` |
| Upload `chunk_options` | `document_admission.rs` `parse_upload_chunk_fields` |
| Caps 40/100 | `extract_caps.rs` / LightRAG `constants.py` |
| Per-chunk budget first principles | [`12-extract-budget-first-principles.md`](12-extract-budget-first-principles.md) |
| Extract budget brainstorm + phases | [`13-extract-budget-brainstorm.md`](13-extract-budget-brainstorm.md) |
| LightRAG configurable caps | [PR #2950](https://github.com/HKUDS/LightRAG/pull/2950), SPEC-001/054 |
| Ops lens extract budget | [`05-lenses/008-extract-budget.md`](05-lenses/008-extract-budget.md) |
| LLM power × yield × QA (first principles) | [`10-llm-power-first-principles.md`](10-llm-power-first-principles.md) |
| Aug 2026 research evidence pack | [`11-research-evidence-aug-2026.md`](11-research-evidence-aug-2026.md) |
| Stronger construction LLM → multi-hop QA | [arXiv:2502.11371](https://arxiv.org/abs/2502.11371) Table 5 (71.17→75.08 Overall) |
| Denoising / less-is-more | [arXiv:2510.14271](https://arxiv.org/abs/2510.14271) (~40% entity cut, QA↑) |
| Builder correctness ceiling ~68% | [CS-RAG HTML](https://arxiv.org/html/2603.14828) |
| Ops lens (Acc-fair vs upsize extract) | [`05-lenses/007-llm-power-research.md`](05-lenses/007-llm-power-research.md) |

## Code SSOT (target)

| Concern | Path |
|---------|------|
| Policy + resolve | `edgequake-pipeline/src/adaptive_chunking.rs` |
| Build chunker | `ingestion_pipeline.rs` `build_chunker_config` |
| Metadata apply | `edgequake-core/.../helpers.rs` |
| Create/update | `workspace_ops.rs` |
| Worker inject | `text_insert/prepare.rs`, `workspace_pipeline_factory.rs` |
| UI card | `workspace-chunking-card.tsx` |
| Extract caps SSOT | `edgequake-pipeline/src/prompts/extract_caps.rs` |

## DRY rule

Threshold math lives **only** in `adaptive_chunking.rs`. API/UI store mode + numbers; they never recompute 50KB/100KB bands.
