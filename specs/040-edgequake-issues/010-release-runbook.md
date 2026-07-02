# SPEC-040 — Release Runbook (v0.13.2)

**Release:** v0.13.2  
**Date:** 2026-07-02  
**Scope:** GitHub issues #250, #251, #253, #259, #262  
**Spec:** [008-implementation-plan.md](./008-implementation-plan.md)

---

## Pre-release checklist

- [x] All five issues implemented and cross-referenced in `009-cross-reference-matrix.md`
- [x] Migration `078_age_child_workspace_stats.sql` + `checksums.lock` updated
- [x] `release_gates.sh` semver parity gate
- [x] Docker frontend `NEXT_PUBLIC_APP_VERSION` wired in `release-docker.yml`
- [x] Version bumped: `VERSION`, `edgequake/Cargo.toml`, `edgequake_webui/package.json` → **0.13.2**
- [x] CHANGELOG `[0.13.2]` section written
- [ ] PR merged to `edgequake-main` (or release branch)
- [ ] GitHub issues closed with closure comments

---

## Release commands (maintainer)

```bash
# 1. Verify gates locally
./scripts/release_gates.sh

# 2. SPEC-040 targeted tests
cd edgequake
cargo test -p edgequake-storage --features postgres --test graph_sota_tests
cargo test -p edgequake-api --features postgres --test workspace_document_scope
cargo test -p edgequake-api --features postgres --lib bundled_models provider_catalog::tests

cd ../edgequake_webui
bun test src/lib/query/__tests__/conversation-errors.test.ts
PLAYWRIGHT_BASE_URL=http://localhost:3000 pnpm exec playwright test \
  e2e/spec040-workspace-switch-conversation.spec.ts \
  e2e/stale-conversation-recovery.spec.ts

# 3. Commit release (if not already done)
git add VERSION CHANGELOG.md edgequake/Cargo.toml edgequake_webui/package.json \
  edgequake/migrations/checksums.lock specs/040-edgequake-issues/
git commit -m "$(cat <<'EOF'
Release v0.13.2 — SPEC-040 issue fixes (#250–#253, #259, #262)

Graph index repair (M078), ghost duplicate recycle, conversation FK guard,
models.toml runtime precedence, and Docker/UI version parity.
EOF
)"

# 4. Tag and push (triggers release-docker.yml)
git tag v0.13.2
git push origin HEAD
git push origin v0.13.2

# 5. Monitor CI
gh run list --workflow=release-docker.yml --limit 3
gh release view v0.13.2
```

---

## Post-release verification

```bash
# Confirm M078 auto-applied on upgrade (sqlx on backend start)
docker exec edgequake-postgres psql -U edgequake -d edgequake \
  -c "SELECT version, description, installed_on FROM _sqlx_migrations WHERE version = 78;"

# Performance proof (local or staging)
./specs/040-edgequake-issues/e2e/measure_graph_stats_perf.sh

# Docker quickstart with pinned version
EDGEQUAKE_VERSION=0.13.2 docker compose -f docker-compose.quickstart.yml up -d
curl -s http://localhost:8080/health | jq .version

# Large production graphs (>100k nodes)
psql "$DATABASE_URL" -f edgequake/migrations/support/078/concurrent.sql
```

---

## Rollback

| Component | Rollback |
| --------- | -------- |
| API/Frontend images | Deploy previous tag `v0.13.1` |
| M078 indexes | `DROP INDEX` on child `"Node"`/`"EDGE"` only — no data loss |
| Config | Revert `models.toml` loader change only if runtime override breaks deploy |

---

## GitHub issue closure

After tag is green, close with comments from `specs/040-edgequake-issues/e2e/issue-closure-comments.md` (or run `scripts/close_spec040_issues.sh`).
