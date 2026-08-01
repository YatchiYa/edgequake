# 05 — Edge Cases (SPEC-099)

| ID | Scenario | Expected | Laws |
|----|----------|----------|------|
| **EC-099-01** | Short viewport (height ≤800px) with 1 active run | Upload collapsed; zone ≤35vh; ≥1 table row visible without scrolling past fold if possible | LAW-099-4 |
| **EC-099-02** | 20 concurrent queued/working runs | Zone scrolls internally; table remains reachable; no toast storm | LAW-099-2/4/6 |
| **EC-099-03** | Delete mid-upload (same doc id) | Delete session owns narrative; never flash Completed/Ready; upload toast demoted | LAW-099-6 · LAW-098-10 |
| **EC-099-04** | `query_ready=false` after Completed (Indexed fence) | StatusCell shows secondary Indexed-not-queryable; not a second green Ready | LAW-099-3 |
| **EC-099-05** | `delete_failed` terminal | Badge/filter honest; failed highlight uses domain; Retry ≠ reprocess | LAW-099-1 · LAW-098-11 |
| **EC-099-06** | Empty corpus | Expand dropzone is hero; no Clear All prominence; empty inventory state | LAW-099-4/5 |
| **EC-099-07** | Single-file upload, Active runs not yet painted | Brief toast OR admission row OK; once zone owns id, toast demotes | LAW-099-6 |
| **EC-099-08** | Reprocess cleaning + Active runs dual (SPEC-086) | ProgressPanelRow queues until ActiveRuns paints; no duplicate steppers in table | LAW-099-2 |
| **EC-099-09** | Stuck pipeline banner + active runs | Stuck banner may remain (operator signal); non-stuck processing banner demoted | LAW-099-2 · F-099-14 |
| **EC-099-10** | Corpus > `VIRTUAL_PAGE_SIZE` (100) | UI shows overflow / N of M; filter counts do not claim full corpus | LAW-099-7/8 |
| **EC-099-11** | Filter to Failed while Working chips show | Header count matches filtered rows; Working header pills are global status, not filter lie | LAW-099-8 |
| **EC-099-12** | Keyboard-only upload after collapse | Collapsed slot still focusable; Enter/Space or explicit Upload opens file picker; drag-target retained | LAW-099-4 · a11y |
| **EC-099-13** | Dark mode StatusCell | Composite cell contrast ≥ WCAG AA; fence secondary not low-contrast gray-on-gray | LENS-accessibility |
| **EC-099-14** | Selection of 50 rows + active runs open | Selection toolbar replaces search row; zone still capped; bulk delete honesty intact | LAW-099-9 · LAW-098-10 |
| **EC-099-15** | `cancelling` / `held` / `dead_letter` statuses | Domain recognizes them; badge presentation map covers icons; merge ranks coherent | LAW-099-1 |
| **EC-099-16** | Scroll inventory with ≥20 docs (viewport ~800px) | Chrome (title/filters/dropzone) stays pinned; table scrolls internally; `window.scrollY===0`; no white spacer band below virtual rows | LAW-099-4 |
| **EC-099-17** | Hard refresh while Active run in flight | Feedback zone reserved (skeleton or prior hint) before list paints; inventory Y does not jump; soft Refresh keeps placeholder list | LAW-099-4 |

## ASCII — EC-099-02 viewport

```ascii
┌─ Header (Working · N) ─────────────────────────────┐
├─ Search / filters / collapsed upload ──────────────┤  ~80px
├─ Feedback zone (scroll) max 35vh ──────────────────┤
│  run 1 … run 20 (compact cards)                    │
├─ Inventory table (flex-1) ─────────────────────────┤
│  rows visible                                      │
└────────────────────────────────────────────────────┘
         toast: suppressed (zone owns session)
```

## ASCII — EC-099-03 delete mid-upload

```ascii
 Upload session admits doc D
   → zone shows Uploading/Queued for D
     → user deletes D
       → deletion-session pin + AdmissionPhaseRow (Deleting)
         → table dimming session-driven
           → FORBIDDEN: Completed · Ready flash
```
