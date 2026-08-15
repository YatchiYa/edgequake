# Lens 005 — Front Designer

## Visual fit

Match existing Settings cards (ProviderStatus, Attribution): same border/radius/spacing tokens; Lucide `ExternalLink` for CTA; status pill colors from existing system (success/warning/muted).

## Do not

- Invent a purple Langfuse-branded marketing hero on Settings
- Cards-inside-cards for requirements — use checklist rows like Provider hub
- Animate the Open button continuously

## Component sketch

```ascii
  Card
   ├─ Header: title + status Badge
   ├─ Body: requirement rows (key, satisfied)
   ├─ Code block: env snippet (mono, copy)
   └─ Footer: secondary Copy | primary Open (if enabled)
```

## Testids

`langfuse-settings-card`, `langfuse-status`, `langfuse-open-link`, `langfuse-copy-env`

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
