# 009 — Screens (ASCII): Current vs Target

---

## A. CURRENT — Documents (as observed / code)

```text
┌─ Workspace / SPEC-047 smoke ──────────────────────── API v0.18  ☀ 🌐 ⚙ ─┐
│ Documents  10 ●                                              [Pipeline Busy] │
│                                                          [Refresh] [Clear] │
├────────────────────────────────────────────────────────────────────────────┤
│ [Search……………]  [All Status (10) ▾]   Created ▲   Updated                   │
├────────────────────────────────────────────────────────────────────────────┤
│ ℹ Processing 1 document(s) — areal_….pdf: Extracting entities…  Details → │
│   ▲ free-text stage_message; may disagree with table                       │
├────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Dropzone ─────────────────────────────────────────────────────────────┐ │
│ │  Drag & drop · TXT MD JSON PDF · ≤50MB · Parser: Workspace Default     │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│ Processing Files:  Reading → Uploading → Extracting → Done                 │
│   ▲ CLIENT-ONLY 4 steps ≠ server UnifiedStage                              │
│ areal_2807.01120v2.pdf  ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░  [×]    │
│   ▲ may call GET /ingestion/{id}/progress → 404                            │
├────────────────────────────────────────────────────────────────────────────┤
│ ☐ Title              Status      Entities  Cost     Created      Updated   │
│ ☐ other.pdf          Completed✓   7236    $0.488   NEW 30m      21m       │
│ ☐ …                  Completed✓   …       …        …            …         │
│ ☐ areal_….pdf        Completed✓   …       …        …            …         │
│   ▲ row can show Completed while banner/Busy say Working (DEF Busy skew)   │
└────────────────────────────────────────────────────────────────────────────┘
```

**Pain points on this screen**

1. Three stories: Busy pill · banner · row  
2. Upload stepper vocabulary ≠ pipeline stages  
3. Completed rows compete visually with active work  
4. Progress bar without trustworthy N/M  

---

## B. TARGET — Documents (SPEC-048)

```text
┌─ Workspace / SPEC-047 smoke ──────────────────────── API v0.18  ☀ 🌐 ⚙ ─┐
│ Documents  10 ●                                    [Working · 1 run] [↻][🗑]│
│                                                      ▲ from PipelineActivity │
├────────────────────────────────────────────────────────────────────────────┤
│ [Search] [Status ▾]  Created ▲  Updated                                      │
├────────────────────────────────────────────────────────────────────────────┤
│ ▌ WORKING  areal_2807….pdf                                                   │
│   Extracting entities · chunk 42 / 351 · mode: full · ~minutes               │
│   [Open run]                                                    Details →    │
│   ▲ same IngestionRunView as row + active card                               │
├────────────────────────────────────────────────────────────────────────────┤
│ ┌ Dropzone (quiet when Working) ───────────────────────────────────────────┐ │
│ │ Drop files · Parser: Workspace Default                                   │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
│ ┌ Active run ──────────────────────────────────────────────────────────────┐ │
│ │ Up ✓ Conv ✓ Prep ✓ Chunk ✓ Extract ● Glean ○ Merge ○ … Store ○ Done ○  │ │
│ │ ████████████░░░░  42/351 chunks · last tick 3s ago                       │ │
│ └──────────────────────────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────────────────────┤
│ ☐ Title         Status              Entities  Cost    Created   Updated    │
│ ☐ areal_….pdf   Extracting · 42/351   —       $0.12   2m        just now  │
│ ☐ other.pdf     Completed ✓          7236    $0.488  30m       21m        │
│   ▲ muted when not active; active row uses RunView overlay                 │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## C. TARGET — Run detail dialog

```text
┌─ Ingestion run · areal_2807.01120v2.pdf ──────────────────────────── [×] ─┐
│ track_id: 9f3a…   document_id: …   mode: full                              │
│                                                                            │
│ Timeline                                                                   │
│  14:02:01  queued                                                          │
│  14:02:03  uploading          ✓                                            │
│  14:02:05  converting         ✓  pages 117/117                             │
│  14:02:08  preprocessing      ✓                                            │
│  14:03:12  chunking           ✓  chunks 351                                │
│  14:03:40  extracting         ●  42/351   ████████░░                       │
│  —         gleaning           ○                                            │
│  —         merging            ○                                            │
│  —         embedding          ○                                            │
│  —         storing            ○                                            │
│                                                                            │
│ Tasks                                                                      │
│  extract-batch-3   running                                                 │
│                                                                            │
│ [Cancel run]                                              [Copy track_id]  │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## D. Soft-reprocess modes (badge)

```text
  mode=full     →  Re-extract + merge
  mode=entities →  Re-extract entities only
  mode=merge    →  Reuse snapshot · merge only   (SPEC-047 P7e)
```

Cross-ref: [005 UI](./005-lens-ui-designer.md) · [010 components](./010-components-navigation-ascii.md)
