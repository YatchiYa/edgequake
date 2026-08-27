# 11 — Lens: Full Stack Developer

> **Cross-refs:** [Matrix](02-cross-ref-matrix.md) · [SPEC-124](../124-langfuse-support/)

## Surfaces

| Layer | K8s wiring |
|-------|------------|
| Web | `EDGEQUAKE_API_URL` via ConfigMap; ingress `edgequake.local` |
| API | Langfuse env from Secret + ConfigMap |
| Settings UI | `/api/v1/settings/langfuse` — unchanged contract |

## Request flow

```ascii
  Browser → ingress → edgequake-web → edgequake-api → postgres
                              │
                              └── Settings card reads langfuse DTO
  Browser → langfuse.local (Open in Langfuse link)
```

## Tests (reuse)

- [`spec124-langfuse-settings.spec.ts`](../../edgequake_webui/e2e/spec124-langfuse-settings.spec.ts)
- [`spec124-langfuse-sessions.spec.ts`](../../edgequake_webui/e2e/spec124-langfuse-sessions.spec.ts)

## DRY

Do not fork Playwright specs; parameterize via `PLAYWRIGHT_BASE_URL` port-forward.
