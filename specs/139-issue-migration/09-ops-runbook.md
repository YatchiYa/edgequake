# 09 — Ops runbook (0.26.1 → SPEC-139 engine)

Confirm-drop remains consent-gated. Backup before DROP OLD.

**Pin:** product `VERSION` is still **0.26.2**. GHCR `0.26.3` exists only after
`make version-bump VERSION=0.26.3` and `git tag v0.26.3`. Until then deploy
**this branch / Unreleased binary**, not `ghcr.io/raphaelmansuy/edgequake:0.26.1`.

```text
1. Backup (pg_dump -Fc / volume snapshot)
2. Deploy the SPEC-139 engine binary (schema train still 149 — no new sqlx)
3. Set EDGEQUAKE_MIGRATION_MODE=automatic
4. Start the server — engine resumes:
   - w1-chunk-text-backfill
   - w2-dedup-remainder
   - wc-shell-remainder
   - w5-artifact-remainder
   - w3-chunk-embedding-backfill (reclaims prior verify-failed)
   - iw2-fleet-embedding-backfill (deduped UNNEST)
   - iw2-fleet-provenance-stamp
5. Watch: edgequake migrate status
          edgequake migrate guard
6. Optional: EDGEQUAKE_MIGRATION_VERIFY_EQUALITY=1 only if you need sampled
   vector equality as a hard gate (default is coverage, matching 126)
7. GREEN + backup:
   edgequake migrate --confirm-drop
8. edgequake migrate   # 142 emptiness assert
9. Verify /health version (0.26.3 only after the tag)
```

## Do not

- `--confirm-drop` while guard is RED
- Edit applied 125/126/131 SQL
- Set `EDGEQUAKE_MIGRATION_CONFIRM_DROP=1` in a shared `.env`
- Expect leftover **orphans** (no typed parent) or SPEC-111 alias stalls to
  go GREEN without classification — remainder jobs copy what they can

## Guard before migrate

If guard errors with typed SSOT missing (`document_artifacts` does not exist):
run `edgequake migrate` first, then guard.

## Dual-legacy stalls

If `uncovered_fleet` stays small and non-zero after iw2 + stamp: SPEC-111
stalls (many legacy keys → one typed row). Do not drop; see
[`specs/111-issues/09-ops-runbook.md`](../111-issues/09-ops-runbook.md).

## Detail

- Product pin when tagged: [`docs/operations/upgrade-to-0.26.3.md`](../../docs/operations/upgrade-to-0.26.3.md)
- Leftover 091 ladder: [`upgrade-to-0.26.0.md`](../../docs/operations/upgrade-to-0.26.0.md)
- Consent CLI: [`upgrade-to-0.26.1.md`](../../docs/operations/upgrade-to-0.26.1.md)
