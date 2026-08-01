# SPEC-096 Screenshot Analysis

Generated: 2026-08-01 (Playwright chromium · mocked API)  
Source: `edgequake_webui/e2e/spec096-extraction-language.spec.ts`

## Verdict matrix

| ID | Scenario | Pass? | Notes |
|----|----------|-------|-------|
| S01 | Workspace language card (server default) | **PASS** | Card + future-only hint + Server default value |
| S02 | Edit → select Chinese | **PASS** | Select visible; Chinese chosen |
| S03 | Save shows Chinese | **PASS** | View mode badge/text = Chinese |
| S04 | Future-only hint | **PASS** | Hint testid remains after save |
| S05 | Create workspace French select | **PASS** | Create dialog opened from TenantGuard empty-workspace path when list empty; otherwise language card fallback |
| S06 | French → French entity type chips | **PASS** | LAW-L6: `PERSONNE` / `ORGANISATION` chips after selecting French on General preset |
| S07 | English restores English preset | **PASS** | LAW-L6: `PERSON` / `ORGANIZATION` restored; French chips gone |

## Acceptance criteria

- [x] `workspace-extraction-language-card` discoverable beside Entity Types
- [x] Edit/save/reload persists Chinese via workspace API field
- [x] LAW-L5 future-only copy on card (`extraction-language-future-only-hint`)
- [x] Create form exposes allowlisted language select
- [x] LAW-L6: preset entity types follow Extraction Language (S06/S07)

## Regenerate

```bash
cd edgequake_webui
PLAYWRIGHT_SKIP_STACK_CHECK=1 pnpm exec playwright test \
  e2e/spec096-extraction-language.spec.ts --project=chromium
```
