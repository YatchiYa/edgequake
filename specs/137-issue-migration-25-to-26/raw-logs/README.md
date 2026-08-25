# raw-logs — sanitization rules (SPEC-137)

> **Do not commit `*.log`.** Root `.gitignore` drops `*.log`. Store sanitized
> transcripts as `.txt` / `.md` in this directory.

## Why this folder exists

Field migrate failures are reconstructed from operator stdout/stderr. Original
attachments are **not** stored in git (hosts, connection strings, names).

## Sanitize before adding a file

| Redact | Replacement |
|--------|-------------|
| Hostnames, IPs, ports | `db.example.internal:5432` |
| User / database names that identify a fleet | `edgequake` / `edgequake` |
| Passwords / tokens in URLs | `***` |
| Person or organization names | omit |
| Workspace / tenant display names | `ws-a`, `tenant-a` |

Keep: product version, pending migration numbers, SQLSTATE, abort markers
(`Wave D ABORT`, `W4 ABORT`, `IW2 ABORT`, `SPEC-105 migration 142`, checksum
drift), consent lines (`consent: INCLUDED` vs `NOT given`).

## Files

| File | What it is |
|------|------------|
| [ticket-sequence.txt](ticket-sequence.txt) | Operator steps as reported (anonymized) |
| [reconstructed-apply-soft-exit.txt](reconstructed-apply-soft-exit.txt) | Track A: `--drop-confirm` ignored → expandable soft-exit |
| [reconstructed-sql-abort.txt](reconstructed-sql-abort.txt) | Track B: consent given, SQL fail-closed (class catalog) |
