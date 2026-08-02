# LENS — Database Expert (SPEC-101)

## Atomic initialize

Single transaction:

1. Insert `users` (admin, argon2 hash) if needed  
2. Insert `tenants` (+ metadata defaults)  
3. Insert `workspaces` (+ metadata / inheritance)  
4. Insert `memberships` (owner on tenant+workspace)  

Rollback on any failure → **no half-provisioned rows** (LAW-101-4).

## Idempotency

- Guard: if `has_login_users && tenant_count > 0` → `409 AlreadyInitialized`  
- Stable UUIDs for legacy Default (`…0002` / `…0003`) only when `ensure_defaults` runs  
- Wizard-created entities use random UUIDs (normal create path)

## Metadata JSONB (unchanged)

| Entity | Model keys |
|--------|------------|
| Tenant | `default_llm_*`, `default_embedding_*`, `default_vision_llm_*` |
| Workspace | `llm_*`, `embedding_*`, `vision_llm_*` |

Empty / omitted → resolve via inheritance ladder at runtime.

## Upgrade safety

Existing deployments with Default tenant: `needs_setup=false`. Never delete `…0002`/`…0003` in migrations for this SPEC.
