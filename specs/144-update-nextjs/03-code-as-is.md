# 03 — Code As-Is

## Dependency pin

| Package | Version |
|---------|---------|
| `next` | `16.2.11` |
| `react` / `react-dom` | `19.2.3` |
| `eslint-config-next` | `16.2.11` |
| Lockfile SSOT | `pnpm-lock.yaml` (`bun.lock` stale — ignore) |

## Network boundary (split)

```ascii
  HTTP request
       │
       ├──────────── root middleware.ts ──► auth redirect (cookie)
       │
       └──────────── src/proxy.ts ────────► /swagger-ui → /swagger-ui/ (307)
```

- [middleware.ts](../../edgequake_webui/middleware.ts): SPEC-083 X-27 auth when
  `NEXT_PUBLIC_AUTH_ENABLED` or `DISABLE_DEMO_LOGIN`.
- [src/proxy.ts](../../edgequake_webui/src/proxy.ts): swagger slash only.

## Config hotspots

[next.config.ts](../../edgequake_webui/next.config.ts):

- `compress: false` (SSE)
- `experimental.proxyTimeout` / `proxyClientMaxBodySize`
- `output: "standalone"`
- `skipTrailingSlashRedirect: true`
- Dev rewrites to backend

## Build paths

| Path | Command | Bundler |
|------|---------|---------|
| Local / CI safe | `scripts/safe-build.sh` | `--webpack` |
| Docker | `RUN npx next build` | Turbopack default (diverges) |

## Async params

- Most pages: client `useParams` / `useSearchParams`.
- `w/[slug]/page.tsx`: `params: Promise<…>` + `use(params)`.
- `swagger-ui/[[...path]]/route.ts`: `await context.params`.

## Existing Instant / loading shells

- Only `(dashboard)/documents/loading.tsx` found.
- No `cacheComponents` / `partialPrefetching` flags.

## Regression surfaces

SPEC-143 sync, SSE streaming, auth redirect, swagger UI, upload proxy,
PDF worker copy on `dev`.
