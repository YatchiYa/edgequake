# Lens 005 — Front Designer

## Stake

Reuse existing upload chrome; no new dashboard surface.

## Composition

```ascii
  Documents page
    └─ dropzone
    └─ UploadProgressList (existing)
         └─ one row per file (name, %)
    └─ documents table (admit presence)
```

## Constraints

- No hero redesign; no new card grid for progress.
- Density: N≤20 rows scroll inside existing panel.
- Distinct `data-testid` for multi-PDF e2e hooks if missing (`document-dropzone-input` already exists).

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
