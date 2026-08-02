# ISSUE — Reconfigure Workspace Wizard (Wave 8)

> **SPEC-101** · **LAW-101-12** · Findings F-101-22…25

## Problem

Workspace configuration uses a dense **inline edit** mode on `/workspace` (and a partial copy on the deeplink route). Create flows already share `WizardShell` + progressive disclosure; **edit does not**. Users get no guided Impact Review for rebuild consequences; PDF parser is absent from create-time guidance; deeplink lacks vision/PDF parity.

## Target

```ascii
WorkspacePage (read-only)
├── [Edit Configuration] ──opens──► ReconfigureWorkspaceWizard
│                                   ├── Models
│                                   ├── Document parsing (PDF)
│                                   ├── Extraction (+ strict)
│                                   └── Review + Impact
└── Stats / cards / rebuild actions
```

## Acceptance

1. `workspace-edit-config` opens `reconfigure-workspace-wizard` (no inline Save/Cancel on page).
2. Steps: models → document-parsing → extraction → review; prefills current workspace values.
3. Review shows `wizard-reconfigure-impact` with change + rebuild hints when models differ.
4. Apply → `PUT /workspaces/{id}`; page invalidates; `pendingRebuild` set from diff.
5. Deeplink `/w/[slug]/workspace` same wizard + full field parity.
6. Draft restore keyed by workspace id (LAW-101-9); dirty cancel confirm (LAW-101-10).
7. Playwright `spec101-reconfigure-wizard` + UX capture evidence green; SPEC-096 updated.

## Test IDs

| ID | Surface |
|----|---------|
| `workspace-edit-config` | Header entry (compat) |
| `reconfigure-workspace-wizard` | Dialog root |
| `wizard-step-models` | Models step |
| `wizard-step-document-parsing` | PDF parser step |
| `wizard-step-extraction` | Extraction step |
| `wizard-reconfigure-impact` | Impact block on Review |
| `wizard-finish` / Apply | Shell finish (label Apply) |

## Code

- `components/onboarding/reconfigure-workspace-wizard.tsx`
- `components/onboarding/steps/document-parsing-step.tsx`
- `lib/onboarding/{wizard-state,model-payload,workspace-config-diff}.ts`
- Thin wire in `(dashboard)/workspace/page.tsx` + `w/[slug]/workspace/page.tsx`

## Follow-up (not blocking)

Settings vision/PDF cards share `buildWorkspaceUpdatePayload`; deep-link Settings → reconfigure wizard deferred.
