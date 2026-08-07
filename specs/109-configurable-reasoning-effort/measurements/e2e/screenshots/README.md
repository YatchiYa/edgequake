# SPEC-109 E2E screenshots

Captured by `make spec109-e2e` → `edgequake_webui/e2e/spec109-reasoning-effort.spec.ts`
via `spec109Screenshot()` → this directory.

Date: 2026-08-05 · Live stack backend `:8090` / frontend `:3010` · **6/6 Playwright passed**

| File | Surface | Visual notes |
|------|---------|--------------|
| `01-query-page.png` | `/w/{slug}/query` | Query shell ready (tenant seeded); settings trigger present |
| `01-query-sheet.png` | Query settings sheet | Generation → Reasoning effort control visible |
| `02-query-effort-options.png` | Effort select open | Catalog-filtered options; Auto (inherit) default |
| `03-settings-page.png` | `/settings` | Settings shell |
| `03-settings-fleet.png` | Server LLM card | Fleet **Default reasoning effort** + start of per-role overrides |
| `04-settings-by-role.png` | `server-reasoning-by-role` | extract / query / vlm role selects + Save server defaults |
| `05-explainability-roles.png` | `reasoning-roles-explain` | Desired vs effective (clamped) per role; Auto→omit/none for Mistral |
| `06-workspace-page.png` | `/w/{slug}/workspace` | Workspace config shell |
| `06-workspace-role.png` | Role effort readonly | **Extract effort: Auto**, **Query effort: Auto** |
| `07-documents-page.png` | `/w/{slug}/documents` | Documents shell |
| `07-documents-upload.png` | Upload / parser | `spec038-upload-parser-select` |
| `07-documents-vision-effort.png` | Vision parser | **Vision effort** select (Auto inherit) beside Vision parser |

## Network proof

Test `chat/query request includes reasoning_effort when set` intercepts `**/api/v1/chat/completions**` and `**/api/v1/query**` and asserts `reasoning_effort: "low"` after selecting Low in the query sheet.
