# 02 — Cross-Ref Matrix (SPEC-101)

Paths under `edgequake_webui/` unless noted.

| Finding | Code | Test | Law |
|---------|------|------|-----|
| F-101-01 | `components/onboarding/{wizard-shell,create-*-wizard}.tsx`; thin `header-tenant-selector` / `tenant-guard` | Playwright `spec101-create-wizards` | LAW-101-1 |
| F-101-02 | `steps/model-defaults-step.tsx`; two-step provider→model (no chip bars) | Playwright `spec101-no-chip-storm` | LAW-101-3 |
| F-101-03 | `server-defaults-card.tsx`; `use-server-model-defaults` (+ vision) | Playwright `spec101-server-defaults-explicit` · `issue-233-*` | LAW-101-2 |
| F-101-04 | `edgequake-api` `handlers/setup.rs`; gate `ensure_defaults` | Rust setup · Playwright `spec101-first-run` | LAW-101-4/7 |
| F-101-05 | `use-setup-status.ts` → `setNeedsOnboarding` | unit + first-run | LAW-101-1 |
| F-101-06 | remove `tenant-workspace-selector.tsx` | grep / build | LAW-101-1 |
| F-101-07 | `lib/onboarding/model-payload.ts` | unit `model-payload.test.ts` | LAW-101-5 |
| F-101-08 | ServerDefaultsCard skeleton | Playwright defaults card | LAW-101-2 |
| F-101-09 | CreateTenantWizard PATCH default workspace | Playwright tenant complete | LAW-101-4 |
| F-101-10 | `e2e/spec101-*.spec.ts` hard asserts | CI | LAW-101-6 |
| F-101-11 | `lib/onboarding/wizard-draft-storage.ts` | unit `wizard-draft-storage.test.ts` | LAW-101-9 |
| F-101-12 | `WizardShell` `hideCancel` + `showCloseButton={false}` on first-run | Playwright `spec101-first-run` | LAW-101-10 |
| F-101-13 | dirty cancel AlertDialog in shell | Playwright create cancel | LAW-101-10 |
| F-101-14 | live region + step `aria-labelledby` | Playwright a11y attrs / capture | LAW-101-8 |
| F-101-15 | step inline validation | unit canProceed + Playwright | LAW-101-6 |
| F-101-16 | slug hint + Review Edit links | Playwright / unit | Wave 6 UX |
| F-101-17 | `spec101Screenshot` + `e2e/spec101-ux-capture.spec.ts` → `evidence/after-*.png` | Playwright capture | LAW-101-8 |
| F-101-18 | hardened defaults regex / tenant finish / first-run UI | `spec101-*` | LAW-101-6 |
| F-101-19 | `context-trigger-chip.tsx` one-line `Tenant — Workspace` | unit + `spec101-context-selector` | LAW-101-11 |
| F-101-20 | `context-selector-popover.tsx` (Organization → Workspace, keep-open) | Playwright `spec101-context-selector` | LAW-101-11 |
| F-101-21 | popover search/empty i18n keys | Playwright copy / unit | LAW-101-11 |
| F-101-22 | `reconfigure-workspace-wizard.tsx`; remove page `isEditing` | Playwright `spec101-reconfigure-wizard` | LAW-101-12 |
| F-101-23 | deeplink workspace page wires same wizard | Playwright reconfigure / parity | LAW-101-12 |
| F-101-24 | `steps/document-parsing-step.tsx` | Playwright document-parsing step | LAW-101-12 |
| F-101-25 | `workspace-config-diff.ts` + Review impact | unit + Playwright impact | LAW-101-12 |

## Non-regression anchors

| Prior gate | Protects |
|------------|----------|
| `e2e/issue-233-workspace-create-defaults.spec.ts` | Server-defaults summary |
| `e2e/spec032-tenant-workspace-dialogs.spec.ts` | Dialog presence / models API |
| `e2e_issue288_login_bootstrap` (Rust) | Env bootstrap admin |
| `e2e/helpers/spec013-bootstrap.ts` | Deterministic UI context |

## Issue ↔ finding

| Issue | Findings |
|-------|----------|
| [ISSUE-wizard-shell-dry](issues/ISSUE-wizard-shell-dry.md) | F-101-01, F-101-06, F-101-07 |
| [ISSUE-server-defaults-explicit](issues/ISSUE-server-defaults-explicit.md) | F-101-02, F-101-03, F-101-08 |
| [ISSUE-secure-first-run](issues/ISSUE-secure-first-run.md) | F-101-04, F-101-05, F-101-09 |
| [ISSUE-spec101-e2e-gates](issues/ISSUE-spec101-e2e-gates.md) | F-101-10, F-101-18 |
| [ISSUE-wizard-ux-qc](issues/ISSUE-wizard-ux-qc.md) | F-101-11…17 |
| [ISSUE-context-selector-clarity](issues/ISSUE-context-selector-clarity.md) | F-101-19, F-101-20, F-101-21 |
| [ISSUE-reconfigure-workspace-wizard](issues/ISSUE-reconfigure-workspace-wizard.md) | F-101-22, F-101-23, F-101-24, F-101-25 |
