# SPEC-055 — Release Plan: v0.17.0

Date: 2026-07-14  
Target: **v0.17.0**  
Branch: feat/spec047-vision-ingest-spec048-progress → edgequake-main

---

## First Principles

```
Release = Version bump + Quality gates + Git tag + CI/CD publishes artifacts

WHY CI/CD: Never publish manually. The release-docker.yml workflow is triggered
by a semver git tag (v*.*.*). Quality gates run inside CI, not locally, so the
release artifact is built from the same state that passed automated checks.

WHY publish pdf2md first: The Docker build context is edgequake/ only. The
path dep ../../edgequake-pdf2md is outside that context and fails in CI.
Fix: publish edgequake-pdf2md 0.9.7 to crates.io, then switch from path dep
to registry dep before cutting the main release.
```

---

## Release Checklist

```
Phase 0 — edgequake-pdf2md
[ ] 0-A  Commit uncommitted changes in edgequake-pdf2md
[ ] 0-B  Verify edgequake-pdf2md version = 0.9.7 (already set)
[ ] 0-C  cargo publish edgequake-pdf2md (crates.io)
         GATE: crates.io shows 0.9.7 as latest

Phase 1 — Dependency switch
[ ] 1-A  Switch edgequake/Cargo.toml pdf2md dep from path+version to registry
         BEFORE: { path = "../../edgequake-pdf2md", version = "0.9.7", ... }
         AFTER:  { version = "0.9.7", ... }
[ ] 1-B  cargo generate-lockfile  (refreshes Cargo.lock source to registry)
[ ] 1-C  cargo build -p edgequake --no-default-features  (smoke check)

Phase 2 — Version bump (0.16.0 → 0.17.0)
[ ] 2-A  edgequake/Cargo.toml  version = "0.17.0"
[ ] 2-B  edgequake_webui/package.json  version = "0.17.0"
[ ] 2-C  VERSION  0.17.0
[ ] 2-D  README.md  badge + docker image tags
[ ] 2-E  CHANGELOG.md  promote [Unreleased] → [0.17.0] — 2026-07-14

Phase 3 — Quality gates (local)
[ ] 3-A  cargo fmt --all -- --check
[ ] 3-B  cargo clippy --workspace --lib --locked -- -D warnings
[ ] 3-C  cargo test --workspace --lib --locked
[ ] 3-D  ./scripts/release_gates.sh
[ ] 3-E  bun test (webui unit)

Phase 4 — Commit + PR + merge
[ ] 4-A  git add -A
[ ] 4-B  git commit -m "release: bump to v0.17.0"
[ ] 4-C  git push origin feat/spec047-vision-ingest-spec048-progress
[ ] 4-D  Create PR → edgequake-main (wait for CI green)
[ ] 4-E  Merge PR

Phase 5 — Tag + CI/CD trigger
[ ] 5-A  git checkout edgequake-main && git pull
[ ] 5-B  git tag v0.17.0
[ ] 5-C  git push origin v0.17.0
         → Triggers release-docker.yml:
           - Quality gates (fmt, clippy, tests, SPEC proofs)
           - Build amd64 Docker image
           - Build arm64 Docker image
           - Merge manifest as :0.17.0 and :latest on GHCR
```

---

## What ships in v0.17.0

### New Features (since 0.16.0)
- **SPEC-047**: Vision ingest — PDF → Markdown via VLM with page-level progress
- **SPEC-048**: Real-time ingestion progress via WebSocket
- **SPEC-050**: Pipeline UX parity — deletion progress, stage visibility
- **SPEC-052**: Dialog layout audit — all dialogs properly bounded (CSS Grid fix)
- **SPEC-053**: Graph search reliability — removes O(V+E) semaphore from search
- **SPEC-053**: UNION→OR fix in incident-edges query (json has no equality op)
- **Migration 086**: Edge BFS index reconcile

### Bug Fixes
- **#297**: Workspace delete now drops the per-workspace vector table (no orphan)
- **#296**: `proxyClientMaxBodySize` uses correct `SizeLimit` type (was `string`)
- **#294**: Warning emitted when API keys fall back to instance-local KV store
- **#292**: Docker image published as 0.16.0 (was missing 0.15.1)
- **SPEC-054**: 7 GitHub issues triaged and closed

### Performance
- `pg_get_incident_edges_batch`: O(log E) via `"EDGE"` child table + BitmapOr
- `pg_node_degrees_batch`: O(log E) via property indexes, no vertex JOIN
- `stream_graph`: semaphore released after data fetch, not end of SSE stream

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.17.0        (multi-arch manifest)
ghcr.io/raphaelmansuy/edgequake:latest         (aliases to 0.17.0)
ghcr.io/raphaelmansuy/edgequake:0.17.0-amd64  (linux/amd64)
ghcr.io/raphaelmansuy/edgequake:0.17.0-arm64  (linux/arm64)
```
