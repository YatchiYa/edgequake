# LENS — Information Hierarchy (SPEC-099)

## Question

On `/documents`, what must win the first viewport — and what must defer?

## Verdict

| Rank | Surface | Idle | Busy |
|------|---------|------|------|
| 1 | Inventory table | Dominant (≥60% height) | Remains reachable below zone |
| 2 | Search / filter / sort | Always visible, compact | Same |
| 3 | Upload slot | Expanded hero when empty/idle | Collapsed drag-target |
| 4 | Feedback zone | Hidden when no live work | ≤35vh, sole live narrative |
| 5 | Working/Queued summary | Hidden | Compact header chips OK |
| 6 | Cost / NEW / parser detail | Secondary / defer | Same |
| — | Clear All | Demoted (overflow) | Demoted |

## ASCII — hierarchy

```ascii
IDLE first viewport                    BUSY first viewport
┌ 1 Inventory ─────────────────┐       ┌ 5 Chips · 2 Filters ──────────┐
│  (scan titles + status)      │       ├ 3 Upload collapsed ───────────┤
├ 2 Filters ───────────────────┤       ├ 4 Feedback zone (narrative) ──┤
├ 3 Upload (admit) ────────────┤       ├ 1 Inventory (compact status) ─┤
└ 6 Meta (cost/new) deferred ──┘       └ toast demoted ────────────────┘
```

## Anti-patterns (observed)

1. Upload band taller than a table row cluster while corpus is already populated.  
2. Two peer success pills stealing scan weight from title.  
3. Destructive Clear All sharing primary action weight with Refresh.  
4. Toast repeating what the zone already says.

## Laws

LAW-099-2 · LAW-099-3 · LAW-099-4 · LAW-099-5 · LAW-099-6

## Cross-ref

Findings F-099-02…F-099-06 · Issues `ISSUE-serving-fence-presentation`, `ISSUE-feedback-zone-density`, `ISSUE-upload-slot-collapse`, `ISSUE-destructive-action-hierarchy`
