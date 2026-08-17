# LENS — Accessibility (SPEC-099)

## Question

Do StatusCell, collapsed upload, and demoted destructive actions remain operable and perceivable for keyboard and AT users?

## Requirements

| Area | Requirement |
|------|-------------|
| StatusCell | Accessible name includes pipeline status **and** fence (`Ready` / `Indexed, not yet queryable`); color not sole signal |
| Contrast | Composite cell and secondary fence meet WCAG 2.2 AA in light + dark |
| Collapsed upload | Focusable control; Space/Enter opens picker; drag-target announced; `data-collapsed` not `aria-hidden` on the only upload control |
| Clear All | Still reachable via keyboard from overflow; typed confirm focus trap correct; Escape cancels |
| Feedback zone | Scrollable region has accessible name (“Active runs”); Cancel buttons have unique names per file |
| Live regions | Prefer zone updates over duplicate toast polite announcements for the same session |
| Selection mode | Replacing toolbar must not strand focus; move focus to selection bar heading/count |
| Table | Sortable headers remain buttons; virtualized rows keep tab order sane |

## Anti-patterns to avoid in Waves 2–5

1. Hiding fence only visually while removing it from the accessibility tree (breaks SPEC-091 AT honesty).  
2. Collapsing upload by `display:none` on the drop input with no alternate control.  
3. Toast + zone both `aria-live` asserting conflicting states.  
4. Icon-only Clear All in overflow without accessible name.

## Laws

LAW-099-3 · LAW-099-4 · LAW-099-5 · LAW-099-6 · EC-099-12 · EC-099-13

## Verification

- Playwright: role/name asserts in `spec099-status-cell-fence`, `spec099-upload-collapse`, `spec099-clear-all-demoted`.  
- Manual: VoiceOver / keyboard pass on idle + busy (W8 checklist).
