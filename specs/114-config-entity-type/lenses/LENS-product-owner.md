# LENS — Product Owner (SPEC-114)

## JTBD

When I set up or reconfigure a workspace, I want to pick a **domain KG schema** (entity types + relation types) so extractions produce a coherent graph without teaching the model my vocabulary in free text.

## Success

| Metric | Target |
|--------|--------|
| Time to configure schema | &lt; 2 minutes via domain preset |
| Relation config available | 100% of create/reconfigure flows |
| Preset → Apply → reload fidelity | 100% round-trip |
| Support tickets "wrong relation labels" | Down after adoption |

## In / Out

| In (v1) | Out |
|---------|-----|
| Dual allowlists + domain presets | OWL/RDF, SHACL |
| Hybrid visual preview | Full ontology canvas (v2) |
| Strict mode for relations | Auto-infer from documents |
| Honest rebuild messaging | Silent graph rewrite on PUT |

## Narrative

```ascii
Pick domain → See types + relations + preview → Apply
        │
        ▼
"This workspace extracts Manufacturing vocabulary"
```

## Risks

- Over-strict relation lists → empty/poor graphs → default free-form when empty (LAW-114-3).  
- Preset drift → trust loss → LAW-114-4 parity.  
- Users expect existing graph to update → LAW-114-6 copy on review.
