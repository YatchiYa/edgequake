# SPEC-169 — Perfect user management

Users, roles, invitations, sign-in links and passwords at the level of a best-in-class SaaS member screen, with first-class Keycloak awareness.

## Two role axes (never conflated)

| Axis | Values | Stored in | Decides |
| --- | --- | --- | --- |
| Instance role | `admin`, `user`, `readonly` | `users.role` | Platform powers: SSO, models, suspend a user |
| Organization role | `owner`, `admin`, `member`, `readonly` (UI: Viewer) | `memberships.role` | What a person may do inside one organization |

SSO members: the IdP role claim maps to the organization role at every sign-in, capped by `max_role`. A manual edit **pins** the role. A removed SSO member returns at the next IdP sign-in (the confirm dialog says so).

## API (all under `/api/v1`)

| Route | Who | Purpose |
| --- | --- | --- |
| `GET /roles` | any signed-in user | Role catalog, built from the enforcement predicates |
| `GET /tenants/{t}/members` | manager | Directory: filter, sort, page, summary, seats |
| `PATCH/DELETE /tenants/{t}/members/{u}` | manager (account fields: platform admin) | Role, suspend, instance role, removal |
| `POST /tenants/{t}/invitations` · `DELETE …/{id}` · `POST …/{id}/resend` | manager | Invitation lifecycle |
| `GET/POST /auth/invitations/{token}[/accept]` | public, token-gated | Preview and join |
| `POST /tenants/{t}/members/{u}/access-links` | manager who outranks the target | One-time `magic_login` or `password_reset` link |
| `GET /auth/access-links/{token}` · `POST …/redeem` | public, token-gated | Preview, then sign in (reset also sets the password) |
| `POST /auth/password` | signed in | Change own password; returns a fresh session |

Invariants: the last Owner and the last platform admin cannot be demoted, removed or suspended. Every token is 256-bit random, stored only as SHA-256, shown once, single use and atomically claimed. Re-issuing supersedes the previous link of the same purpose. A reset or password change revokes every refresh token and open link of that user.

## Data

- `213_spec169_tenant_invitations.sql` — `tenant_invitations`, one open invite per (tenant, lower(email)).
- `214_spec169_user_access_links.sql` — `user_access_links`, one open link per (user, purpose) via a partial unique index.

No SMTP: links are shared by the admin out of band, exactly like invitations. A person who forgot their password asks an admin (the login page says so).

## Where things live

- Backend: `handlers/members/*`, `handlers/access_links/*` (pure `guards.rs`), `services/invitations/*`, `services/access_links/*`, `handlers/auth/user_provision.rs`.
- Frontend: `components/members/*`, `components/auth/{password-field,password-strength,new-password-fields,change-password-dialog}.tsx`, `lib/auth/password-policy.ts` (mirrors `validate_password_strength`), `lib/members/permissions.ts` (mirrors the guards).

## Evidence

- Rust: `tests/e2e_spec169_members.rs` (12), `tests/e2e_spec169_access_links.rs` (7), `tests/e2e_spec169_access_links_pg.rs` (memory always; PostgreSQL with `EDGEQUAKE_SPEC169_PG_URL`), unit tests in `access_links/guards.rs`, `services/access_links/*`.
- Playwright (hermetic, `@spec169`): `edgequake_webui/e2e/spec169/{members,access}.spec.ts`.
  Run: `cd edgequake_webui && SPEC169_ROUND=round-NN npx playwright test e2e/spec169 --project=mock-api`.
- Screenshots: `e2e/screenshots/round-00-baseline` (the old Settings card), `round-01` (members), `round-02` (access links, passwords, invitation page, mobile).
