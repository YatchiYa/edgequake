# SPEC-108 — Extraction Density vs LightRAG

> **Trigger:** Partner sees ~12 367 entities / ~12 407 relations on one ingest and suspects chunking.  
> **Method:** First principles + EQ↔LightRAG code compare + dual-SUT geometry/extract arms.  
> **Not a fork of** [SPEC-026](../026-egdequake-vs-lightrag/) (full product audit) or [SPEC-001](../001-benchmark/) (Acc).

## Partner question

> J’avais une question concernant le paramétrage du parsing.
> Je viens d’ingérer un document et je remarque qu’il y a 12 367 entités et 12 407 relations.
> Je pense que cela pourrait être lié à un problème de chunking.

**Short answer:** Oui, lié au chunking **et** à la métrique affichée. Le compte document = **mentions pré-dédup (M)**, pas les nœuds uniques du graphe (U). Le défaut produit **adaptive** réduit les gros docs à 600 tokens → plus de chunks → M gonfle. LightRAG reste à 1200/100 fixe. Détail: [07-partner-reply.md](07-partner-reply.md).

## Status board

| ID | Hypothesis | Verdict | Evidence |
|----|------------|---------|----------|
| H1 | Metric illusion (M ≠ U) | **Primary** | [05](05-execution-report.md), [06](06-root-cause-ranking.md), mock M/U≈3.3 |
| H2 | Adaptive geometry inflates N→M | **Confirmed** | S2 B/A≈1.99; N_B=317 fits ≥309 for M=12k |
| H3 | Strategy geometry (Pdf vs R) | Secondary | Code + protocol |
| H4 | Merge gap vs LightRAG | Monitor | Acc audit EQ 3950 / LR 3580 |
| H5 | True over-extract | Not required | Caps 40/100 LR-parity |

## Document map

```ascii
 00-why / 00-issue-data
   → 01-first-principles (LAW-X1..X5)
   → 02-cross-ref-matrix
   → 03-code-comparison
   → 04-execution-protocol
   → 05-execution-report
   → 06-root-cause-ranking
   → 07-partner-reply
   → measurements/
```

## Sample documents

| Role | Path |
|------|------|
| Primary | `zz_test_docs/academic_papers/lighrag_2410.05779v3.pdf` (+ gold MD) |
| Secondary | GraphRAG-Bench medical (one doc) when Acc freeze present |
| Partner private | Optional slot-in — not in this pack |

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-026](../026-egdequake-vs-lightrag/) | Full EQ↔LR audit — do not re-grade features |
| [SPEC-001 / 029](../001-benchmark/001-edgquake-improvements/029-ingest-parity-audit.md) | Dual-SUT ingest count audit tooling |
| [SPEC-001 / 054](../001-benchmark/001-edgquake-improvements/054-extract-caps-lr-parity.md) | Extract caps 40/100 |
| [SPEC-086 F-extraction](../086-improve-ingestion-ux/findings/F-extraction-quality-parity.md) | Density ≠ vanity absolute counts |
| [SPEC-096](../096-multi-language-extraction/) | Extraction language |
| [SPEC-013](../013-fix-issues-05-2026/entity_extraction/) | Entity-type policy |

## DRY rule

Deep algorithm parity text lives in SPEC-026. Acc fair pins live in SPEC-001 / `tools/bench001`. SPEC-108 **cross-refs** and adds extraction-density laws + partner-facing measurement. If packs disagree on algorithm truth, **SPEC-026 / code win**.
