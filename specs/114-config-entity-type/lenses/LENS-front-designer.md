# LENS — Front Designer (SPEC-114)

## Visual direction

Stay inside the existing wizard / shadcn system. Do **not** introduce purple-on-white AI clichés, glow, or floating promo badges on the preview.

## Composition

| Element | Treatment |
|---------|-----------|
| Domain summary | Compact header: icon + name + “Change domain” text button |
| Dual panels | Equal-weight columns on `md+`; stacked on mobile |
| Chips | Existing Badge + remove; entity chips keep color swatches |
| Relation chips | Same chip chrome **without** color picker |
| Preview | Muted border panel; small node pills + edge labels; max height with scroll |
| Presets | Icon grid already in EntityTypeSelector — reuse for schema-level domain pick |

## Motion (intentional, light)

1. Preview fades/updates when lists change (short opacity).  
2. Domain switch cross-fades chip sets.  
3. Optional: preview edge labels stagger in (≤3 items visible).

## Density budget (SPEC-101)

Extraction step must remain completable without scroll-trap on laptop viewports: language block compact; domain collapsed by default after selection; preview ≤ ~140px tall.

## Tokens

Use existing CSS variables (`border`, `muted-foreground`, `primary` for Next/Apply only). Preview nodes use entity-type resolved colors when available (SPEC-102), else muted.
