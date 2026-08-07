# E2E-109-08 Playwright notes

Spec: `edgequake_webui/e2e/spec109-reasoning-effort.spec.ts`

- Asserts query settings sheet exposes `reasoning-effort-select`.
- Asserts server LLM config card exposes the same control when settings page is reachable.

Run:

```bash
cd edgequake_webui
pnpm exec playwright test e2e/spec109-reasoning-effort.spec.ts
```

Requires live stack (`make dev-bg`) for non-skip paths; tests skip gracefully when auth/settings surfaces are absent.
