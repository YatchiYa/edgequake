# 05 — E2E Test Matrix (SPEC-110)

> Proof target: `make spec110-migrate-118-proof`  
> Artifacts: [measurements/](measurements/)

## Matrix

| ID | Case | Harness | Required |
|----|------|---------|----------|
| **E2E-110-01** | Seed two workspaces + `wsdoc:WS1:DOC` + `wsdoc:WS2:DOC` in an `eq_*_kv` table → **old** 118 SQL raises `21000` | Docker Postgres + SQL fixture (or capture once in measurements) | Yes (repro) |
| **E2E-110-02** | Same seed → **patched** 118 succeeds; exactly one `documents` row for DOC; `workspace_id = least(WS1, WS2)` | Same + assert query | Yes |
| **E2E-110-03** | Re-run patched 118 body (or full migrate twice) → no error; COALESCE leaves non-NULL workspace stable | Same | Yes |
| **E2E-110-04** | Two `injection::…` metadata keys same inj id, different workspaces → patched 121 succeeds (one row) | Same | Yes |
| **E2E-110-05** | Source guard: `118_*.sql` and `121_*.sql` contain `DISTINCT ON` on conflict id; must not use bare `SELECT DISTINCT` before `ON CONFLICT (id) DO UPDATE` for wsdoc/injection | `#[test]` grepping migration files | Always (no DB) |
| **E2E-110-06** | `_sqlx_migrations` has version 118 success with broken SHA → repair without DEV_MODE errors loud; with DEV_MODE updates to fixed SHA | Unit/integration on reconcile module | Yes |
| **E2E-110-07** | Image proof: `docker run …:0.24.1 migrate` fails on fixture DB; local/`0.24.2` image succeeds | Script + log capture under measurements | Yes for release |

## Fixture sketch (E2E-110-01/02)

```sql
-- Assume public.workspaces + an eq_<tenant>_kv table exist (post-117 schema).
-- WS_A < WS_B lexicographically as UUID text for least() assertion.

INSERT INTO public.workspaces (workspace_id, …) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', …),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', …);

INSERT INTO eq_demo_kv (key, value) VALUES
  ('wsdoc:aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:dddddddd-dddd-dddd-dddd-dddddddddddd', '{}'),
  ('wsdoc:bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb:dddddddd-dddd-dddd-dddd-dddddddddddd', '{}');
```

Assert after patched 118:

```sql
SELECT id, workspace_id FROM public.documents
WHERE id = 'dddddddd-dddd-dddd-dddd-dddddddddddd';
-- expect workspace_id = aaaaaaaa-…
```

## Suggested test locations (implementation)

| Gate | Suggested package / path |
|------|--------------------------|
| SQL fixture + apply | `edgequake-api` or `edgequake-storage` postgres e2e test (soft-skip without `DATABASE_URL`) |
| Source guard | Same file or `tests/contract_spec110_migration_dedup.rs` |
| Checksum repair | Next to `m078` tests / new `m118` module tests |
| Docker proof | `scripts/spec110_migrate_118_proof.sh` invoked by Makefile |

## Makefile contract

```makefile
# Pseudocode — land in root Makefile during implementation wave
spec110-migrate-118-proof:
        @echo "SPEC-110: wsdoc ON CONFLICT proof"
        ./scripts/spec110_migrate_118_proof.sh
```

Script should:

1. Ensure Postgres (reuse `make postgres-start` or compose).
2. Apply schema through 117 (or restore fixture dump).
3. Seed multi-ws wsdoc keys.
4. Run patched migrator / cargo test gates.
5. Write `measurements/e2e110-*.txt` and optional `SUMMARY.md` update.

## Pass criteria for release cut

- E2E-110-02, 03, 04, 05, 06 green in CI (or soft-skip DB with 05 always on).
- E2E-110-01 evidence recorded (historical failure of old SQL) in measurements.
- E2E-110-07 recorded before tagging **v0.24.2** (or local binary proof if GHCR not yet published).
