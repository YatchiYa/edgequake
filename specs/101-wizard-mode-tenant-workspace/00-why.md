# 00 — Why (SPEC-101)

## Five WHYs

### WHY-1 — Why is Create Tenant hard?

Because the dialog packs **name + three full model pickers** (provider chips × capability chips × dropdown) into one modal. Cognitive load spikes before the user understands what a tenant is.

### WHY-2 — Why are provider chips the default?

Because SPEC-043 `ModelPickerPanel` originally defaulted an **external** provider chip bar (`showProviderFilters=true`) and create dialogs mounted three pickers at once. Power-user discovery leaked into first-value setup. Provider choice is now a dedicated **Provider** select (two-step → Model); chip bars are gone.

### WHY-3 — Why do server defaults feel hidden?

Workspace create (#233) shows a summary; **tenant create does not**. Vision is often omitted from the summary. Users cannot trust “Server default” without seeing `provider/model`.

### WHY-4 — Why is first-run insecure / opaque?

Boot calls `ensure_defaults()` → silent Default tenant/workspace. Admin comes from env vars, not a guided password choice. Empty installs look “already set up” without an owner who chose credentials.

### WHY-5 — Why does the same form exist three times?

`HeaderTenantSelector`, `TenantGuard`, and orphan `TenantWorkspaceSelector` diverged (required models, colon vs slash IDs, vision presence). DRY debt became UX debt: inconsistent requiredness and dead `needsOnboarding`.

## Causal ASCII

```ascii
  Boot ensure_defaults ──► silent Default tenant/ws
           │
           ▼
  User opens UI ──► TenantGuard OR Header dialog
           │              │
           │              ├─ Tenant: 3× chip pickers (required)
           │              └─ Workspace: defaults card (header only)
           ▼
  Triple implementations diverge ──► confusion + skips in e2e
           │
           ▼
  Activation delayed (no clear path to first workspace)
```

## Activation event

**First workspace ready** (tenant exists, workspace selected, models resolved — server defaults OK). Everything in the wizard reverse-engineers to that moment.

## Evidence

| Before | File |
|--------|------|
| Create Tenant chip storm | [evidence/before-create-tenant.png](evidence/before-create-tenant.png) |
| Create Workspace (better defaults, still one-shot) | [evidence/before-create-workspace.png](evidence/before-create-workspace.png) |
