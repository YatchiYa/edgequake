# 01 — First Principles

## Axioms

1. \(K\) bounds **one LLM response**, not unique graph size \(U\).  
2. Soft prompt without hard truncate is insufficient (models over-extract).  
3. When the model saturates \(K\), \(M \approx K \times N\) — co-design with SPEC-116 geometry.  
4. Raising \(K\) to chase card \(M\) usually adds noise (DEG-RAG).  
5. Fair dual-SUT requires matched \(K\) (and \(N\), gleaning, model).

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-117-1** | \(K\) is a per-response budget, not a global graph quota |
| **LAW-117-2** | Precedence: document API > workspace > fleet env > 40/100 |
| **LAW-117-3** | Default = inherit (null metadata) |
| **LAW-117-4** | Soft prompt + hard truncate both required |
| **LAW-117-5** | Do not raise \(K\) to chase vanity \(M\); co-design with \(N\) |
| **LAW-117-6** | One SSOT resolve in `edgequake-pipeline` |
| **LAW-117-7** | Future ingestions only + Rebuild KG honesty |
| **LAW-117-8** | Truncation observable; selection under \(K\) beats blind FIFO alone |

## Causal diagram

```ascii
  text → ChunkingPolicy → N chunks
              │
              ▼
  Extract LLM (soft K + rank) ──► parse
              │
              ▼
  Hard truncate (relation-aware; fifo via env)
              │
              ├──extract_caps_applied?──► yes + gleaning left
              │                                         │
              │                                         ▼
              │                              continue: additional ents
              ▼
         Mentions M ≤ K×passes → merge → U → QA
```

## Acc / LR identity

```text
Inherit (null)  ≡  EDGEQUAKE_MAX_EXTRACTION_ENTITIES=40
                   EDGEQUAKE_MAX_EXTRACTION_RECORDS=100
                   (or unset → same defaults)

Preset chip     ≡  workspace keys 40 / 100

Acc FIFO pin    ≡  EDGEQUAKE_EXTRACT_CAPS_SELECTION=fifo
                   (product default: relation_aware)
```
