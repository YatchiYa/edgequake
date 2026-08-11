# Lens — LLM Power × Research (Product / Ops)

> Companion to [`../10-llm-power-first-principles.md`](../10-llm-power-first-principles.md) and  
> [`../11-research-evidence-aug-2026.md`](../11-research-evidence-aug-2026.md).

## Job story

> As a workspace admin, I want to know whether to **pin Acc-fair chunking**, **upsize the extract LLM**, or **rebuild/merge**, so research quality improves without chasing vanity entity counts.

## Decision tree

```ascii
  Partner / Acc: “too many / too few entities” or “weak multi-hop answers”
           │
           ▼
  Is geometry Acc-fair (Fixed 1200/100) for this workspace?
      │ no ──► SPEC-116 chip “Match LightRAG (Acc fair)” + Rebuild KG
      │
      yes
      │
      ▼
  Is the complaint about card M vs unique U / Acc?
      │ vanity M ──► educate (SPEC-108); judge U + Acc
      │
      real multi-hop QA weak
      │
      ▼
  Upsize extract LLM (LAW-P2) OR tighten schema / gleaning / resolve (LAW-P4)
      │
      ▼
  Re-measure Acc / partner questions — not raw M
```

## Requirements

| ID | Requirement |
|----|-------------|
| LP-1 | Acc-fair geometry before model bake-offs |
| LP-2 | Document that denser \(U\) ≠ better research (DEG-RAG) |
| LP-3 | Prefer extract upsize for multi-hop QA gaps after fair pin |
| LP-4 | Treat local JSON/schema failures as decoding, not “model too small” alone |
| LP-5 | Cross-link SPEC-114 schema + Rebuild KG on policy change |

## Non-requirements

- Changing fleet adaptive default  
- Automatic model routing by entity count  
- Replacing LLM extract with SpaCy in product (research-only note in `11`)

## Messaging (copy seeds)

- **Geometry:** “Chunking controls how many pieces we extract from. Match LightRAG uses Fixed 1200/100.”  
- **Model:** “A stronger extract model can improve multi-hop answers; it does not guarantee more useful unique entities.”  
- **Honesty:** “Document card counts are mentions before merge. Graph unique counts are what research walks.”

## Success metric

Ops/partner can explain a density or QA gap as **\(N\) vs \(y\) vs merge** in one sentence, and pick Acc-fair vs extract upsize correctly on first try.
