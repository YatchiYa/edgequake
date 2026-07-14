# 005 — Lens: UI Designer

**Job:** visual hierarchy that makes one active run obvious  
**Constraint:** work inside existing Documents layout (no brand rewrite)

---

## 1. Hierarchy (target Documents viewport)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ Workspace · Documents (N)                          [Live•] [Working|…]  │  ← H1 status
├──────────────────────────────────────────────────────────────────────────┤
│ ▌ WORKING · areal_2807….pdf · Extracting · chunk 42/351 · ~12m left    │  ← H2 banner
│   [Open run]                                              Details →     │
├──────────────────────────────────────────────────────────────────────────┤
│ Search · Filters · Sort                                                 │
│ ┌ dropzone ───────────────────────────────────────────────────────────┐ │  ← H3 intake
│ │  Drop files · Parser: Workspace Default                             │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
│ ┌ Active runs (1) ────────────────────────────────────────────────────┐ │  ← H3 runs
│ │  PDF  areal_…  ●●●●●○○  Extracting  ████████░░  42/351            │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
│ ┌ Table ──────────────────────────────────────────────────────────────┐ │  ← H4 archive
│ │  Title · Status · Entities · Cost · Created · Updated · ⋮           │ │
│ │  … Completed rows muted while a run is active                       │ │
│ └─────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

**Rule:** At most **one primary motion** (active run bar). Completed rows are quiet.

---

## 2. Status chrome

| State | Color role | Motion |
|-------|------------|--------|
| Working | Accent / info | Subtle pulse on active step only |
| Queued | Neutral | Static |
| Stuck | Warning | No pulse; CTA |
| Completed | Success | Check; no animation |
| Failed / partial | Danger / warning | Static + icon |

Avoid: purple-on-white AI cliché; neon glow; pill spam in hero.

---

## 3. Stepper design (server-aligned)

```text
  Upload  Convert*  Prep  Chunk  Extract  Glean  Merge  Sum  Embed  Store  Done
    ✓        ✓       ✓      ✓       ●       ○      ○     ○     ○      ○     ○
                                    └─ active: ring + label under
```

Compact chrome may collapse Glean/Sum into Extract/Merge when space-constrained,
but the **data model** still uses full `UnifiedStage` (code is law).

\* Convert only for PDF. Skip = muted dash, not error.

**Upload-only strip** (pre-`track_id`): keep Reading → Uploading; then **morph** into server stepper (don’t keep a parallel 4-step legend).

---

## 4. Density

| Zone | Density |
|------|---------|
| Banner | One line + optional expand |
| Active run card | Medium — stage + bar + N/M |
| Table | Compact — badge + one subline max |
| Dialog | Full timeline + tasks + cancel |

---

## 5. Attention to detail checklist

- [ ] Truncate filenames mid-string with tooltip full name  
- [ ] `stage_progress` is 0–1 → display as % only when determinate  
- [ ] Align entity/cost columns tabular nums  
- [ ] “NEW” badge expires (don’t compete with Working)  
- [ ] Pipeline Busy pill width stable (no layout jump Queued↔Busy)  
- [ ] Dark/light: contrast AA on stage labels  

Cross-ref: [010 components](./010-components-navigation-ascii.md)
