# SPEC-052 — Layout Audit

> **Date:** 2026-07-14  
> **Branch:** feat/spec047-vision-ingest-spec048-progress  
> **Scope:** All dialogs and primary screens in `edgequake_webui`

---

## 1. Methodology

Each dialog component was inspected for:

| Check | Rule |
|-------|------|
| Container model | `DialogContent` must declare its layout model explicitly (`grid` default vs `flex flex-col` for full-height viewers) |
| Height overflow | `max-h-*` without `overflow-y-auto` silently clips content |
| Button visibility | Toggle / action buttons must use `Button`, not `Badge`, so hit-area and box model are guaranteed |
| Footer consistency | Confirmation-style footers use `<DialogFooter className="gap-2">` |
| Close button | Full-screen (`p-0`) dialogs must set `showCloseButton={false}` to prevent overlap with header actions |
| Alignment | Row items use `items-center`, not `items-start`, unless multi-line text justifies top-align |
| Import hygiene | Unused component imports (e.g. `Badge` after replacing with `Button`) removed |

---

## 2. Dialog Inventory & Findings

### 2.1 `duplicate-upload-dialog.tsx` ✅ Fixed (turn 1)

| Finding | Severity | Fix |
|---------|----------|-----|
| `Badge` used as interactive toggle — `inline-flex` with no guaranteed hit area caused visual overlap when two badges are side-by-side | High | Replaced with `Button variant="default/outline" size="sm"` |
| `items-start` on row container misaligned icon and button group vs. two-line filename block | Medium | Changed to `items-center`; removed `mt-0.5` offset hack |
| Unused `Badge` import left after replacement | Low | Removed import |

### 2.2 `document-viewer-dialog.tsx` ✅ Fixed (turn 2)

| Finding | Severity | Fix |
|---------|----------|-----|
| `DialogContent` is `grid` by default; children used `flex-1 overflow-hidden` which has no effect in a grid container — viewer panel did not fill available height | **Critical** | Added `flex flex-col` to override the default grid |
| Auto-generated `×` close button (`absolute top-4 right-4`) overlapped the action buttons (Download, Open-in-tab) in the `p-0` full-screen header | High | Added `showCloseButton={false}` |

### 2.3 `document-detail-dialog.tsx` ✅ Fixed (turn 2)

| Finding | Severity | Fix |
|---------|----------|-----|
| `max-h-[90vh]`/`max-h-[85vh]` set but no `overflow-y-auto` — tall content (error banner + PDF split-view at 450 px height) silently clipped at the bottom | High | Added `overflow-y-auto` to both branches of the className ternary |

### 2.4 `export-dialog.tsx` ✅ Fixed (turn 2)

| Finding | Severity | Fix |
|---------|----------|-----|
| Footer used raw `<div className="flex justify-end gap-2">` — inconsistent bottom padding vs. all other dialogs which use `<DialogFooter>` | Medium | Added `DialogFooter` to imports; replaced div with `<DialogFooter className="gap-2">` |

### 2.5 `large-pdf-admission-dialog.tsx` ✅ Fixed (turn 2)

| Finding | Severity | Fix |
|---------|----------|-----|
| `<DialogFooter>` missing `gap-2` — Cancel and Confirm buttons rendered flush | Low | Added `className="gap-2"` |

### 2.6 `reprocess-dialog.tsx` ✅ No action required

- `max-w-lg`, `DialogFooter className="gap-2"` ✓  
- `ReprocessOption` uses label+RadioGroupItem correctly ✓  
- `inflight` warning banner with border-amber ✓  

### 2.7 `bulk-reprocess-dialog.tsx` ✅ No action required

- Same structure as `reprocess-dialog`, consistent ✓  

### 2.8 `delete-confirm-dialog.tsx` ✅ No action required

- `max-w-md`, `DialogFooter className="gap-2"` ✓  
- Warning banner `items-start gap-2` correct for multi-line text ✓  

### 2.9 `bulk-delete-confirm-dialog.tsx` ✅ No action required

- `AlertDialogContent` (shadcn default `max-w-lg`) ✓  
- `AlertDialogDescription asChild` with `div` for complex content ✓  

### 2.10 `clear-documents-dialog.tsx` ✅ No action required

- `AlertDialogContent` default ✓  
- Real-time `Progress` bar during deletion ✓  
- Confirmation input blocks accidental submit ✓  

