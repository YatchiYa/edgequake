# 00 — WHY (SPEC-099)

Evidence screenshots: [`evidence/01-idle-completed.png`](evidence/01-idle-completed.png) · [`evidence/02-busy-active-runs.png`](evidence/02-busy-active-runs.png) · [`evidence/03-legacy-active-card.png`](evidence/03-legacy-active-card.png).

## Symptom A — Dual success pills (Completed + Ready)

Idle Documents table shows every finished row as two peer green pills: **Completed** and **Ready**. Operators read “done twice.” The serving fence (`query_ready`) is a real SPEC-091 signal, but presentation treats it as a second primary success badge.

### Five WHYs (A)

1. Why do users see two green pills? `EnhancedStatusBadge` renders `StatusBadge` then `ServingFenceBadge` as siblings.  
2. Why is Ready a sibling? SPEC-091 IS3 / LD-09 required a visible fence when `query_ready` is projected.  
3. Why peer styling? Both use Badge chrome with emerald success affordance — no secondary hierarchy.  
4. Why wasn’t presentation refined? Fence truth shipped before a StatusCell composition pass; audits (SPEC-030 F-DOC) noted noise, not fence.  
5. Why harden now? v0.22 corpus views amplify dual-pill noise on every completed row; delete honesty (SPEC-098) already fights Completed/Ready flash mid-delete.

### Causal chain (A)

```ascii
 query_ready boolean on document shell
   → ServingFenceBadge always mounts next to StatusBadge
     → both emerald / success-weighted
       → scan path: "Completed" AND "Ready" = dual success
         → fence meaning (queryable vs indexed) is lost
```

---

## Symptom B — Triple narrative / viewport starvation

Busy upload (evidence 02): **Active runs** cards narrate Admit→Materialize, the **table** still shows Queued/Uploading badges for the same docs, and a **toast** says “Uploading N file(s)…”. Feedback zone is capped at 35vh but dropzone remains reserved; inventory is pushed below the fold.

### Five WHYs (B)

1. Why three narratives? Upload toast (`use-file-upload`), ActiveRunsPanel, and table status cells each own a slice of the same session.  
2. Why does toast stay? Toast is fire-and-forget loading feedback; it does not demote when feedback zone lists the same files.  
3. Why does dropzone still cost height? `quiet` densifies padding/copy but does not collapse to a drag target.  
4. Why wasn’t collapse done? SPEC-048 quiet mode optimized for “still uploadable,” not viewport budget under multi-run.  
5. Why harden now? Multi-file admit (GH-350) + dual-run UX (SPEC-086) make busy first viewport unusable on laptop heights.

### Causal chain (B)

```ascii
 Multi-file drop
   → toast.loading("Uploading N...")
     + ActiveRuns cards (stepper + bars)
       + table badges for same ids
         + quiet dropzone still in toolbar
           → ≤35vh zone + toolbar chrome starve table
```

References: [NN/g Progressive Disclosure](https://www.nngroup.com/articles/progressive-disclosure/) — show the essential surface first; defer secondary chrome.

---

## Symptom C — Status dual-SSOT drift

`lib/documents/status-domain.ts` and `components/documents/status-badge.tsx` both export `normalizeStatus`, `isProcessingStatus`, `isTerminalStatus`, `getDocumentDisplayStatus`. List merge imports domain; many UI paths import badge. Logic can diverge (e.g. cancelling / held / dead_letter coverage).

### Five WHYs (C)

1. Why two modules? Status helpers grew in the badge component; domain was extracted for merge honesty without finishing the migration.  
2. Why do callers still hit badge? Convenience re-exports + existing imports in hooks/tests (`document-status.ts`, `ingestion-run-view`, cancel tests).  
3. Why is drift dangerous? Merge can treat a status as inflight while UI paints terminal (or the reverse) — honesty bugs.  
4. Why wasn’t it unified in SPEC-098? 098 closed delete pins/merge; UI presentation SSOT was out of scope.  
5. Why harden now? Every new lifecycle status (deleting, delete_failed, cancelling) must be edited twice or it silently splits.

### Causal chain (C)

```ascii
 New lifecycle / pipeline status added
   → domain updated for merge ranks
     OR badge updated for paint
       → one path lags
         → table / zone / filter disagree
```

---

## Symptom D — Clear All peer to Refresh

Header places **Clear All** next to **Refresh** as a primary peer action. Confirmation dialog exists, but proximity to a benign control violates error-prevention proximity guidance ([NN/g proximity of consequential options](https://www.nngroup.com/articles/proximity-consequential-options/)). SPEC-030 **F-DOC-01** already flagged this; still true on v0.22 UI (evidence 01).

### Five WHYs (D)

1. Why is Clear All in the header? Historical “operator power” affordance for wiping a demo workspace.  
2. Why next to Refresh? Both were “global document actions” without severity ranking.  
3. Why isn’t the dialog enough? Confirmations habituate; peer placement increases slips before the dialog ([NN/g confirmation dialogs](https://www.nngroup.com/articles/confirmation-dialog/)).  
4. Why wasn’t it moved? Later specs focused on ingestion honesty, not chrome hierarchy.  
5. Why harden now? Larger corpora + Clear All = irreversible knowledge-graph wipe risk.

### Causal chain (D)

```ascii
 Benign Refresh  |  Destructive Clear All  (same visual weight)
   → slip / misclick
     → typed confirm may still be raced through
       → full corpus delete
```

---

## Symptom E — Scale / filter honesty

`VIRTUAL_PAGE_SIZE = 100` in `document-manager.tsx` fetches a capped page; UI title count and “All Status (N)” can disagree with each other and with true corpus size (evidence 02: Documents **17** vs All Status **11**). Silent truncation (GH-319) implies a complete inventory when it is not.

### Five WHYs (E)

1. Why can counts disagree? Header may use raw/total while filter chips use filtered subset, or busy docs are excluded from a chip bucket inconsistently.  
2. Why is the fetch capped? Virtual table assumes one page; server pagination not wired end-to-end.  
3. Why no overflow affordance? Product assumed “dev corpora < 100.”  
4. Why wasn’t this closed? GH-319 open; honesty work prioritized delete/ingest.  
5. Why harden now? Production workspaces exceed 100; silent incompleteness destroys trust in the inventory.

### Causal chain (E)

```ascii
 GET /documents pageSize=100
   → UI paints "Documents N" without "of M" / overflow
     → filter chips use different aggregator
       → operator believes the table is the full corpus
```

---

## Designer summary (first viewport)

```ascii
IDLE (evidence 01)                          BUSY (evidence 02)
┌─ Header + Clear All (danger loud) ─┐      ┌─ Header + Working/Queued pills ──┐
├─ Search / Filter / Sort ───────────┤      ├─ Search / quiet dropzone ────────┤
├─ LARGE dropzone (~2 rows tall) ────┤      ├─ Active runs cards (≤35vh) ──────┤
├─ Table: Completed + Ready × N ─────┤      │  + table badges pulse same docs │
└─ NEW badges / Cost always on ──────┘      └─ Toast "Uploading N..." (3rd SSOT)┘
```

Target: progressive disclosure of upload + live work; inventory first when idle; one narrative SSOT when busy; one status domain; destructive actions demoted.
