# Lens — Front-End Engineer

## Patterns to copy

- `workspace-chunking-card.tsx`  
- `workspace-config-diff.ts` / `model-payload.ts`  
- Wizard extraction step subsection  

## Types

```ts
extract_max_entities?: number | null;
extract_max_records?: number | null;
```

Null/omit = inherit. Payload clears by omitting keys (same as chunking).

## Constants

```ts
EXTRACT_BUDGET_LIGHTRAG = { entities: 40, records: 100 }
```

## No client-side precedence

UI never computes doc > workspace > env. Display stored workspace values only.
