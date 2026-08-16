# Lens 006 — AI Engineer

## Why extraction emits colliding names

Vision / LLM KG extraction on diagrams often labels edges and nodes with
**arrow glyphs transcribed into the name**:

```ascii
  SHADED_BOX -> CIRCULAR_TARGET
       becomes entity name fragment: SHADED_BOX_->CIRCULAR_TARGET

  margin value chains:  LEFT_MARGIN_VALUE_1->_00_->_+
```

That is reasonable for human-readable graph labels; it is hostile to an
unescaped `SRC->TGT` wire format.

## Options (ranked)

| Option | Pros | Cons | SPEC-133 stance |
|--------|------|------|-----------------|
| Index-guided parse | Fixes stored keys without re-extract | Multi-both-resolve rare ambiguity | **Ship now** |
| Escape / length-prefix keys | Invertible forever | Migrates Plane B; dual-read window | Follow-up LAW-133-9 |
| Prompt: forbid `->` in names | Reduces new collisions | Loses label fidelity; doesn't fix history | Optional later |
| Post-normalize strip arrows | Simple | Destroys meaning; merge collisions | Reject |

## Prompt / schema guidance (optional later)

If extraction schema is tightened:

- Keep visual arrow semantics in **description** or a `label` property.
- Keep `name` as a stable identifier without raw `->` / `:` characters.
- Document the rule next to SPEC-096 / extraction caps — not a silent rename.

## Eval

Add a fixture manuscript page (or synthetic extract JSON) with ≥1 target-arrow
relationship to Acc / ingest smoke so the class cannot regress silently.

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- Edges: [../10-edge-cases.md](../10-edge-cases.md)
