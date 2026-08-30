# Lens — AI Engineer

## Role of AI in this feature

PDF→Markdown page markers are produced by the vision / EdgeParse assembly path
(SPEC-083 / SPEC-134). SPEC-143 **consumes** that metadata; it does not call
an LLM at view time.

## Invariants the AI pipeline must keep

| Invariant | Why sync needs it |
|-----------|-------------------|
| Every extracted page emits `<!-- edgequake-page:N -->` | MD anchors |
| Empty pages still get a marker | Page index alignment with PDF |
| Mixed-modality stitch preserves order | SPEC-134 stitch by markers |
| MM sidecar restamps markers | Avoid inheriting last doc page |

## Out of scope for SPEC-143

- Improving OCR / vision quality.
- Hallucinated page numbers in Query answers (SPEC-142).
- Training a model to align paragraphs.

## Failure modes affecting sync

| Failure | User impact | Mitigation |
|---------|-------------|------------|
| Missing markers (legacy) | No MD↔PDF sync | LAW-143-6 degrade |
| Duplicate / out-of-order markers | Wrong section | Prefer first occurrence of N; log soft warn |
| Marker N > PDF numPages | Scroll no-op | Clamp to numPages |

## Observability (optional)

- FE: count of injected anchors vs `numPages` (dev metric / console debug).
- No new LLM metrics required.

## Cross-refs

- Markers: SPEC-083 X-13
- Vision stitch: SPEC-134
- Laws: [01-first-principles.md](../01-first-principles.md)
