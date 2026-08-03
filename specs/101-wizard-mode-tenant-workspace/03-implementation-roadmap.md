# 03 — Implementation Roadmap (SPEC-101)

## Waves

| Wave | Deliverable | DoD |
|------|-------------|-----|
| **0** | Spec pack + evidence | Docs linked from README; screenshots in evidence/ |
| **1** | WizardShell, ServerDefaultsCard, ModelDefaultsStep, `wizard-state` / `model-payload` | Unit tests green; density simple picker |
| **2** | CreateTenantWizard + CreateWorkspaceWizard; wire header + guard; delete orphan | Header/guard use wizards; no TenantWorkspaceSelector imports |
| **3** | `GET/POST /setup/*`; gate ensure_defaults; FirstRunWizard | Fresh empty auth-on DB shows wizard; initialize atomic |
| **4** | Playwright spec101-* + Rust setup tests; OpenAPI | Matrix rows green |
| **5** | A11y, i18n, skeletons, clippy/fmt; matrix paths final | LAW-101-6 satisfied |
| **6** | UX QC: drafts, dismiss honesty, inline validation, Review Edit, evidence capture | LAW-101-8…10; after-*.png; hardened e2e |
| **7** | Context selector clarity: two-line Tenant/Workspace chip; workspaces-first popover | LAW-101-11; `spec101-context-selector` green |
| **8** | Reconfigure Workspace wizard; replace inline edit; Impact Review; deeplink parity | LAW-101-12; `spec101-reconfigure-*` + evidence |

## Backend sequence (Wave 3)

1. Add `SetupStatusResponse` / `SetupInitializeRequest` DTOs + OpenAPI.  
2. Implement `GET /api/v1/setup/status` (public or auth-aware).  
3. Implement `POST /api/v1/setup/initialize` in a single PG transaction.  
4. Gate `ensure_defaults`: skip when fresh-setup mode (see LAW-101-7 escapes).  
5. Wire routes in `routes.rs`.

## Frontend sequence (Waves 1–2)

1. Pure lib: validation + payload builders.  
2. Presentational shell + card.  
3. Steps.  
4. Three wizards.  
5. Replace dialogs in header/guard.  
6. Delete orphan.

## Exit criteria

- [x] First-run on empty DB creates admin+tenant+workspace without env password.  
- [x] Create Tenant/Workspace never show chip storm on happy path.  
- [x] ServerDefaultsCard always shows three lines.  
- [x] issue-233 + issue-288 still green.  
- [x] Orphan selector gone.  
- [x] Draft restore (non-secrets) + first-run non-dismissible + dirty cancel confirm.  
- [x] Multi-viewport after evidence PNGs via `spec101-ux-capture`.  
- [x] Header always dual-labels Tenant + Workspace; popover workspaces-first / keep-open on tenant (LAW-101-11).
- [x] Reconfigure wizard replaces inline edit; Impact Review + rebuild hints (LAW-101-12).
- [x] Deeplink workspace page opens same reconfigure wizard with PDF/vision parity.
- [x] `spec101-reconfigure-wizard` + `after-reconfigure-*.png` evidence green.
