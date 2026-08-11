# 02 — Cross-Reference Matrix

| Claim | Authority |
|-------|-----------|
| Caps 40/100 law | [SPEC-001/054](../001-benchmark/001-edgquake-improvements/054-extract-caps-lr-parity.md), [`extract_caps.rs`](../../edgequake/crates/edgequake-pipeline/src/prompts/extract_caps.rs) |
| LightRAG defaults | `DEFAULT_MAX_EXTRACTION_ENTITIES=40`, `DEFAULT_MAX_EXTRACTION_RECORDS=100`; PR [#2950](https://github.com/HKUDS/LightRAG/pull/2950) |
| Budget first principles | [SPEC-116/12](../116-adaptive-chunking/12-extract-budget-first-principles.md) |
| Budget brainstorm phases | [SPEC-116/13](../116-adaptive-chunking/13-extract-budget-brainstorm.md) |
| \(M \approx K \times N\) | [SPEC-108](../108-extraction-compared-light-rag/) |
| Geometry co-design | [SPEC-116](../116-adaptive-chunking/) ChunkingPolicy |
| Workspace metadata pattern | SPEC-096 language, SPEC-116 chunking |
| Wizard | SPEC-101 |
| Gleaning wrapper | `extractor/gleaning.rs`, `json_gleaning_prompt` |
| Denoising “less is more” | [arXiv:2510.14271](https://arxiv.org/abs/2510.14271) |

## Code SSOT (target)

| Concern | Path |
|---------|------|
| Caps resolve | `prompts/extract_caps.rs` |
| Prompt limits + rank | `prompt_quantity_limits_section` |
| Apply hard truncate | `apply_extraction_caps` |
| Options inject | `IngestionPipelineOptions.extraction_caps` |
| Metadata apply | `edgequake-core/src/extract_budget_metadata.rs` |
| Workspace CRUD | `workspace_ops.rs` + DTOs |
| Doc admission | `document_admission.rs` |
| Worker | `prepare.rs`, `workspace_pipeline_factory.rs` |
| UI | `workspace-extract-budget-card.tsx` |

## DRY rule

Cardinality validation and resolve live **only** in pipeline/core helpers. UI stores ints; never reimplements precedence.
