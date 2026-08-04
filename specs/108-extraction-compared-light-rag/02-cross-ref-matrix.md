# 02 — Cross-Reference Matrix (SPEC-108)

Finding IDs stable within this pack. Deep remediation / Acc science stay in cited specs.

## External anchors

| Spec / doc | What we reuse | What we do **not** fork |
|------------|---------------|-------------------------|
| [SPEC-026](../026-egdequake-vs-lightrag/) | C-01 extract parity A; C-03 chunking breadth; ingest/algorithm docs | Full feature grades |
| [SPEC-026 003-ingestion](../026-egdequake-vs-lightrag/003-ingestion/001-ingestion-comparison.md) | Pipeline topology | Adaptive product default narrative (added here) |
| [SPEC-001 / 029 ingest audit](../001-benchmark/001-edgquake-improvements/029-ingest-parity-audit.md) | `audit_eq_lr_ingest.py` signals | Acc promotion rules |
| [SPEC-001 / 054 extract caps](../001-benchmark/001-edgquake-improvements/054-extract-caps-lr-parity.md) | 40/100 law | Acc REJECT on B9 story |
| [SPEC-001 fair_pins](../../tools/bench001/bench001/fair_pins.py) | 1200/100, adaptive off | Publish scorecard |
| [SPEC-086 F-extraction](../086-improve-ingestion-ux/findings/F-extraction-quality-parity.md) | Density /1k chars | UI column shipping |
| [SPEC-096](../096-multi-language-extraction/) | Extraction language SSOT | Language mission |
| [SPEC-013 entity extraction](../013-fix-issues-05-2026/entity_extraction/) | Strict vs permissive types | Type taxonomy redesign |
| [SPEC-107](../107-issue/) | Partner pack style | SQLSTATE / INV-03 |

## LightRAG upstream (verified 2026-08)

| Constant | Value | Source |
|----------|------:|--------|
| `CHUNK_SIZE` | 1200 | [FileProcessingPipeline.md](https://github.com/HKUDS/LightRAG/blob/main/docs/FileProcessingPipeline.md), `api/config.py` |
| `CHUNK_OVERLAP_SIZE` | 100 | same |
| `DEFAULT_MAX_EXTRACTION_ENTITIES` | 40 | [constants.py](https://github.com/HKUDS/LightRAG/blob/main/lightrag/constants.py) |
| `DEFAULT_MAX_EXTRACTION_RECORDS` | 100 | same |
| `DEFAULT_MAX_GLEANING` | 1 | same |
| `DEFAULT_CHUNK_P_SIZE` | 2000 | P strategy only — not Acc R/F default |

Local reference checkout: `/Users/raphaelmansuy/Github/03-working/LightRAG`.

## Internal finding IDs (this pack)

| ID | Claim | Docs |
|----|-------|------|
| X-01 | Document `entity_count` is pre-dedup mention sum M | 01, 03, 05 |
| X-02 | Product adaptive ON shrinks large docs to 600 tok | 01, 03, 05 |
| X-03 | Fair pin EQ vs LR requires fixed 1200/100 | 02, 04 |
| X-04 | Partner ~12k is consistent with high N × yield, not proof of broken merge | 00-issue-data, 06, 07 |
| X-05 | Density (U or M per 1k chars) is the comparable metric | 01, 05, SPEC-086 |

## Conflict rule

| If… | Then… |
|-----|-------|
| SPEC-108 vs SPEC-026 on algorithm parity | **SPEC-026 + code** win |
| SPEC-108 vs Acc fair pins | **SPEC-001 / fair_pins** win for science claims |
| SPEC-108 vs partner UI reading | SPEC-108 LAW-X1 wins for “what the number means” |
