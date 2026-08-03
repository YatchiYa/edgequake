# LENS — Full Stack Developer (SPEC-101)

## FE composition

```ascii
HeaderTenantSelector / TenantGuard / FirstRunGate
        │
        ▼
   *Wizard (entry)
        │
        ├── WizardShell (chrome)
        ├── steps/* (presentational)
        ├── hooks/use-*-wizard (mutations)
        └── lib/onboarding/* (pure)
```

## API contracts

### `GET /api/v1/setup/status`

```json
{
  "needs_setup": true,
  "has_login_users": false,
  "tenant_count": 0,
  "workspace_count": 0,
  "auth_enabled": true,
  "bootstrap_admin_configured": false
}
```

### `POST /api/v1/setup/initialize`

Body: admin credentials (optional if bootstrap already configured) + tenant basics + workspace basics + optional model overrides.  
Response: `{ tenant, workspace, user? }`  
Errors: `409` already initialized; `400` validation; `503` if auth off incorrectly used.

## Inheritance (runtime — do not change)

```
Request override
  → Workspace metadata
  → Tenant default_*
  → server_config.llm_defaults
  → Env EDGEQUAKE_DEFAULT_*
  → Compiled / models.toml
```

Create payloads **omit** model fields when `useServerDefaults=true` so the ladder applies (LAW-101-5).

## ensure_defaults gate

```
provision_defaults =
  EDGEQUAKE_PROVISION_DEFAULTS=true
  OR EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD set
  OR tenant_count > 0 already
```

Else skip silent Default seed (fresh wizard owns creation).
