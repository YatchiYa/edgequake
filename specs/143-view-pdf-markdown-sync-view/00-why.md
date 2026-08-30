# 00 — Five WHYs

## Problem statement

Users viewing a PDF and its extracted Markdown side-by-side cannot keep the
two panes aligned: changing the PDF page does not scroll Markdown to the
matching section, scrolling Markdown does not update the PDF page, and the
mouse wheel does not navigate PDF pages.

---

### WHY 1 — Why can't users keep PDF and Markdown aligned?

**Answer:** There is no shared active-page controller. Each pane scrolls
independently. FEAT0733 claims “panel synchronization controls” but only
toggles layout mode (PDF / MD / side-by-side).

---

### WHY 2 — Why is there no shared page controller?

**Answer:** PDFViewer is a single-page renderer with internal `pageNumber`.
It accepts `currentPage` for inbound deeplinks but has no `onPageChange`.
Toolbar prev/next never write `?page=` or notify Markdown.

---

### WHY 3 — Why doesn't Markdown expose page positions?

**Answer:** Product markdown already embeds `<!-- edgequake-page:N -->`
(SPEC-083 X-13). HTML comments are stripped by the markdown pipeline /
sanitizer, so no DOM anchors exist for IntersectionObserver or scrollIntoView.

---

### WHY 4 — Why doesn't the mouse wheel change PDF pages?

**Answer:** One `<Page>` sits in `overflow-y-auto`. Wheel pans the current
canvas. There is no continuous page stack, so scroll never crosses page
boundaries into a next sheet.

---

### WHY 5 — Why was marker metadata not reused for the viewer?

**Answer:** Markers were built for chunking (`page_start`/`page_end`) and
MM asset binding. The document viewer never closed the loop: storage → DOM
anchors → sync. SPEC-033 / SPEC-142 stop at “open page N”, not continuous
read sync.

---

## Root cause (one line)

```text
Page markers exist in storage; the viewer never lifts them into DOM anchors
or a shared page controller, and the PDF pane is single-page so wheel cannot
navigate.
```

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
