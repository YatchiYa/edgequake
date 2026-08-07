# 00 — First Principles (SPEC-101)

## Axioms

1. **Provisioning is machine work; onboarding is human work** — never leave a half-built tenant from a half-finished wizard.  
2. **One concern per step** — name/identity, models, extraction, review are separate.  
3. **Defaults are honest** — if the system will use a model, show `provider/model` explicitly.  
4. **Progressive disclosure** — Advanced overrides after the happy path is clear.  
5. **DRY shells** — one wizard composition; entry points only wire context.  
6. **Evidence beats vibes** — every finding maps to a unit or Playwright/Rust gate.  
7. **Do not break headless** — CI bootstrap and env admin remain operable.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-101-1** | One wizard shell — Create Tenant, Create Workspace, First-run share `WizardShell` + steps; no third dialog implementation. |
| **LAW-101-2** | Server defaults are always explicit — `ServerDefaultsCard` shows LLM · Embedding · Vision as `provider/model` (or “not configured”) before any override. |
| **LAW-101-3** | Happy-path model step has zero pickers / chip bars; Advanced uses **two-step** Provider select → Model select (`model-picker-provider-trigger`). |
| **LAW-101-4** | Fresh setup is atomic and idempotent — `POST /setup/initialize` creates admin+tenant+workspace+membership in one transaction; repeat → 409. |
| **LAW-101-5** | Inheritance ladder unchanged — Request → Workspace → Tenant → `server_config.llm_defaults` → Env → compiled; UI only discloses it. |
| **LAW-101-6** | CI is proof — every F-101-* has a unit, Playwright, or Rust gate; inherit issue-233 / spec032 / issue-288 green. |
| **LAW-101-7** | Headless bootstrap remains — `EDGEQUAKE_BOOTSTRAP_ADMIN_*` and `EDGEQUAKE_PROVISION_DEFAULTS` opt into silent defaults for demos/CI. |
| **LAW-101-8** | Evidence QC — every primary wizard surface has before/after PNG + multi-viewport capture gate (1440 / 768 / 375); capture-to-disk + DOM asserts (not pixel-diff). |
| **LAW-101-9** | Draft restore — non-secret fields survive refresh via sessionStorage; passwords never persisted (WCAG redundant-entry / EC-101-02). |
| **LAW-101-10** | Dismiss honesty — first-run is non-dismissible (no Cancel/X); create wizards confirm dirty cancel. |
| **LAW-101-11** | Context is always dual — header trigger shows **one line** `Tenant — Workspace` (full names in `title` / `aria-label` / `data-full-name`); popover is **Organization → Workspace** (tenant select keeps open; workspace select closes). |
| **LAW-101-12** | Reconfigure Workspace shares `WizardShell` + steps; prefills current values; Impact Review discloses rebuild/reprocess consequences before `PUT`; page is read-only overview (no parallel inline edit); inheritance ladder unchanged (LAW-101-5). |
| **LAW-101-13** | Never-silent defaults — any “Server default” label must include the **resolved value** in parentheses (e.g. `Server Default (Vision)`, `Server default (English)`, `Server Default (ollama/gemma4:latest)`); never show bare “Server Default” alone. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | Shared steps + `model-payload.ts` omit fields when using server defaults; one `ServerDefaultsCard`; one `buildWorkspaceUpdatePayload` for reconfigure/settings. |
| **SRP** | Shell = chrome; steps = fields; hooks = mutations; lib = pure validation + config diff. |
| **OCP** | New step = add to step list; shell unchanged. Fourth entry = `reconfigure-workspace` kind. |
| **DIP** | Wizards depend on `useSetupStatus` / create/update APIs, not header/page internals. |
| **ISP** | Steps receive only their slice of wizard state. |
| **LSP** | Memory/dev and postgres setup status share the same DTO shape. |

## Inheritance (do not break)

| Prior           | Constraint                                               |
| -----------------| ----------------------------------------------------------|
| #233 / SPEC-013 | Workspace create may omit model fields → server defaults |
| SPEC-041        | Vision LLM remains configurable                          |
| SPEC-043        | Full picker available in Advanced                        |
| issue-288       | Env bootstrap admin path stays green                     |
| SPEC-096        | Extraction language optional on workspace create         |
