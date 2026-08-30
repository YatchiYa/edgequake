# 09 — Edge Cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-01 | No page markers | Sync disabled; PDF stack works | E-143-08 |
| EC-02 | Content before first marker | Treat as page 1 (chunker parity) | U-143-01 |
| EC-03 | Cross-page chunk span | Deeplink / sync use `page_start` | E-143-07 + SPEC-135 |
| EC-04 | Rapid PDF↔MD scroll | Driver lock + 200ms settle | U-143-02 |
| EC-05 | Virtualized markdown | Prefer non-virtual path for docs with markers; or observe only mounted anchors | Manual + unit |
| EC-06 | Zoom while scrolling | Remeasure placeholder heights; keep active page | Unit / manual |
| EC-07 | Layout overlay + stack | Overlay only on active sheet | Component |
| EC-08 | URL vs local scroll race | Controller SSOT; URL follower unless external | E-143-02 |
| EC-09 | 1-page PDF | Stack of one; wheel no overflow page change | E2E smoke |
| EC-10 | `numPages > 20` | Windowed ±2; placeholders | Unit / manual large PDF |
| EC-11 | Duplicate marker N | First wins; later ignored for id uniqueness | U-143-01 |
| EC-12 | Marker N > numPages | Clamp | Unit |
| EC-13 | Dialog vs detail page | Shared hook + same testids | E2E one surface + unit wire |
| EC-14 | Sync OFF mid-jump | Cancel pending follower scroll | E-143-05 |
| EC-15 | MM fence + markers | Inject still works around fence | U-143-03 |

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
