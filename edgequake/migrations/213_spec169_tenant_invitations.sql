-- ============================================================================
-- Migration 213: SPEC-169 — organization invitations
-- ============================================================================
-- Pending "invite by email" grants. The accept token is never stored, only its SHA-256.
-- Auth-plane table: read before a tenant context exists (accept link), so — like the
-- SPEC-158 federation tables (164) — it is service-scoped; every query filters tenant_id.

SET search_path = public;

CREATE TABLE IF NOT EXISTS tenant_invitations (
    invitation_id   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    workspace_id    UUID REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    role            TEXT NOT NULL DEFAULT 'member'
                    CHECK (role IN ('owner', 'admin', 'member', 'readonly')),
    token_hash      TEXT NOT NULL UNIQUE,
    invited_by      UUID,
    invited_by_name TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL,
    accepted_at     TIMESTAMPTZ,
    accepted_by     UUID,
    revoked_at      TIMESTAMPTZ
);

-- One open invitation per (tenant, email); accepted / revoked rows free the address.
CREATE UNIQUE INDEX IF NOT EXISTS uq_tenant_invitations_open_email
    ON tenant_invitations (tenant_id, lower(email))
    WHERE accepted_at IS NULL AND revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_tenant_invitations_tenant_open
    ON tenant_invitations (tenant_id, created_at)
    WHERE accepted_at IS NULL AND revoked_at IS NULL;