### 2.11 `pipeline-status-dialog.tsx` ✅ No action required

- `sm:max-w-lg overflow-hidden` ✓  
- `DialogTitle` has `pr-8` to avoid overlap with default close button ✓  
- Explicit scroll areas per content section ✓  

### 2.12 `keyboard-shortcuts-dialog.tsx` ✅ No action required

- `sm:max-w-lg` ✓  
- No `DialogFooter` needed (keyboard-only dismiss via Esc) ✓  

### 2.13 `edit-quota-dialog.tsx` ✅ No action required

- `sm:max-w-sm`, `DialogFooter className="gap-2"` ✓  

### 2.14 `share-dialog.tsx` ✅ No action required (intentional)

- Footer uses raw `<div className="flex justify-between">` — this is **intentional UX**: destructive "Remove Link" on the left, safe actions on the right. `DialogFooter` has `sm:flex-row-reverse` which would break this separation.

### 2.15 `entity-edit-dialog.tsx` ✅ No action required

- `sm:max-w-lg flex flex-col max-h-[90dvh]` ✓  
- `ScrollArea flex-1 min-h-0` inside form ✓  

### 2.16 `relationship-edit-dialog.tsx` ✅ No action required

- `sm:max-w-md`, `DialogFooter className="gap-2"` ✓  

### 2.17 `document-viewer-dialog.tsx` (full-screen) — see §2.2

### 2.18 Knowledge injection dialogs (`knowledge/page.tsx`) ⚠️ Accepted

| Finding | Severity | Decision |
|---------|----------|----------|
| `<DialogFooter>` is nested inside `<TabsContent>` rather than at root `<DialogContent>` level. On most screens this has no visual impact since no `max-h` is set on the dialog. | Low | **Accept as-is** — refactoring requires extracting the inline dialog to a dedicated component, which is out of scope for this audit. Tracked for future cleanup. |

---

## 3. Screen Inventory

All primary routes audited; screenshots in `e2e/screenshots/pages/`.

| Route | Screenshot | Notes |
|-------|-----------|-------|
| `/documents` | `pages/documents.png` | Upload drop-zone, toolbar, table |
| `/graph` | `pages/graph.png` | Force-graph canvas |
| `/query` | `pages/query.png` | Chat + sidebar |
| `/knowledge` | `pages/knowledge.png` | Injection card grid |
| `/settings` | `pages/settings.png` | Card-based settings |
| `/workspace` | `pages/workspace.png` | Workspace management |
| `/costs` | `pages/costs.png` | Cost table |
| `/pipeline` | `pages/pipeline.png` | Pipeline list |
| `/api-explorer` | `pages/api-explorer.png` | OpenAPI explorer |

---

## 4. DRY / SOLID Compliance

### What was enforced

| Principle | Enforcement |
|-----------|-------------|
| **DRY** | `DialogFooter`, `DialogHeader`, `DialogTitle`, `DialogDescription` used consistently across all dialogs — no raw divs for standard footer patterns (except intentional `share-dialog`) |
| **Single Responsibility** | Each dialog handles exactly one user intent (delete, reprocess, export, …) |
| **Open/Closed** | All dialogs extend shadcn primitives without patching them |
| **Dependency Inversion** | Dialogs depend on `@/components/ui/dialog` primitives, not on raw Radix |

### Shared patterns

| Pattern | Usage |
|---------|-------|
| `flex flex-col` on full-screen `DialogContent` | `document-viewer-dialog` |
| `overflow-y-auto` on bounded `DialogContent` | `document-detail-dialog` |
| `<DialogFooter className="gap-2">` | All confirmation dialogs |
| `Button variant="default/outline"` for toggles (not `Badge`) | `duplicate-upload-dialog` |

---

## 5. E2E Tests

Playwright spec: `edgequake_webui/e2e/spec052-layout-audit.spec.ts`  
Tagged `@audit` — runs under the `audit` project (single worker, extended timeout).

```
specs/052-layout-audit/e2e/screenshots/
  pages/            full-page screenshots of every route
  dialogs/          dialog screenshots (triggered programmatically)
  states/           edge-case states (error, empty, loading)
```

Run:
```bash
cd edgequake_webui
E2E_LIVE_STACK=1 pnpm exec playwright test spec052-layout-audit --project=audit
```
