# 11 — Honest assessment

## What this spec can claim

- Multi-PDF **admit** reliability and honesty (Plane A).
- Non-blocking wake delivery when channel is saturated.
- Per-file UI failure isolation under client concurrency 3.
- Test coverage gap closed for multi-PDF (not only MD).

## What this spec must not claim

- Faster end-to-end KG completion for N PDFs (#361 / SPEC-122).
- That vision convert will keep up with admit rate under Docker defaults.
- That switching to `/pdf/batch` would fix #378 (body-sum risk; WebUI stays N×).

## Residual risks

| Risk | Residual |
|------|----------|
| Slow BYTEA under DB pressure | Client timeout → per-file error (honest) |
| Vision starvation | UX vocabulary + SPEC-122 FAQ |
| Partner still says “stuck” while queued | Education / chips; measure if needed |

## Cross-refs

- SPEC-122: [../122-implementation/](../122-implementation/)
- Reproduction: [12-reproduction.md](12-reproduction.md)
