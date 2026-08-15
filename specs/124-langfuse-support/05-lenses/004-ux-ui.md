# Lens 004 — UX / UI Designer

## Job story

> When I wonder if Langfuse is on, I open Settings and see a clear status — not a scavenger hunt through env files.

## Principles

1. **Discoverable** — card always visible (like Provider status).
2. **Honest** — never show Open when export cannot work.
3. **Actionable** — copyable snippet + docs link; Open when ready.
4. **Calm** — one card, one primary CTA; no dashboard chrome in Settings.

## Microcopy

| Element | Copy |
|---------|------|
| Title | Langfuse Observability |
| Unconfigured | Set `LANGFUSE_PUBLIC_KEY` and `LANGFUSE_SECRET_KEY`, then restart with OTEL enabled. |
| Enabled | Traces export to Langfuse. |
| Misconfigured | Keys set but this binary was built without the `otel` feature. |
| CTA | Open in Langfuse |

## Cross-refs

- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front: [005-front-designer.md](005-front-designer.md)
