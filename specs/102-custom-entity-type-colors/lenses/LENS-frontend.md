# LENS — Frontend (SPEC-102)

## Question

Is there exactly one color path for entity-type mode across graph surfaces?

## Verdict

Required: `resolveEntityTypeColor` + `useEntityTypeColors`; no private palettes.

## Anti-patterns

- Hardcoded hex in components  
- Tailwind type→class maps that diverge from hex SSOT  
- Applying overrides in community mode  

## Laws cited

LAW-102-1, LAW-102-4, LAW-102-5, LAW-102-6  
