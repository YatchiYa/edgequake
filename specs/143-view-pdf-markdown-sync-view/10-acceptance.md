# 10 — Acceptance

## Must pass

- [ ] Spec pack complete under `specs/143-view-pdf-markdown-sync-view/`
- [ ] Continuous PDF stack: wheel changes `pdf-page-indicator[data-page]`
- [ ] `onPageChange` updates `?page=` (debounced)
- [ ] Markdown contains `[data-eq-page]` for each marker
- [ ] Sync ON: PDF→MD and MD→PDF page alignment
- [ ] Sync OFF: independent scroll
- [ ] FEAT0733 toggle present with `data-testid="pdf-md-sync-toggle"`
- [ ] Keyboard PageDown/Up changes page when PDF focused
- [ ] SPEC-128 overlay still works on active page
- [ ] SPEC-142 deeplink `?page=4` still lands both panes
- [ ] Unit tests for page-markers + controller
- [ ] Playwright `spec143-pdf-markdown-sync.spec.ts` green
- [ ] No DB migration

## Explicitly deferred (P1)

- [ ] Persist sync preference in localStorage
- [ ] Pixel/bbox paragraph sync beyond SPEC-128 figures
- [ ] Virtualized markdown full IO parity for 10k+ line docs

## Sign-off

| Role | Sign |
|------|------|
| PO | Outcome: readable sync |
| Fullstack | DRY/SOLID modules landed |
| PDF viewer | Continuous stack + windowing |
| QA | E2E matrix green |

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
