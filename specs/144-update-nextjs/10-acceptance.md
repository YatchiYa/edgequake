## Checklist

- [x] `next` and `eslint-config-next` pinned to **16.3.3** in package.json + pnpm lock
- [x] Root `middleware.ts` removed; single `src/proxy.ts` owns auth + swagger
- [x] `next.config.ts` keeps `compress: false` and numeric proxy limits
- [x] Dockerfile uses `next build --webpack` (parity with safe-build)
- [x] `pnpm typecheck` / proxy+pin unit tests / `next build --webpack` green
- [x] Auth (non-auth mode), API explorer, OODA-228, SPEC-143 e2e green on **:3010**
- [x] `spec144-next-upgrade-smoke.spec.ts` green on **:3010**
- [x] Instant Navigations: **flags off** with documented blocker; shells + `@next/playwright` prepared
- [x] Playwright isolated webServer / skip-stack default port is **3010**
- [x] No DB migration shipped
- [x] SPEC-144 doc pack complete (this directory)

## Definition of done

All boxes checked; README status board I1–T1 / A1 marked Done.
Instant Navigations flags may remain off if the documented React postpone
blocker persists — that is an accepted residual (see 11-honest-assessment).

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Honest assessment: [11-honest-assessment.md](11-honest-assessment.md)
