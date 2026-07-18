# SPEC-071 — Wave-2 greenfield turnkey

**Status:** Active  
**Depends on:** SPEC-064–069 (Wave-2 floors proven; dedicated mid-scale wall)  
**Goal:** One-recipe greenfield path for the supported 100k shape (recipe, warmup, `/ready` clarity, docs). Floors stay at 100k; no DiskANN in this pack.

## Commands

```bash
# Print / export turnkey env (opt-in — no silent DB flip)
make wave2-greenfield-env
WAVE2_GREENFIELD=1 make dev   # or make backend-bg

# Warm partial HNSW for workspaces (API or script)
curl -X POST http://localhost:8080/api/v1/admin/ann/warmup \
  -H 'Content-Type: application/json' \
  -d '{"workspace_ids":["<uuid>"]}'
./scripts/wave2_warmup.sh

make product-limits-check
```

## Locked decisions

- No silent halfvec/partial flip on existing DBs
- Floors unchanged (`highest_green_N=100k`)
- DiskANN deferred to SPEC-070
- `/ready` = catalog ANN presence when Wave-2 flag on (not plan-shape)

## Checklist

- [x] Recipe + make + .env.example
- [x] Warmup API + script + /ready clarity + e2e
- [x] Docs + product-limits-check
