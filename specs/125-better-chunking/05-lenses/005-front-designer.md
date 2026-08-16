# Lens 005 — Front Designer

## Visual

Do not add a second card. Extend the existing Chunking card with a muted one-line hint under the description, same `text-[11px] text-muted-foreground` as future-only.

```ascii
  [Scissors] Chunking                         [Fixed 1200/100]
  How documents are split …
  Markdown files pack small headings into the token budget.
  Applies to future document ingestions. Rebuild to re-chunk.
```

No new colors, no new icons. Do not show kill-switch env in the card (ops-only, like other fleet pins).

## Testid

`chunking-markdown-pack-hint` — Playwright asserts copy present on `/workspace` edit.
