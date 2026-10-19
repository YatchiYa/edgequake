-- ============================================================================
-- Migration 214: SPEC-169 — one-time access links (magic sign-in, password reset)
-- ============================================================================
-- An organization admin issues a link for a member; the member opens it once. The token is
-- never stored, only its SHA-256. Auth-plane table (read before any tenant context exists):
-- service-scoped like 164 / 213; every issue path checks the issuer's rights in the API layer.
-- `user_id` carries no FK: users live in the identity store, which differs per deployment mode.

SET search_path = public;

CREATE TABLE IF NOT EXISTS user_access_links (
    link_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL,
    tenant_id       UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    purpose         TEXT NOT NULL CHECK (purpose IN ('magic_login', 'password_reset')),
    token_hash      TEXT NOT NULL UNIQUE,
    created_by      UUID,
    created_by_name TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ
);

-- At most one live link per (user, purpose): re-issuing revokes the previous one.
CREATE UNIQUE INDEX IF NOT EXISTS uq_user_access_links_open
    ON user_access_links (user_id, purpose)
    WHERE used_at IS NULL AND revoked_at IS NULL;
