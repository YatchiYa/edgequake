# SPEC-101 — Wizard Mode Tenant / Workspace Onboarding

> **Product pin**: EdgeQuake v0.22.0+  
> **Status**: Waves 0–8 (Wave 8 reconfigure workspace wizard)  

> **Inherits**: SPEC-013/#233 server defaults · SPEC-032 workspace models · SPEC-041 vision LLM · SPEC-043 model picker · SPEC-027 auth bootstrap · SPEC-096 extraction language · SPEC-099 progressive disclosure  
> **Peers**: `docs/operations/runtime-auth-hardening.md` · issue-288 login bootstrap

## Start here

1. [00-why.md](00-why.md) — Five WHYs + causal ASCII  
2. [00-first-principles.md](00-first-principles.md) — LAW-101-1…12 + SOLID/DRY  
3. [01-finding-register.md](01-finding-register.md) — F-101-*  
4. [02-cross-ref-matrix.md](02-cross-ref-matrix.md) — code ↔ law ↔ test  
5. [03-implementation-roadmap.md](03-implementation-roadmap.md) — Waves 0–8 + DoD  
6. [04-e2e-test-matrix.md](04-e2e-test-matrix.md) — gates  
7. [05-edge-cases.md](05-edge-cases.md) — EC register  
8. Issues → [`issues/`](issues/)  
9. Lenses → [`lenses/`](lenses/)  
10. Evidence → [`evidence/README.md`](evidence/README.md)

## Scope (locked)

| In | Out |
|----|-----|
| Secure first-run wizard (admin → tenant → workspace) on empty auth-on DB | Full WebUI redesign |
| Create Tenant / Create Workspace multi-step wizards | Changing inheritance ladder semantics |
| Reconfigure Workspace wizard (models · PDF · extraction · impact) | Workspace rename/slug in reconfigure |
| Explicit server defaults (LLM · Embedding · Vision) | Billing, invites, sample docs |
| DRY shell shared by 4 entry points; delete orphan selector | Feature tours (graph/docs/query) |
| `GET/POST /api/v1/setup/*` atomic initialize | Replacing env bootstrap for CI/headless |

## Locked decisions

1. **Provisioning ≠ onboarding UX** — atomic/idempotent machine work; wizard is guided human path.  
2. **Fresh empty DB** (zero tenants ∧ zero login-capable users ∧ auth on) → no silent Default seed.  
3. **Existing DBs** → never force first-run; reuse wizard for Create Tenant/Workspace.  
4. **Headless escape** — `EDGEQUAKE_BOOTSTRAP_ADMIN_*` / `EDGEQUAKE_PROVISION_DEFAULTS=true` remain.  
5. **Server defaults first** — explicit `provider/model` × 3; Advanced only for override (no chip storm).  
6. **One wizard shell** — First-run · Create Tenant · Create Workspace · Reconfigure Workspace.  
7. **CI is proof** — every F-101-* maps to unit or Playwright/Rust gate.  
8. **Evidence QC** — before/after PNGs + multi-viewport capture (LAW-101-8).  
9. **Draft restore / dismiss honesty** — LAW-101-9 / LAW-101-10.  
10. **Reconfigure replaces inline edit** — Workspace page is read-only overview; Edit Configuration opens guided wizard (LAW-101-12).

## Surfaces

| Surface | Role |
|---------|------|
| `components/onboarding/wizard-shell.tsx` | Progress, Back/Next, a11y |
| `server-defaults-card.tsx` | Explicit LLM/Embed/Vision SSOT paint |
| `steps/*` | One concern per step |
| `first-run-wizard.tsx` | Secure onboarding entry |
| `create-tenant-wizard.tsx` / `create-workspace-wizard.tsx` | Ongoing create |
| `reconfigure-workspace-wizard.tsx` | Edit Configuration → guided PUT |
| `HeaderTenantSelector` / `TenantGuard` | Thin shells wiring wizards + context selection |
| `components/layout/context-selector/*` | Tenant — Workspace chip + popover (LAW-101-11) |
| `GET/POST /api/v1/setup/*` | Status + atomic initialize |

## Target composition

```ascii
WizardShell
├── ProgressBar (step N / M)
├── StepHeader (title + explained subtitle)
├── StepBody
│   ├── AdminCredentialsStep      (first-run only)
│   ├── TenantBasicsStep
│   ├── ModelDefaultsStep         ← ServerDefaultsCard + Advanced
│   ├── WorkspaceBasicsStep       (create paths)
│   ├── DocumentParsingStep       (reconfigure — PDF parser)
│   ├── WorkspaceExtractionStep   (workspace / reconfigure path)
│   └── ReviewStep                (+ Impact on reconfigure)
└── Footer [Cancel] [Back] [Next | Create | Apply]
```

## Verification

```bash
cd edgequake_webui && bun test src/lib/onboarding/
pnpm exec playwright test e2e/spec101-
pnpm exec playwright test e2e/issue-233-workspace-create-defaults.spec.ts
pnpm exec playwright test e2e/spec101-reconfigure-
ls specs/101-wizard-mode-tenant-workspace/evidence/after-*.png
ls specs/101-wizard-mode-tenant-workspace/evidence/after-reconfigure-*.png
```

See [04-e2e-test-matrix.md](04-e2e-test-matrix.md).
