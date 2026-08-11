# Lens 001 — Product Owner

## Job to be done

Operators must trust that workspace/upload parser settings are honored. A “Vision” workspace that silently runs EdgeParse destroys confidence in all config surfaces.

## Outcomes

1. Vision means Vision. EdgeParse means EdgeParse. Auto means “system may pick.”
2. Priority is publishable: Upload > Workspace > Tenant > Env.
3. Batch upload behaves like N honest single uploads — no silent widening of overrides.
4. Acceptance is e2e-gated, not “works on my machine.”

## Non-goals

- Marketing EdgeParse as always faster without honesty about when it runs.
- Shipping Auto as the default for existing “Server Default (Vision)” workspaces.

## Success metrics

| Metric | Target |
|--------|--------|
| Mis-parse incidents (Vision UI → EdgeParse lineage) | 0 after ship |
| E2E priority matrix | All green |
| Support tickets “wrong parser” | Down |

## Messaging

- Settings: “Resolves to Vision — Vision will be used.”
- Auto: “Auto may use EdgeParse for born-digital PDFs.”
