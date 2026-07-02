#!/usr/bin/env bash
# Close SPEC-040 GitHub issues with detailed closure comments.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO="${GITHUB_REPOSITORY:-raphaelmansuy/edgequake}"
VERSION="${1:-0.13.2}"

comment_for() {
  local issue="$1"
  case "$issue" in
    262)
      cat <<EOF
Fixed in **v${VERSION}** (SPEC-040).

**Root cause:** AGE indexes on parent inheritance tables while queries scan child \`"Node"\`/\`"EDGE"\` tables → nested-loop plans and 15s graph stream timeout.

**Fix:**
- Migration \`078_age_child_workspace_stats.sql\` — child workspace + text-cast edge indexes + ANALYZE
- \`graph_lifecycle.rs\` — startup ensure for \`idx_edge_start_id_text\` / \`idx_edge_end_id_text\`
- Ops: \`migrations/support/078/concurrent.sql\` for large production graphs

**Tests:** \`graph_sota_tests\` 11/11 passed.

**Docs:** \`specs/040-edgequake-issues/\`
EOF
      ;;
    259)
      cat <<EOF
Fixed in **v${VERSION}** (SPEC-040).

**Root cause:** Stale \`conversation_id\` after workspace switch → FK violation on assistant message INSERT.

**Fix:**
- \`conversation_guard.rs\` + pre-save checks in \`streaming.rs\` / \`completion.rs\`
- UI clears conversation on tenant/workspace switch
- \`CONVERSATION_GONE\` SSE code + client recovery

**Tests:** Playwright stale-conversation + spec040-workspace-switch (5/5); \`conversation-errors.test.ts\` (3/3).

**Docs:** \`specs/040-edgequake-issues/\`
EOF
      ;;
    253)
      cat <<EOF
Fixed in **v${VERSION}** (SPEC-040).

**Root cause:** Orphan \`doc:hash:*\` KV keys without visible metadata caused duplicate-upload loop.

**Fix:**
- \`workspace_content_hash_dedup.rs\` — \`recycle_orphan_workspace_hash()\`
- Integrated in \`document_reingest.rs\` before \`StillProcessing\`
- Upload replace toasts in \`use-file-upload.ts\`

**Tests:** \`orphan_content_hash_is_recycled_on_reupload\` integration test passed.

**Docs:** \`specs/040-edgequake-issues/\`
EOF
      ;;
    251)
      cat <<EOF
Fixed in **v${VERSION}** (SPEC-040).

**Root cause:** \`load_bundled_models_config()\` preferred embedded catalog over runtime \`EDGEQUAKE_MODELS_CONFIG\`.

**Fix:** Runtime-first precedence in \`bundled_models.rs\`:
1. \`ModelsConfig::load()\` (env / cwd / home)
2. Embedded \`models.toml\` fallback
3. Builtin defaults

**Tests:** \`runtime_models_config_overrides_bundled\` + provider catalog tests passed.

**Docs:** \`specs/040-edgequake-issues/\`
EOF
      ;;
    250)
      cat <<EOF
Fixed in **v${VERSION}** (SPEC-040).

**Root cause:** Docker frontend build did not inject release version; UI showed stale \`package.json\` semver while API reported \`Cargo.toml\` version.

**Fix:**
- \`release-docker.yml\` passes \`NEXT_PUBLIC_APP_VERSION\` from release tag to frontend build
- \`edgequake_webui/Dockerfile\` — build-arg + ENV
- \`release_gates.sh\` — semver parity gate (API vs WebUI)
- Both artifacts bumped to **${VERSION}**

**Verify after pull:** UI footer and \`GET /health\` both show \`${VERSION}\`.

**Docs:** \`specs/040-edgequake-issues/\`
EOF
      ;;
    *)
      echo "Unknown issue: $issue" >&2
      return 1
      ;;
  esac
}

for issue in 262 259 253 251 250; do
  echo "Closing #${issue}..."
  gh issue close "$issue" \
    --repo "$REPO" \
    --comment "$(comment_for "$issue")"
done

echo "Done. Closed SPEC-040 issues on ${REPO}."
