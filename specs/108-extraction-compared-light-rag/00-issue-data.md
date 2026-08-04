# 00 — Issue Data (Partner Question)

> Source: user message (French) on parsing / extraction counts after single-document ingest.  
> No production SQL dump attached — this is a **product-metrics / chunking** question, not SPEC-107 SQLSTATE.

## Original (French)

> J’avais une question concernant le paramétrage du parsing.
>
> Je viens d’ingérer un document et je remarque qu’il y a 12 367 entités et 12 407 relations.
>
> Je pense que cela pourrait être lié à un problème de chunking.

## English extract

| Signal | Value | Partner interpretation |
|--------|------:|------------------------|
| Document entities | 12 367 | “Too many — chunking bug?” |
| Document relations | 12 407 | Nearly 1:1 with entities |
| Suspected cause | Chunking / parsing params | |

## Back-of-envelope (pre-measurement)

| Bound | Math | Implication |
|-------|------|-------------|
| Min chunks @ max extract | 12 367 ÷ 40 ≈ **309** chunks | Caps = 40 ents/response (LR parity) |
| Adaptive large-doc size | **600** tokens/chunk (doc >100 KB bytes) | Product default `EDGEQUAKE_ADAPTIVE_CHUNKING` on |
| Fair Acc / LR default | **1200** / overlap **100** | Fewer chunks for same text |
| Near 1:1 ent:rel | 12367 ≈ 12407 | Consistent with per-chunk extract yield, not unique graph density |

## What we do **not** know from the message

- Document bytes / format (PDF vs MD)
- Whether counts are UI document card vs workspace graph stats
- LLM provider / gleaning / adaptive env on their deployment
- Unique AGE node count for the same document

SPEC-108 measures public samples under controlled arms to rank hypotheses; partner private doc can slot into the same protocol later.
