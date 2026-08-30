# Lens — Front Designer

## Visual language

Stay inside the existing document viewer chrome. Do not invent a new carded
dashboard. Sync is a single toolbar control beside the existing view-mode
icons.

## Chrome additions

```ascii
  [ PDF | Side | MD ]   [ Link sync ]   … existing resize …
         view modes      FEAT0733
```

- Sync icon: `Link2` / `Link2Off` (lucide) — muted when off.
- Sticky MD page hint: small text at top of markdown scroll, not a floating card.
- PDF page sheets: subtle vertical gap between pages (existing muted bg).

## Testids (stable)

| Element | testid / attr |
|---------|---------------|
| PDF indicator | `pdf-page-indicator` + `data-page` |
| PDF sheet | `pdf-page-sheet` + `data-page` |
| Sync toggle | `pdf-md-sync-toggle` + `data-sync` |
| MD anchor | `data-eq-page` |
| MD page hint | `md-page-indicator` |

## Responsive

- Sync toggle only meaningful in side-by-side; hide or disable in single-pane modes.
- Mobile stack: sync still applies when both panes mount sequentially.

## Cross-refs

- UX: [004-ux.md](004-ux.md)
- Spec: [06-ux-ui-spec.md](../06-ux-ui-spec.md)
