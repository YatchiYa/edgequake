# Lens — UX / UI

## Composition

One card under Chunking on `/workspace`:

```ascii
  Extract budget
  ○ Inherit fleet (recommended)
  ○ Custom  [ents] [records]
  [ Match LightRAG (40/100) ]
  Hint: future ingestions · Rebuild KG
  Hint: denser N × saturate K → more M
```

## Progressive disclosure

Custom ints only when Custom selected. Preset chip sets Custom 40/100.

## A11y

- Radios named `extract-budget-mode`  
- Spinbuttons labeled  
- `data-testid`: `workspace-extract-budget-card`, `extract-budget-mode-inherit`, `extract-budget-mode-custom`, `extract-budget-entities`, `extract-budget-records`, `extract-budget-preset-lightrag`
