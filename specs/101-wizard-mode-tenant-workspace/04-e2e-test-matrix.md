# 04 — E2E Test Matrix (SPEC-101)

| Gate | Type | Asserts | Law |
|------|------|---------|-----|
| `src/lib/onboarding/__tests__/wizard-state.test.ts` | unit | steps, canProceed, create-tenant includes extraction | LAW-101-1 |
| `src/lib/onboarding/__tests__/model-payload.test.ts` | unit | omit when useServerDefaults | LAW-101-5 |
| `src/lib/onboarding/__tests__/wizard-draft-storage.test.ts` | unit | round-trip; passwords excluded | LAW-101-9 |
| `e2e/spec101-server-defaults-explicit.spec.ts` | Playwright | three lines match `provider/model` or not configured | LAW-101-2 |
| `e2e/spec101-no-chip-storm.spec.ts` | Playwright | no provider bar on happy path / advanced | LAW-101-3 |
| `e2e/spec101-create-wizards.spec.ts` | Playwright | workspace + tenant finish succeed | LAW-101-1 |
| `src/lib/layout/__tests__/format-context-labels.test.ts` | unit | aria/title/truncate helpers | LAW-101-11 |
| `e2e/spec101-context-selector.spec.ts` | Playwright | two-line labels; keep-open; workspaces-first | LAW-101-11 |
| `e2e/spec101-first-run.spec.ts` | Playwright | mocked status → wizard UI; no Cancel/X | LAW-101-4/10 |
| `e2e/spec101-ux-capture.spec.ts` | Playwright | multi-viewport 1440/768/375 + after PNGs + viewport fit | LAW-101-8 |
| `edgequake-api` setup status/initialize | Rust | atomic + 409 | LAW-101-4 |
| `e2e_issue288_login_bootstrap` | Rust | env bootstrap | LAW-101-7 |
| `issue-233-workspace-create-defaults` | Playwright | non-regression | LAW-101-2 |
| `src/lib/onboarding/__tests__/workspace-config-diff.test.ts` | unit | rebuild hints / no-op | LAW-101-12 |
| `e2e/spec101-reconfigure-wizard.spec.ts` | Playwright | open → steps → Apply; cards reflect | LAW-101-12 |
| `e2e/spec101-reconfigure-ux-capture.spec.ts` | Playwright | 1440/768/375 after-reconfigure PNGs | LAW-101-8/12 |
| `e2e/spec096-extraction-language.spec.ts` | Playwright | language/entity via reconfigure wizard | LAW-101-12 · SPEC-096 |

## Commands

```bash
cd edgequake_webui
bun test src/lib/onboarding/
pnpm exec playwright test e2e/spec101-
pnpm exec playwright test e2e/spec101-reconfigure-
pnpm exec playwright test e2e/spec096-extraction-language.spec.ts
pnpm exec playwright test e2e/issue-233-workspace-create-defaults.spec.ts
ls specs/101-wizard-mode-tenant-workspace/evidence/after-*.png
ls specs/101-wizard-mode-tenant-workspace/evidence/after-reconfigure-*.png

cd ../edgequake
cargo test -p edgequake-api --lib handlers::setup
cargo test -p edgequake-api --test e2e_spec101_setup --features postgres
```
