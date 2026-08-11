# Lens 004 — UX / UI

## Honesty rules

1. Never label a mode “Resolves to Vision” if Auto can switch to EdgeParse.
2. Inherit options must show the **true** resolved choice: Vision / EdgeParse / Auto.
3. After upload, detail view shows effective method; if Auto rewrote, show a short note.

## Upload control

```ascii
  Parser for this upload
  [ Workspace Default (Vision) ▼ ]   ← when cascade yields vision
  [ Workspace Default (Auto)   ▼ ]   ← when cascade yields auto
  [ Vision ] [ EdgeParse ] [ Auto ]
```

## Settings control

Workspace / Tenant:

- Server Default (shows resolved leaf)
- Vision
- EdgeParse
- Auto
- Clear / inherit (none)

## Large PDF admission

- Dialog applies override **only to large files**.
- Non-large files keep the dropzone selection.
- Copy: “Applies to N large PDFs; M other files keep current parser.”

## Empty / error states

- Invalid backend string → treat as unset + surface warning (no silent wrong method).
