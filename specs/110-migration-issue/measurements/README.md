# SPEC-110 measurements

> Evidence slots for E2E-110 and release proof. See **[SUMMARY.md](SUMMARY.md)** for the brutal assessment.

## Layout

```text
measurements/
  README.md
  SUMMARY.md                  ← honest pass/fail + residual risks
  e2e110-repro-0241.txt       ← E2E-110-01 evidence (old SQL failure path)
  e2e110-patched-ok.txt       ← E2E-110-01..05 cargo e2e
  e2e110-source-guard.txt     ← contract_spec110
  e2e110-checksum-repair.txt  ← contract + spec083 loud-refuse
  e2e110-checksums-after.txt  ← lockfile + check_migration_checksums.sh
```

## Status

| Artifact | Status |
|----------|--------|
| Pack authored | **Done** |
| Implementation + local e2e proof | **Done** (`make spec110-migrate-118-proof`) |
| Brutal assessment | **Done** — [SUMMARY.md](SUMMARY.md) |
| v0.24.2 GHCR | **Pending** cut (deferred; SPEC-109 WIP in tree) |
