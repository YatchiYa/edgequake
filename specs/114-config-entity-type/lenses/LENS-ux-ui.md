# LENS — UX / UI (SPEC-114)

## Mental model

Users configure a **schema**, not two unrelated string lists. The Extraction step should answer:

1. What domain am I in?  
2. Which entity types?  
3. Which relation types?  
4. What will the graph vocabulary look like? (preview)

## Hybrid layout (wizard density)

Dialog shell: `sm:max-w-3xl` / `lg:max-w-5xl` (not `max-w-lg`) so dual panels breathe.

```ascii
┌─ Extraction (wide wizard) ──────────────────────────────────────────────┐
│ Language [▼……………]     Manufacturing · 12 entities · 6 relations [Domain]│
│ ┌─ Entity types ──────┐ ┌─ Relation types ────┐ ┌─ Visual schema ─────┐ │
│ │ chips (taller)      │ │ chips (taller)      │ │ type pills          │ │
│ │ + Add  ☑ Strict     │ │ + Add  ☑ Strict     │ │ A─REL→B edges       │ │
│ └─────────────────────┘ └─────────────────────┘ └─────────────────────┘ │
│ Applies to future extractions…                                          │
└─────────────────────────────────────────────────────────────────────────┘
md: 2-col panels + preview full-width below
lg+: 3-col (entity | relation | preview sidebar)
```

## Progressive disclosure

| Level | Content |
|-------|---------|
| Default | Domain summary + dual panels + collapsed/compact preview |
| Change domain | Preset cards (icons) |
| Bulk edit | Tab per panel (existing pattern) |
| v2 | Expand preview → canvas |

## Copy principles

- Empty relations: “No relation types — model may use free-form labels.”  
- Schema change: “Applies to future extractions. Rebuild KG to refresh existing graph.”  
- Do not say “server defaults” for relations when empty — say free-form.

## A11y

- Panels labeled; chips have remove accessible names.  
- Preview is supplementary — not the only way to understand lists.  
- Strict checkboxes have described-by helper text.
