# 00 — Why SPEC-108

## Trigger

A user asked about parsing parameters after ingesting one document and seeing **~12 367 entities** and **~12 407 relations**, suspecting a **chunking** problem.

## Why not only SPEC-026?

| Pack | Audience | Job |
|------|----------|-----|
| **SPEC-026** | Engineering | Full EQ↔LightRAG comparative audit (algorithms, query, features) |
| **SPEC-001** | Acc science | Fair dual-SUT HybridRAG scorecard |
| **SPEC-108** | Partner + eng | Answer “why so many entities?” with count laws, chunk geometry, and measured arms |

SPEC-026 already grades extract parity **A** and notes chunking breadth. It does **not** explain document-card vanity counts or product-default adaptive sizing as the partner lens.

## Non-goals

- Re-audit full SPEC-026 feature matrix
- Change Acc publication ingest pins
- Ship UI “entities per 1k chars” in this mission (follow-up if H1 wins)
- Partner private document without provided bytes
- Product code fixes here — docs + measurements only; fixes are a follow-up SPEC if ranked

## Success

Partner can distinguish **M (mentions)** vs **U (unique graph nodes)**, see how adaptive chunking multiplies M, and compare fair-pinned EQ to LightRAG without confusing metrics.
