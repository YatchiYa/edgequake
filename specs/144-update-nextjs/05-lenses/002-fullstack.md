# Lens — Full Stack Developer

## Scope

Frontend-only framework bump + network-boundary refactor. No API contract
change. Preserve dev rewrites to Axum backend.

## Work packages

| ID | Change | Files |
|----|--------|-------|
| WP1 | Bump deps | `package.json`, `pnpm-lock.yaml` |
| WP2 | Unify proxy | `src/proxy.ts`; delete `middleware.ts` |
| WP3 | Build parity | `Dockerfile`, `safe-build.sh` comment |
| WP4 | Instant allowlist | `next.config.ts`, `loading.tsx`, e2e |
| WP5 | Smoke e2e | `e2e/spec144-next-upgrade-smoke.spec.ts` |

## DRY / SOLID checklist

- Extract `authGuard` / `swaggerSlash` helpers — no duplicated matcher logic.
- Single exported `proxy` + single `config.matcher`.
- Do not fork Next config constants; keep upload limits import SSOT.
- Do not reintroduce gzip for SSE.

## Verification commands

```bash
cd edgequake_webui
pnpm typecheck
pnpm test
pnpm run build
pnpm exec playwright test e2e/spec144-next-upgrade-smoke.spec.ts \
  e2e/ooda-228-critical-path.spec.ts \
  e2e/streaming-test.spec.ts \
  e2e/issue-180-auth-runtime-hardening.spec.ts \
  e2e/api-explorer.spec.ts \
  e2e/spec143-pdf-markdown-sync.spec.ts
```

## Failure modes

| Symptom | Likely cause |
|---------|--------------|
| NFT ENOENT on build | Turbopack standalone — keep `--webpack` |
| SSE arrives as one chunk | `compress` re-enabled |
| Auth deep link open | proxy matcher dropped auth |
| Blank swagger | trailing-slash redirect lost |
| Instant build errors | unguarded dynamic + flags on |

## Cross-refs

- As-is: [03-code-as-is.md](../03-code-as-is.md)
- Plan: [07-implementation-plan.md](../07-implementation-plan.md)
