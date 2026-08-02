# Evidence (SPEC-101)

| File | Description |
|------|-------------|
| [before-create-tenant.png](before-create-tenant.png) | Pre-SPEC Create Tenant — chip storm + three required pickers |
| [before-create-workspace.png](before-create-workspace.png) | Pre-SPEC Create Workspace — server defaults summary + advanced |
| `after-create-tenant-*.png` | Post Wave 6 wizard (models / review) per viewport |
| `after-create-workspace-*.png` | Post Wave 6 wizard (models / review) per viewport |
| `after-reconfigure-*.png` | Wave 8 reconfigure (models / document-parsing / review) per viewport |

## Capture (LAW-101-8)

```bash
cd edgequake_webui
pnpm exec playwright test e2e/spec101-ux-capture.spec.ts
pnpm exec playwright test e2e/spec101-reconfigure-ux-capture.spec.ts
```

Writes via `spec101Screenshot()` into this directory and `e2e/screenshots/spec101/`.
Viewports: **1440 · 768 · 375**.
