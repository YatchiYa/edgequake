# Lens — Front Designer

## Visual system (reuse, don’t invent)

Match [`WorkspaceExtractionLanguageCard`](../../../edgequake_webui/src/components/workspace/workspace-extraction-language-card.tsx):

- Compact `Card` `gap-2 py-4`
- Lucide icon + indigo accent (`text-indigo-600`) — use `Scissors` or `SplitSquareVertical`
- `Badge variant="secondary"` for resolved mode
- `text-xs` / `text-[11px]` hints
- shadcn `Select` / segmented control / `Button` chip

## Motion

Minimal: chip press + toast on save (sonner rebuild hint). No hero animations on settings.

## Anti-looks

No purple gradient theme, no stat strips, no multi-card “chunking dashboard.”
