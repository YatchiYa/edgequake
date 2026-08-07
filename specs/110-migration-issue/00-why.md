# 00 — Why SPEC-110

## Trigger

A partner running EdgeQuake in PPD cannot complete the SPEC-091 upgrade path. They invoke:

```bash
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:0.24.1 migrate --confirm-drop
```

Preflight reports 24 pending migrations (118–142, skipping 127). Apply starts, then aborts on **migration 118** (`spec091 wsdoc backfill`) with:

```text
ON CONFLICT DO UPDATE command cannot affect row a second time
```

Database state after failure: **`latest_applied = 117`**. Migration 118 did not record success (sqlx transactional apply → rollback). Serving boot (LD-15) will refuse to start while SAFE SCHEMA / pending migrate train remains behind — exit **78** with a migrate hint.

## User impact

| Layer | Impact |
|-------|--------|
| Ops | Upgrade to typed SSOT (091) blocked; DROP OLD waves never reached |
| Product | Cannot cut over from KV membership index to `documents.workspace_id` |
| Serving | New binary may refuse boot until migrate succeeds |
| Trust | Partner correctly concludes a **new image** is required |

## Why this is a product defect (not operator error)

- Consent (`--confirm-drop`) and pool connectivity are fine.
- Preflight correctly classifies 118 as SAFE SCHEMA.
- Failure is deterministic Postgres law on the **embedded** SQL shipped in v0.24.1.
- The partner DB simply has ≥1 document id present under ≥2 `wsdoc:{ws}:{doc}` keys — a legitimate legacy membership shape.

## Non-goals

- Redesigning multi-tenant document membership as a join table.
- Skipping or reordering SPEC-091 drop waves.
- “Hot-patching” SQL inside a running `0.24.1` container without a rebuild.

## Success condition

Partner re-runs migrate with **v0.24.2** (or equivalent patched binary), migration 118+ apply cleanly on multi-ws wsdoc fixtures, and e2e gates in [05-e2e-test-matrix.md](05-e2e-test-matrix.md) are green with artifacts under [measurements/](measurements/).
