# Lens — UX / UI Designer

## One composition

`/workspace` gets **one** new card — not a settings dashboard dump.

```ascii
  ┌─────────────────────────────────────────────┐
  │  ⊞ Chunking                                 │
  │  How documents are split before extraction  │
  │  Future ingestions only · Rebuild KG …      │
  │                                             │
  │  (○ Inherit  ● Adaptive  ○ Fixed)           │
  │  [ Match LightRAG (Acc fair) ]              │
  │                                             │
  │  Fixed → Size [1200]  Overlap [100]         │
  │  Adaptive → “1200→800→600 by document size” │
  └─────────────────────────────────────────────┘
```

## Rules

- Progressive disclosure: size/overlap only when Fixed
- Acc-fair chip is primary CTA for parity seekers
- Read-only badge when not editing
- `data-testid` on card, mode, chip, fields, hint
- Pair with SPEC-108 honesty: do not claim “entities” without M vs U context in helper text
